use criterion::{Criterion, criterion_group, criterion_main};

fn dummy_benchmark(c: &mut Criterion) {
    c.bench_function("dummy", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, dummy_benchmark);
criterion_main!(benches);
