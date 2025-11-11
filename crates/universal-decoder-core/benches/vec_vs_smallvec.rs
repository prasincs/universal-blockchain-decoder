use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use smallvec::SmallVec;

/// Benchmark Vec vs SmallVec for common transaction parsing scenarios

fn bench_vec_small_elements(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_elements");

    // Test with 1-8 elements (typical for signatures, inputs, outputs)
    for size in [1, 2, 4, 8] {
        group.bench_with_input(BenchmarkId::new("vec", size), &size, |b, &size| {
            b.iter(|| {
                let mut v = Vec::new();
                for i in 0..size {
                    v.push(black_box(i));
                }
                black_box(v)
            })
        });

        group.bench_with_input(BenchmarkId::new("smallvec_8", size), &size, |b, &size| {
            b.iter(|| {
                let mut v = SmallVec::<[u32; 8]>::new();
                for i in 0..size {
                    v.push(black_box(i));
                }
                black_box(v)
            })
        });

        group.bench_with_input(BenchmarkId::new("smallvec_4", size), &size, |b, &size| {
            b.iter(|| {
                let mut v = SmallVec::<[u32; 4]>::new();
                for i in 0..size {
                    v.push(black_box(i));
                }
                black_box(v)
            })
        });
    }
    group.finish();
}

fn bench_vec_medium_elements(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_elements");

    // Test with 16-64 elements (large transactions)
    for size in [16, 32, 64] {
        group.bench_with_input(BenchmarkId::new("vec", size), &size, |b, &size| {
            b.iter(|| {
                let mut v = Vec::new();
                for i in 0..size {
                    v.push(black_box(i));
                }
                black_box(v)
            })
        });

        group.bench_with_input(BenchmarkId::new("smallvec_8", size), &size, |b, &size| {
            b.iter(|| {
                let mut v = SmallVec::<[u32; 8]>::new();
                for i in 0..size {
                    v.push(black_box(i));
                }
                black_box(v)
            })
        });
    }
    group.finish();
}

fn bench_vec_byte_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("byte_data");

    // Test with byte vectors (more realistic for transaction data)
    let sizes = [4, 8, 16, 32, 64];

    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        group.bench_with_input(BenchmarkId::new("vec", size), &data, |b, data| {
            b.iter(|| {
                let mut v = Vec::new();
                for &byte in data {
                    v.push(black_box(byte));
                }
                black_box(v)
            })
        });

        group.bench_with_input(BenchmarkId::new("smallvec_32", size), &data, |b, data| {
            b.iter(|| {
                let mut v = SmallVec::<[u8; 32]>::new();
                for &byte in data {
                    v.push(black_box(byte));
                }
                black_box(v)
            })
        });
    }
    group.finish();
}

fn bench_vec_preallocated(c: &mut Criterion) {
    let mut group = c.benchmark_group("preallocated");

    // Test with pre-allocated capacity (common pattern)
    for size in [4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("vec_with_capacity", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut v = Vec::with_capacity(size);
                    for i in 0..size as u32 {
                        v.push(black_box(i));
                    }
                    black_box(v)
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("smallvec_8", size), &size, |b, &size| {
            b.iter(|| {
                let mut v = SmallVec::<[u32; 8]>::new();
                for i in 0..size as u32 {
                    v.push(black_box(i));
                }
                black_box(v)
            })
        });
    }
    group.finish();
}

fn bench_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration");

    let size = 8;
    let data: Vec<u32> = (0..size).collect();
    let smallvec_data: SmallVec<[u32; 8]> = (0..size).collect();

    group.bench_function("vec_iter", |b| {
        b.iter(|| {
            let sum: u32 = data.iter().map(|&x| black_box(x)).sum();
            black_box(sum)
        })
    });

    group.bench_function("smallvec_iter", |b| {
        b.iter(|| {
            let sum: u32 = smallvec_data.iter().map(|&x| black_box(x)).sum();
            black_box(sum)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vec_small_elements,
    bench_vec_medium_elements,
    bench_vec_byte_data,
    bench_vec_preallocated,
    bench_iteration,
);
criterion_main!(benches);
