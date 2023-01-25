// Bench files are taken from the large corpus found in
// the Canterbury corpus. https://corpus.canterbury.ac.nz/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

fn bench_compress_and_decompress(c: &mut Criterion, file_name: &str) {
    let mut g = c.benchmark_group(file_name);

    let data = fs::read(format!("benches/{file_name}")).unwrap();
    let mut output = vec![0; lzo1x::worst_compress(data.len())];

    g.bench_function("compress", |b| {
        b.iter(|| {
            black_box(lzo1x::compress_to_slice(&data, &mut output));
        })
    });

    let compressed_data = lzo1x::compress_to_slice(&data, &mut output);
    let mut output = vec![0; data.len()];

    g.bench_function("decompress", |b| {
        b.iter(|| {
            black_box(lzo1x::decompress_to_slice(compressed_data, &mut output).ok());
        })
    });
}

fn bench(c: &mut Criterion) {
    bench_compress_and_decompress(c, "bible.txt");
    bench_compress_and_decompress(c, "E.coli");
    bench_compress_and_decompress(c, "world192.txt");
}

criterion_group!(benches, bench);
criterion_main!(benches);
