// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::{rngs::StdRng, Rng, SeedableRng};
use smeagol::{Leaf, LIFE};

fn steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("random");
    let mut leaf = Leaf::new(StdRng::seed_from_u64(56789).gen());

    for n in 1..6 {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                for _ in 0..n {
                    leaf = leaf.tick(&LIFE);
                }
            })
        });
    }
}

criterion_group!(benches, steps);
criterion_main!(benches);
