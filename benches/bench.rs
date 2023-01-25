// Bench file is taken from the large corpus found in
// the Canterbury corpus. https://corpus.canterbury.ac.nz/

use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench(c: &mut Criterion) {
    let data = fs::read("benches/world192.txt").unwrap();
    let mut output = vec![0; lzo1x::worst_compress(data.len())];

    c.bench_function("compress", |b| {
        b.iter(|| {
            black_box(lzo1x::compress_to_slice(&data, &mut output));
        })
    });

    let compressed_data = lzo1x::compress_to_slice(&data, &mut output);
    let mut output = vec![0; data.len()];

    c.bench_function("decompress", |b| {
        b.iter(|| {
            black_box(lzo1x::decompress_to_slice(compressed_data, &mut output).ok());
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
