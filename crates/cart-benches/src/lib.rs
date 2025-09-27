#![forbid(unsafe_code)]

use cart_core::{DecodeReport, EncodeReport, Result, decode, encode};
use std::convert::TryFrom;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, Instant};

const DATA_DIR: &str = "benchmarks/data";

/// Dataset materialized for benchmarking.
#[derive(Debug)]
pub struct Dataset {
    pub bytes: Vec<u8>,
    pub path: Option<String>,
}

/// Specification for datasets shared between Rust and Python benchmarks.
#[derive(Clone, Copy, Debug)]
pub enum DatasetRequest {
    Empty,
    Literal {
        label: &'static str,
        bytes: &'static [u8],
    },
    Pattern {
        label: &'static str,
        size: usize,
    },
    Random {
        label: &'static str,
        size: usize,
    },
    Zeros {
        label: &'static str,
        size: usize,
    },
}

/// Materialise a dataset, writing it to `benchmarks/data/<label>.bin` when appropriate.
#[must_use]
pub fn materialize_dataset(request: DatasetRequest) -> Dataset {
    match request {
        DatasetRequest::Empty => Dataset {
            bytes: Vec::new(),
            path: None,
        },
        DatasetRequest::Literal { label, bytes } => {
            dataset_from_generator(label, bytes.len(), || bytes.to_vec())
        }
        DatasetRequest::Pattern { label, size } => {
            dataset_from_generator(label, size, || generate_pattern_bytes(size))
        }
        DatasetRequest::Random { label, size } => {
            dataset_from_generator(label, size, || generate_random_bytes(size))
        }
        DatasetRequest::Zeros { label, size } => {
            dataset_from_generator(label, size, || vec![b'0'; size])
        }
    }
}

fn dataset_from_generator(
    label: &str,
    expected_len: usize,
    generator: impl FnOnce() -> Vec<u8>,
) -> Dataset {
    let dir = Path::new(DATA_DIR);
    fs::create_dir_all(dir).expect("failed to create benchmarks/data directory");
    let path = dir.join(format!("{label}.bin"));
    let path_string = path.to_string_lossy().into_owned();

    let expected_len_u64 = u64::try_from(expected_len).expect("dataset length exceeds u64");
    let needs_regen = match fs::metadata(&path) {
        Ok(meta) => meta.len() != expected_len_u64,
        Err(_) => true,
    };

    let bytes = if needs_regen {
        let data = generator();
        assert_eq!(
            data.len(),
            expected_len,
            "generated dataset length for {label} did not match expectation"
        );
        fs::write(&path, &data).expect("failed to write dataset");
        data
    } else {
        match fs::read(&path) {
            Ok(data) if data.len() == expected_len => data,
            _ => {
                let data = generator();
                assert_eq!(
                    data.len(),
                    expected_len,
                    "generated dataset length for {label} did not match expectation"
                );
                fs::write(&path, &data).expect("failed to write dataset");
                data
            }
        }
    };

    Dataset {
        bytes,
        path: Some(path_string),
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size);
    let mut x: u32 = 42;
    for _ in 0..size {
        x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let byte = u8::try_from((x >> 16) & 0xFF).expect("random byte fits into u8");
        buf.push(byte);
    }
    buf
}

fn generate_pattern_bytes(size: usize) -> Vec<u8> {
    const PATTERN: &[u8] = b"0123456789";
    let mut buf = Vec::with_capacity(size);
    while buf.len() < size {
        let remaining = size - buf.len();
        let chunk = &PATTERN[..remaining.min(PATTERN.len())];
        buf.extend_from_slice(chunk);
    }
    buf
}

/// Per-operation timing details that make throughput comparisons straightforward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OperationStats {
    /// Number of logical bytes processed by the operation.
    pub bytes_processed: u64,
    /// Wall-clock duration of the operation.
    pub duration: Duration,
}

impl OperationStats {
    /// Compute throughput in mebibytes per second. Returns `f64::INFINITY` when the
    /// duration rounds to zero on very fast operations.
    #[must_use]
    pub fn throughput_mib_per_sec(&self) -> f64 {
        let elapsed = self.duration.as_secs_f64();
        if elapsed == 0.0 {
            f64::INFINITY
        } else {
            const BYTES_PER_MEBI: f64 = 1024.0 * 1024.0;
            let bytes = {
                #[allow(clippy::cast_precision_loss)]
                {
                    self.bytes_processed as f64
                }
            };
            bytes / (BYTES_PER_MEBI * elapsed)
        }
    }
}

/// Outcome of an encode measurement, including the encoded payload for downstream comparisons.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodeOutcome {
    pub report: EncodeReport,
    pub stats: OperationStats,
    pub output: Vec<u8>,
}

/// Outcome of a decode measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodeOutcome {
    pub report: DecodeReport,
    pub stats: OperationStats,
    pub output: Vec<u8>,
}

/// Encode a payload using `cart-core::encode`, returning the encoded bytes alongside timing data.
///
/// # Errors
///
/// Returns any [`CartError`](cart_core::CartError) raised by `encode` while processing the payload.
pub fn measure_encode(payload: &[u8]) -> Result<EncodeOutcome> {
    let mut input = Cursor::new(payload);
    let mut output = Vec::with_capacity(payload.len());
    let start = Instant::now();
    let report = encode(&mut input, &mut output)?;
    let duration = start.elapsed();

    let bytes_processed = report.input_bytes;
    Ok(EncodeOutcome {
        report,
        stats: OperationStats {
            bytes_processed,
            duration,
        },
        output,
    })
}

/// Decode an encoded `CaRT` payload, returning timing data and the recovered plaintext.
///
/// # Errors
///
/// Returns any [`CartError`](cart_core::CartError) raised by `decode` while processing the payload.
pub fn measure_decode(encoded_payload: &[u8]) -> Result<DecodeOutcome> {
    let mut input = Cursor::new(encoded_payload);
    let mut output = Vec::new();
    let start = Instant::now();
    let report = decode(&mut input, &mut output)?;
    let duration = start.elapsed();
    let bytes_processed = report.decoded_bytes;

    Ok(DecodeOutcome {
        report,
        stats: OperationStats {
            bytes_processed,
            duration,
        },
        output,
    })
}

/// Convenience helper that encodes and decodes a payload, mirroring the Python baseline workflow.
///
/// # Errors
///
/// Propagates failures from [`measure_encode`] or [`measure_decode`].
pub fn measure_round_trip(payload: &[u8]) -> Result<(EncodeOutcome, DecodeOutcome)> {
    let encode_outcome = measure_encode(payload)?;
    let decode_outcome = measure_decode(&encode_outcome.output)?;
    Ok((encode_outcome, decode_outcome))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn encode_measurement_tracks_bytes() {
        let payload = b"benchmark-payload";
        let outcome = measure_encode(payload).expect("encode outcome");
        assert_eq!(
            outcome.report.input_bytes,
            u64::try_from(payload.len()).unwrap()
        );
        assert_eq!(outcome.stats.bytes_processed, outcome.report.input_bytes);
        assert!(!outcome.output.is_empty());
    }

    #[test]
    fn round_trip_matches_original_payload() {
        let payload = b"rust cart benchmark".repeat(16);
        let (encode_outcome, decode_outcome) =
            measure_round_trip(&payload).expect("round trip outcome");

        assert_eq!(
            decode_outcome.report.decoded_bytes,
            u64::try_from(payload.len()).unwrap()
        );
        assert_eq!(decode_outcome.output, payload);

        // Throughput is either finite or effectively instantaneous on very fast hosts.
        assert!(
            encode_outcome.stats.throughput_mib_per_sec().is_finite()
                || encode_outcome.stats.throughput_mib_per_sec().is_infinite()
        );
    }
}
