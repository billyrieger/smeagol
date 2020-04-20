use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use smeagol::{Leaf, Rule};

const GLIDER: u64 = 0b_00000000_00000000_00001000_00000100_00011100_00000000_00000000_00000000;

fn glider(c: &mut Criterion) {
    let mut group = c.benchmark_group("glider");
    for step in 0u8..10 {
        group.throughput(Throughput::Elements(step as u64));
        group.bench_with_input(BenchmarkId::from_parameter(step), &step, |b, &step| {
            b.iter(|| {
                Rule::new(&[3], &[2, 3]).step(Leaf::new(GLIDER), step);
            })
        });
    }
}

criterion_group!(benches, glider);
criterion_main!(benches);
