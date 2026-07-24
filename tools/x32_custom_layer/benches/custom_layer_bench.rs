use criterion::{criterion_group, criterion_main, Criterion};

fn dummy_benchmark(c: &mut Criterion) {
    c.bench_function("dummy", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, dummy_benchmark);
criterion_main!(benches);
