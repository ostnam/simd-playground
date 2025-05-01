use criterion::{criterion_group, criterion_main, Criterion};

use simd_playground::byte_count;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut buf = [0; 2048];
    rand::fill(&mut buf);
    c.bench_function("bytes_count reference 2048", |b| b.iter(|| byte_count::count_bytes_ref(&buf, 128)));
    c.bench_function("bytes_count_x86 2048", |b| b.iter(|| byte_count::count_bytes_x86(&buf, 128)));
    c.bench_function("bytes_count_x86_jne 2048", |b| b.iter(|| byte_count::count_bytes_x86_jne(&buf, 128)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
