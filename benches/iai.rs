use iai::black_box;

fn roundtrip(data: &[u8]) {
    let mut output = vec![0; lzo1x::worst_compress(data.len())];
    let compressed_data = lzo1x::compress_to_slice(data, &mut output);
    let mut output = vec![0; data.len()];
    lzo1x::decompress_to_slice(compressed_data, &mut output).unwrap();
}

fn bible() {
    let data = black_box(include_bytes!("bible.txt"));
    roundtrip(data);
}

fn ecoli() {
    let data = black_box(include_bytes!("E.coli"));
    roundtrip(data);
}

fn world() {
    let data = black_box(include_bytes!("world192.txt"));
    roundtrip(data);
}

iai::main!(bible, ecoli, world);
