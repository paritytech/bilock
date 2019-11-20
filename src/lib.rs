// Copyright (c) 2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{mem, ops::{Deref, DerefMut}, sync::Arc, task::{Context, Poll, Waker}};
use try_lock::{TryLock, Locked};

/// A synchronisation primitive between two owners.
///
/// Similar to `futures::lock::BiLock` but does not allocate under contention.
#[derive(Debug)]
pub struct BiLock<T> {
    shared: Arc<Shared<T>>
}

#[derive(Debug)]
struct Shared<T> {
    value: TryLock<T>,
    waker: TryLock<Option<Waker>>
}

impl<T> BiLock<T> {
    /// Create a new `BiLock` for the given resource.
    pub fn new(value: T) -> (Self, Self) {
        let shared = Shared {
            value: TryLock::new(value),
            waker: TryLock::new(None)
        };
        let a = BiLock { shared: Arc::new(shared) };
        let b = BiLock { shared: a.shared.clone() };
        (a, b)
    }

    /// Try to acquire a lock.
    pub fn poll_lock(&self, cx: &mut Context) -> Poll<BiLockGuard<T>> {
        let mut registered = false;
        loop {
            if let Some(value) = self.shared.value.try_lock() {
                let b = BiLockGuard { owner: self, value: Some(value) };
                return Poll::Ready(b)
            }
            if registered {
                return Poll::Pending
            }
            if let Some(mut waker) = self.shared.waker.try_lock() {
                *waker = Some(cx.waker().clone());
                registered = true
            }
        }
    }

    /// Acquire a lock.
    #[cfg(feature = "futures")]
    pub async fn lock(&self) -> BiLockGuard<'_, T> {
        futures::future::poll_fn(|cx: &mut Context| self.poll_lock(cx)).await
    }
}

/// RAII guard returned from [`BiLock::poll_lock`].
///
/// After successful acquisition of a lock, the resource can be accessed
/// via the [`Deref`]/[`DerefMut`] trait impls of this type.
#[derive(Debug)]
pub struct BiLockGuard<'a, T> {
    owner: &'a BiLock<T>,
    value: Option<Locked<'a, T>>
}

impl<T> Drop for BiLockGuard<'_, T> {
    fn drop(&mut self) {
        mem::drop(self.value.take());
        loop {
            if let Some(mut waker) = self.owner.shared.waker.try_lock() {
                if let Some(w) = waker.take() {
                    w.wake()
                }
                return
            }
        }
    }
}

impl<T> Deref for BiLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T> DerefMut for BiLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}
