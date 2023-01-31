use iai::black_box;

fn bench_roundtrip(data: &[u8]) {
    let mut output = vec![0; lzo1x::worst_compress(data.len())];
    let compressed_data = lzo1x::compress_to_slice(data, &mut output);
    let mut output = vec![0; data.len()];
    lzo1x::decompress_to_slice(compressed_data, &mut output).unwrap();
}

fn bench_roundtrip_bible() {
    let data = black_box(include_bytes!("bible.txt"));
    bench_roundtrip(data);
}

fn bench_roundtrip_ecoli() {
    let data = black_box(include_bytes!("E.coli"));
    bench_roundtrip(data);
}

fn bench_roundtrip_world() {
    let data = black_box(include_bytes!("world192.txt"));
    bench_roundtrip(data);
}

iai::main!(
    bench_roundtrip_bible,
    bench_roundtrip_ecoli,
    bench_roundtrip_world
);
