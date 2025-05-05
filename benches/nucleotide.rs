use criterion::{criterion_group, criterion_main, Criterion};

use simd_playground::nucleotide;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut buf = [0; 46820];
    rand::fill(&mut buf);
    c.bench_function("nucleotide reference", |b| b.iter(|| nucleotide::count_ref(&buf)));
    c.bench_function("nucleotide asm naive", |b| b.iter(|| nucleotide::count_asm_naive(&buf)));
    c.bench_function("nucleotide std::simd", |b| b.iter(|| nucleotide::count_std_simd(&buf)));
    c.bench_function("nucleotide simd intrinsics", |b| b.iter(|| nucleotide::count_simd_intrinsics(&buf)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
