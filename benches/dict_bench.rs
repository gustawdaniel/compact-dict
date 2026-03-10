use criterion::{black_box, criterion_group, criterion_main, Criterion};
use compact_dict::dict::Dict;
use std::collections::HashMap as StdHashMap;
use hashbrown::HashMap as BrownHashMap;
use fxhash::FxHashMap;

fn generate_keys(count: usize) -> Vec<String> {
    let mut keys = Vec::with_capacity(count);
    for i in 0..count {
        keys.push(format!("key_{}", i));
    }
    keys
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("Insert_10k_Strings");
    let keys = generate_keys(10_000);

    group.bench_function("compact_dict", |b| {
        b.iter(|| {
            let mut dict = Dict::<u32>::new(10_000);
            for key in &keys {
                dict.put(key, 1);
            }
            black_box(dict);
        });
    });

    group.bench_function("std_hashmap", |b| {
        b.iter(|| {
            let mut map = StdHashMap::with_capacity(10_000);
            for key in &keys {
                map.insert(key, 1);
            }
            black_box(map);
        });
    });

    group.bench_function("hashbrown", |b| {
        b.iter(|| {
            let mut map = BrownHashMap::with_capacity(10_000);
            for key in &keys {
                map.insert(key, 1);
            }
            black_box(map);
        });
    });

    group.bench_function("fxhash", |b| {
        b.iter(|| {
            let mut map = FxHashMap::default();
            map.reserve(10_000);
            for key in &keys {
                map.insert(key, 1);
            }
            black_box(map);
        });
    });

    group.finish();
}

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("Get_10k_Strings");
    let keys = generate_keys(10_000);

    let mut dict = Dict::<u32>::new(10_000);
    let mut std_map = StdHashMap::with_capacity(10_000);
    let mut brown_map = BrownHashMap::with_capacity(10_000);
    let mut fx_map = FxHashMap::default();
    fx_map.reserve(10_000);

    for key in &keys {
        dict.put(key, 1);
        std_map.insert(key, 1);
        brown_map.insert(key, 1);
        fx_map.insert(key, 1);
    }

    group.bench_function("compact_dict", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(dict.get_or(key, 0));
            }
        });
    });

    group.bench_function("std_hashmap", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(std_map.get(key));
            }
        });
    });

    group.bench_function("hashbrown", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(brown_map.get(key));
            }
        });
    });

    group.bench_function("fxhash", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(fx_map.get(key));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_insert, bench_get);
criterion_main!(benches);
