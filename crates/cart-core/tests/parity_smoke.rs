#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::PathBuf;

const EXPECTED_BIN_BYTES: u64 = 100_000;

#[test]
fn seeded_artifact_pair_exists() -> io::Result<()> {
    let data_dir = benchmarks_data_dir();
    let artifact_bin = data_dir.join("artifact.bin");
    let artifact_cart = data_dir.join("artifact.cart");

    assert!(
        artifact_bin.is_file(),
        "Missing seeded fixture: {}",
        artifact_bin.display()
    );
    assert!(
        artifact_cart.is_file(),
        "Missing seeded fixture: {}",
        artifact_cart.display()
    );

    let bin_meta = fs::metadata(&artifact_bin)?;
    let cart_meta = fs::metadata(&artifact_cart)?;

    assert_eq!(
        bin_meta.len(),
        EXPECTED_BIN_BYTES,
        "artifact.bin should contain {EXPECTED_BIN_BYTES} bytes"
    );
    assert!(
        cart_meta.len() > 0,
        "artifact.cart should contain data after seeding"
    );

    Ok(())
}

fn benchmarks_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("benchmarks")
        .join("data")
}
