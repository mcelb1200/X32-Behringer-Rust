use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::HashMap;

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_only");
    for size in [40_u8, 100_u8].iter() {
        let mut saved_strips_vec = Vec::new();
        let mut saved_strips_map = HashMap::new();

        for i in 1..=*size {
            let strip_data = vec!["state".to_string(); 30];
            saved_strips_vec.push((i, strip_data.clone()));
            saved_strips_map.insert(i, strip_data);
        }

        group.bench_with_input(BenchmarkId::new("vec", size), size, |b, _| {
            b.iter(|| {
                // simulate lookup for all elements
                for i in 1..=*size {
                    let _strip_data = &saved_strips_vec
                        .iter()
                        .find(|(src, _)| *src == i)
                        .unwrap()
                        .1;
                }
            })
        });

        group.bench_with_input(BenchmarkId::new("map", size), size, |b, _| {
            b.iter(|| {
                // lookup for all elements
                for i in 1..=*size {
                    let _strip_data = saved_strips_map.get(&i).unwrap();
                }
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_lookup);
criterion_main!(benches);
