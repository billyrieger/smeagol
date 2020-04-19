use criterion::{black_box, criterion_group, criterion_main, Criterion};

use smeagol::{Leaf, Rule};

const GLIDER: u64 = 0b_00000000_00000000_00001000_00000100_00011100_00000000_00000000_00000000;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("glider", |b| {
        b.iter(|| {
            Rule::new(&[3], &[2, 3]).step(Leaf::new(black_box(GLIDER)));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
