use iai::black_box;

fn roundtrip() {
    let data = black_box(include_bytes!("bible.txt"));
    let mut output = vec![0; lzo1x::worst_compress(data.len())];
    let compressed_data = lzo1x::compress_to_slice(data, &mut output);
    let mut output = vec![0; data.len()];
    lzo1x::decompress_to_slice(compressed_data, &mut output).unwrap();
}

iai::main!(roundtrip);
