use criterion::{criterion_group, criterion_main, Criterion};
use fyre_fractal_function::{compute_all, f};
use std::hint::black_box;

pub fn bench_single(c: &mut Criterion) {
    c.bench_function("n=256", |b| b.iter(|| black_box(f(1.0, 0.5, 9.0, 256))));
}

pub fn bench_all(c: &mut Criterion) {
    // Create a file to write the results
    let file = std::fs::File::create("results/bench_results.txt").unwrap();
    let mut writer = std::io::BufWriter::new(file);

    c.bench_function("n=256, 10000 computations", |b| {
        b.iter(|| {
            black_box(compute_all(0.0, 1.0, 0.0001, 0.5, 9.0, 256, &mut writer).unwrap());
            // Clear the file after each run to avoid accumulating results
            writer.get_mut().set_len(0).unwrap();
        })
    });

    std::fs::remove_file("results/bench_results.txt").unwrap();
}

pub fn bigger_bench(c: &mut Criterion) {
    // Create a file to write the results
    let file = std::fs::File::create("results/bigger_bench.txt").unwrap();
    let mut writer = std::io::BufWriter::new(file);
    c.bench_function("n=256, 200000 computations", |b| {
        b.iter(|| {
            black_box(compute_all(0.0, 1.0, 0.000005, 0.5, 9.0, 256, &mut writer).unwrap());
        })
    });
    std::fs::remove_file("results/bigger_bench.txt").unwrap();
}

// criterion_group!(benches, bench_single, bench_all);
criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    targets = bigger_bench
}
criterion_main!(benches);
