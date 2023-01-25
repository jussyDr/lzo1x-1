// Test files are taken from the artificial corpus found in
// the Canterbury corpus. https://corpus.canterbury.ac.nz/

use std::fs;
use std::path::Path;

fn test_roundtrip(file_name: &str) {
    let data = fs::read(Path::new("tests").join(file_name)).unwrap();

    let mut output = vec![0; lzo1x::worst_compress(data.len())];
    let compressed_data = lzo1x::compress_to_slice(&data, &mut output);

    let mut output = vec![0; data.len()];
    let decompressed_data = lzo1x::decompress_to_slice(compressed_data, &mut output).unwrap();

    assert_eq!(decompressed_data, data);
}

#[test]
fn roundtrip_a() {
    test_roundtrip("a.txt");
}

#[test]
fn roundtrip_aaa() {
    test_roundtrip("aaa.txt");
}

#[test]
fn roundtrip_alphabet() {
    test_roundtrip("alphabet.txt");
}

#[test]
fn roundtrip_random() {
    test_roundtrip("random.txt");
}
