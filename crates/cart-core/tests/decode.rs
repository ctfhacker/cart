#![forbid(unsafe_code)]

use cart_core::decode;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

#[test]
fn decode_seeded_artifact_matches_original() -> io::Result<()> {
    let data_dir = benchmarks_data_dir();
    let cart_path = data_dir.join("artifact.cart");
    let bin_path = data_dir.join("artifact.bin");

    let mut input = File::open(cart_path)?;
    let mut output = Vec::new();
    let report = decode(&mut input, &mut output).expect("decode artifact");

    assert!(
        report.decoded_bytes > 0,
        "decoded byte count should be recorded"
    );

    let mut expected = Vec::new();
    File::open(bin_path)?.read_to_end(&mut expected)?;
    assert_eq!(output, expected, "decoded payload must match seed artifact");

    Ok(())
}

fn benchmarks_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("benchmarks")
        .join("data")
}
