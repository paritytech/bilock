// Copyright (c) 2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use criterion::{criterion_group, criterion_main, Criterion};
use futures::{executor::LocalPool, future, ready, task::SpawnExt};
use std::task::{Context, Poll};

const ITERATIONS: usize = 5000;

criterion_group!(benches, bench);
criterion_main!(benches);

fn bench(c: &mut Criterion) {
    c.bench_function("bilock::BiLock", |b| {
        use bilock::BiLock;
        let mut pool = LocalPool::new();
        b.iter(|| {
            let (a, b) = BiLock::new(0u64);
            let spawner = pool.spawner();

            spawner.spawn(async move {
                for _ in 0 .. ITERATIONS {
                    future::poll_fn(|cx: &mut Context| {
                        let mut lock = ready!(a.poll_lock(cx));
                        *lock += 1;
                        Poll::Ready(())
                    })
                    .await
                }
            })
            .unwrap();

            spawner.spawn(async move {
                for _ in 0 .. ITERATIONS {
                    future::poll_fn(|cx: &mut Context| {
                        let mut lock = ready!(b.poll_lock(cx));
                        *lock += 1;
                        Poll::Ready(())
                    })
                    .await
                }
            })
            .unwrap();

            pool.run()
        })
    });

    c.bench_function("futures::lock::BiLock", |b| {
        use futures::lock::BiLock;
        let mut pool = LocalPool::new();
        b.iter(|| {
            let (a, b) = BiLock::new(0u64);
            let spawner = pool.spawner();

            spawner.spawn(async move {
                for _ in 0 .. ITERATIONS {
                    future::poll_fn(|cx: &mut Context| {
                        let mut lock = ready!(a.poll_lock(cx));
                        *lock += 1;
                        Poll::Ready(())
                    })
                    .await
                }
            })
            .unwrap();

            spawner.spawn(async move {
                for _ in 0 .. ITERATIONS {
                    future::poll_fn(|cx: &mut Context| {
                        let mut lock = ready!(b.poll_lock(cx));
                        *lock += 1;
                        Poll::Ready(())
                    })
                    .await
                }
            })
            .unwrap();

            pool.run()
        })
    });
}

