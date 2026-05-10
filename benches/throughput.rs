//! Benchmarks for `ag_id` (Ag^id).

use ag_id::{derive, Domain};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

/// Benchmark `derive` with various input sizes.
fn bench_derive(c: &mut Criterion) {
    let mut g = c.benchmark_group("derive");

    for size in [16usize, 64, 256, 1024, 65536] {
        let input = vec![0xABu8; size];
        g.throughput(Throughput::Bytes(size as u64));
        g.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            &input,
            |b, input| b.iter(|| derive(black_box(Domain::User), black_box(input))),
        );
    }
    g.finish();
}

/// Benchmark display formatting of `Did`.
fn bench_display(c: &mut Criterion) {
    let id = derive(Domain::User, b"bench-display");
    c.bench_function("display/did_string", |b| {
        b.iter(|| black_box(id.to_string()));
    });
}

criterion_group!(benches, bench_derive, bench_display);
criterion_main!(benches);
