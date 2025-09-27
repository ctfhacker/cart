#![forbid(unsafe_code)]

use cart_benches::{DatasetRequest, materialize_dataset};
use cart_core::{decode, encode};
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use std::io::Cursor;

fn encode_benchmarks(c: &mut Criterion) {
    let dataset = materialize_dataset(DatasetRequest::Random {
        label: "random_100KB",
        size: 100_000,
    });
    let payload = dataset.bytes;

    let mut group = c.benchmark_group("pack");
    group.throughput(Throughput::Bytes(payload.len() as u64));

    group.bench_function("pack_random_100KB", |b| {
        b.iter_batched(
            || Vec::with_capacity(payload.len()),
            |mut output| {
                output.clear();
                let mut input = Cursor::new(payload.as_slice());
                encode(&mut input, &mut output).expect("encode payload");
                criterion::black_box(output);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn decode_benchmarks(c: &mut Criterion) {
    let dataset = materialize_dataset(DatasetRequest::Random {
        label: "random_100KB",
        size: 100_000,
    });
    let payload = dataset.bytes;
    let encoded = encode_once(&payload);

    let mut group = c.benchmark_group("unpack");
    group.throughput(Throughput::Bytes(payload.len() as u64));

    group.bench_function("unpack_random_100KB", |b| {
        b.iter_batched(
            || Vec::with_capacity(payload.len()),
            |mut output| {
                output.clear();
                let mut input = Cursor::new(encoded.as_slice());
                decode(&mut input, &mut output).expect("decode payload");
                criterion::black_box(output);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn encode_once(payload: &[u8]) -> Vec<u8> {
    let mut input = Cursor::new(payload);
    let mut output = Vec::with_capacity(payload.len());
    encode(&mut input, &mut output).expect("encode payload");
    output
}

criterion_group!(codec, encode_benchmarks, decode_benchmarks);
criterion_main!(codec);
