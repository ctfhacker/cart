#![forbid(unsafe_code)]

use cart_benches::{DatasetRequest, materialize_dataset, measure_decode, measure_encode};
use cart_core::{CartError, encode, is_cart, metadata};
use std::env;
use std::io::Cursor;
use std::time::{Duration, Instant};

const DEFAULT_ITERS: usize = 5;

#[derive(Clone, Copy)]
enum Profile {
    UnitTests,
    Full,
}

struct Config {
    iters: usize,
    profile: Profile,
}

#[derive(Clone, Copy)]
enum ScenarioAction {
    Pack { dataset: DatasetRequest },
    Unpack { dataset: DatasetRequest },
    Metadata { dataset: DatasetRequest },
    IsCart { dataset: DatasetRequest },
}

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    action: ScenarioAction,
}

#[derive(Debug, Default, Clone)]
struct Summary {
    bytes: Option<u64>,
    mean_ms: f64,
    min_ms: f64,
    max_ms: f64,
    mean_throughput: Option<f64>,
    dataset: Option<String>,
}

#[derive(Debug, Default)]
struct Stats {
    min: f64,
    max: f64,
    mean: f64,
}

#[derive(Debug)]
struct Row {
    name: &'static str,
    summary: Summary,
}

struct RandomSpec {
    label: &'static str,
    size: usize,
    pack_name: &'static str,
    unpack_name: &'static str,
}

const RANDOM_SPECS: &[RandomSpec] = &[
    RandomSpec {
        label: "random_10KB",
        size: 10_000,
        pack_name: "pack_random_10KB",
        unpack_name: "unpack_random_10KB",
    },
    RandomSpec {
        label: "random_100KB",
        size: 100_000,
        pack_name: "pack_random_100KB",
        unpack_name: "unpack_random_100KB",
    },
    RandomSpec {
        label: "random_1MB",
        size: 1_000_000,
        pack_name: "pack_random_1MB",
        unpack_name: "unpack_random_1MB",
    },
    RandomSpec {
        label: "random_10MB",
        size: 10_000_000,
        pack_name: "pack_random_10MB",
        unpack_name: "unpack_random_10MB",
    },
];

const RANDOM_128_SPEC: RandomSpec = RandomSpec {
    label: "random_128MiB",
    size: 128 * 1024 * 1024,
    pack_name: "pack_large_128MiB_random",
    unpack_name: "unpack_large_128MiB_random",
};

fn main() {
    let config = parse_config();
    if let Err(err) = run(&config) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(config: &Config) -> Result<(), CartError> {
    let scenarios = scenarios(config.profile);

    println!("cart-rs benchmark runner (iters: {})\n", config.iters);

    let mut rows = Vec::with_capacity(scenarios.len());
    for scenario in scenarios {
        let row = match scenario.action {
            ScenarioAction::Pack { dataset } => run_pack(scenario.name, dataset, config.iters)?,
            ScenarioAction::Unpack { dataset } => run_unpack(scenario.name, dataset, config.iters)?,
            ScenarioAction::Metadata { dataset } => {
                run_metadata(scenario.name, dataset, config.iters)?
            }
            ScenarioAction::IsCart { dataset } => {
                run_is_cart(scenario.name, dataset, config.iters)?
            }
        };
        rows.push(row);
    }

    print_table(&rows);
    Ok(())
}

fn run_pack(
    name: &'static str,
    dataset_req: DatasetRequest,
    iters: usize,
) -> Result<Row, CartError> {
    let dataset = materialize_dataset(dataset_req);
    let dataset_path = dataset.path.clone();
    let payload = dataset.bytes;
    let bytes = payload.len() as u64;

    let mut durations = Vec::with_capacity(iters);
    let mut throughputs = Vec::with_capacity(iters);

    for _ in 0..iters {
        let outcome = measure_encode(payload.as_slice())?;
        durations.push(outcome.stats.duration);
        throughputs.push(outcome.stats.throughput_mib_per_sec());
    }

    Ok(Row {
        name,
        summary: summarise(&durations, &throughputs, bytes, dataset_path),
    })
}

fn run_unpack(
    name: &'static str,
    dataset_req: DatasetRequest,
    iters: usize,
) -> Result<Row, CartError> {
    let dataset = materialize_dataset(dataset_req);
    let dataset_path = dataset.path.clone();
    let payload = dataset.bytes;
    let bytes = payload.len() as u64;
    let encoded = encode_once(payload.as_slice())?;

    let mut durations = Vec::with_capacity(iters);
    let mut throughputs = Vec::with_capacity(iters);

    for _ in 0..iters {
        let outcome = measure_decode(&encoded)?;
        durations.push(outcome.stats.duration);
        throughputs.push(outcome.stats.throughput_mib_per_sec());
    }

    Ok(Row {
        name,
        summary: summarise(&durations, &throughputs, bytes, dataset_path),
    })
}

fn run_metadata(
    name: &'static str,
    dataset_req: DatasetRequest,
    iters: usize,
) -> Result<Row, CartError> {
    let dataset = materialize_dataset(dataset_req);
    let encoded = encode_once(dataset.bytes.as_slice())?;

    let mut durations = Vec::with_capacity(iters);
    for _ in 0..iters {
        let mut cursor = Cursor::new(encoded.as_slice());
        let start = Instant::now();
        metadata(&mut cursor)?;
        durations.push(start.elapsed());
    }

    Ok(Row {
        name,
        summary: summarise_no_throughput(&durations, None),
    })
}

fn run_is_cart(
    name: &'static str,
    dataset_req: DatasetRequest,
    iters: usize,
) -> Result<Row, CartError> {
    let dataset = materialize_dataset(dataset_req);
    let encoded = encode_once(dataset.bytes.as_slice())?;

    let mut durations = Vec::with_capacity(iters);
    for _ in 0..iters {
        let mut cursor = Cursor::new(encoded.as_slice());
        let start = Instant::now();
        let detected = is_cart(&mut cursor)?;
        debug_assert!(detected, "encoded dataset should be a CaRT archive");
        durations.push(start.elapsed());
    }

    Ok(Row {
        name,
        summary: summarise_no_throughput(&durations, None),
    })
}

fn summarise(
    durations: &[Duration],
    throughputs: &[f64],
    bytes: u64,
    dataset: Option<String>,
) -> Summary {
    let stats = stats_from_durations(durations);
    let mean_throughput = mean(throughputs);
    Summary {
        bytes: Some(bytes),
        mean_ms: stats.mean * 1_000.0,
        min_ms: stats.min * 1_000.0,
        max_ms: stats.max * 1_000.0,
        mean_throughput: Some(mean_throughput),
        dataset,
    }
}

fn summarise_no_throughput(durations: &[Duration], dataset: Option<String>) -> Summary {
    let stats = stats_from_durations(durations);
    Summary {
        bytes: None,
        mean_ms: stats.mean * 1_000.0,
        min_ms: stats.min * 1_000.0,
        max_ms: stats.max * 1_000.0,
        mean_throughput: None,
        dataset,
    }
}

fn stats_from_durations(durations: &[Duration]) -> Stats {
    if durations.is_empty() {
        return Stats::default();
    }
    let mut min = f64::MAX;
    let mut max: f64 = 0.0;
    let mut sum = 0.0;

    for duration in durations {
        let secs = duration.as_secs_f64();
        min = min.min(secs);
        max = max.max(secs);
        sum += secs;
    }

    #[allow(clippy::cast_precision_loss)]
    let mean = sum / durations.len() as f64;

    Stats { min, max, mean }
}

fn print_table(rows: &[Row]) {
    let headers = vec![
        "Scenario".to_string(),
        "Bytes In".to_string(),
        "Mean Time (ms)".to_string(),
        "Min Time (ms)".to_string(),
        "Max Time (ms)".to_string(),
        "Mean Throughput (MiB/s)".to_string(),
        "Dataset".to_string(),
    ];

    let mut widths: Vec<usize> = headers.iter().map(String::len).collect();
    let row_cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            let Summary {
                bytes,
                mean_ms,
                min_ms,
                max_ms,
                mean_throughput,
                dataset,
            } = &row.summary;
            vec![
                row.name.to_string(),
                format_bytes_opt(*bytes),
                format!("{mean_ms:.3}"),
                format!("{min_ms:.3}"),
                format!("{max_ms:.3}"),
                format_throughput_opt(*mean_throughput),
                dataset.as_deref().unwrap_or("-").to_string(),
            ]
        })
        .collect();

    for cells in &row_cells {
        for (idx, cell) in cells.iter().enumerate() {
            widths[idx] = widths[idx].max(cell.len());
        }
    }

    println!("{}", format_row(&headers, &widths));
    println!("{}", separator(&widths));
    for cells in row_cells {
        println!("{}", format_row(&cells, &widths));
    }
}

fn format_row(cells: &[String], widths: &[usize]) -> String {
    cells
        .iter()
        .enumerate()
        .map(|(idx, cell)| format!("{:width$}", cell, width = widths[idx]))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn separator(widths: &[usize]) -> String {
    widths
        .iter()
        .map(|w| "-".repeat(*w))
        .collect::<Vec<_>>()
        .join("-+-")
}

fn parse_config() -> Config {
    let mut args = env::args().skip(1);
    let mut iters = DEFAULT_ITERS;
    let mut profile = Profile::UnitTests;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--iters" => {
                if let Some(value) = args.next()
                    && let Ok(parsed) = value.parse::<usize>()
                {
                    iters = parsed;
                }
            }
            "--profile" => {
                if let Some(value) = args.next() {
                    profile = match value.as_str() {
                        "unittests" => Profile::UnitTests,
                        "full" => Profile::Full,
                        _ => {
                            eprintln!("unknown profile '{value}', expected 'unittests' or 'full'");
                            std::process::exit(2);
                        }
                    };
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
    }

    Config {
        iters: iters.max(1),
        profile,
    }
}

fn print_help() {
    println!(
        "Usage: cargo run -p cart-benches --bin run [-- --iters N --profile <unittests|full>]\n"
    );
    println!("Options:");
    println!("  --iters N     Number of iterations per scenario (default {DEFAULT_ITERS})");
    println!("  --profile P   Scenario set: 'unittests' (default) or 'full'");
}

#[allow(clippy::cast_precision_loss)]
fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn format_bytes_opt(bytes: Option<u64>) -> String {
    match bytes {
        Some(value) => format_bytes(value),
        None => "-".to_string(),
    }
}

fn format_throughput_opt(throughput: Option<f64>) -> String {
    throughput.map_or_else(|| "-".to_string(), |value| format!("{value:.3}"))
}

#[allow(clippy::cast_precision_loss)]
fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;

    match bytes {
        0 => "0 B".to_string(),
        b if (b as f64) >= MIB => format!("{:.2} MiB", (b as f64) / MIB),
        b if (b as f64) >= KIB => format!("{:.1} KiB", (b as f64) / KIB),
        b => format!("{b} B"),
    }
}

fn unit_test_scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "pack_empty",
            action: ScenarioAction::Pack {
                dataset: DatasetRequest::Empty,
            },
        },
        Scenario {
            name: "unpack_empty",
            action: ScenarioAction::Unpack {
                dataset: DatasetRequest::Empty,
            },
        },
        Scenario {
            name: "pack_small_1B",
            action: ScenarioAction::Pack {
                dataset: DatasetRequest::Literal {
                    label: "small_1B",
                    bytes: b"a",
                },
            },
        },
        Scenario {
            name: "unpack_small_1B",
            action: ScenarioAction::Unpack {
                dataset: DatasetRequest::Literal {
                    label: "small_1B",
                    bytes: b"a",
                },
            },
        },
        Scenario {
            name: "pack_large_128MiB_zeros",
            action: ScenarioAction::Pack {
                dataset: DatasetRequest::Zeros {
                    label: "zeros_128MiB",
                    size: 128 * 1024 * 1024,
                },
            },
        },
        Scenario {
            name: "unpack_large_128MiB_zeros",
            action: ScenarioAction::Unpack {
                dataset: DatasetRequest::Zeros {
                    label: "zeros_128MiB",
                    size: 128 * 1024 * 1024,
                },
            },
        },
        Scenario {
            name: "unpack_simple_pattern_100KB",
            action: ScenarioAction::Unpack {
                dataset: DatasetRequest::Pattern {
                    label: "pattern_100KB",
                    size: 100_000,
                },
            },
        },
        Scenario {
            name: "metadata_only",
            action: ScenarioAction::Metadata {
                dataset: DatasetRequest::Pattern {
                    label: "pattern_100KB",
                    size: 100_000,
                },
            },
        },
        Scenario {
            name: "is_cart",
            action: ScenarioAction::IsCart {
                dataset: DatasetRequest::Pattern {
                    label: "pattern_100KB",
                    size: 100_000,
                },
            },
        },
    ]
}

fn random_scenarios() -> Vec<Scenario> {
    RANDOM_SPECS
        .iter()
        .flat_map(|spec| {
            [
                Scenario {
                    name: spec.pack_name,
                    action: ScenarioAction::Pack {
                        dataset: DatasetRequest::Random {
                            label: spec.label,
                            size: spec.size,
                        },
                    },
                },
                Scenario {
                    name: spec.unpack_name,
                    action: ScenarioAction::Unpack {
                        dataset: DatasetRequest::Random {
                            label: spec.label,
                            size: spec.size,
                        },
                    },
                },
            ]
        })
        .collect()
}

fn extended_random_scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: RANDOM_128_SPEC.pack_name,
            action: ScenarioAction::Pack {
                dataset: DatasetRequest::Random {
                    label: RANDOM_128_SPEC.label,
                    size: RANDOM_128_SPEC.size,
                },
            },
        },
        Scenario {
            name: RANDOM_128_SPEC.unpack_name,
            action: ScenarioAction::Unpack {
                dataset: DatasetRequest::Random {
                    label: RANDOM_128_SPEC.label,
                    size: RANDOM_128_SPEC.size,
                },
            },
        },
    ]
}

fn scenarios(profile: Profile) -> Vec<Scenario> {
    let mut all = unit_test_scenarios();
    all.extend(random_scenarios());
    if matches!(profile, Profile::Full) {
        all.extend(extended_random_scenarios());
    }
    all
}

fn encode_once(payload: &[u8]) -> Result<Vec<u8>, CartError> {
    let mut input = Cursor::new(payload);
    let mut output = Vec::with_capacity(payload.len());
    encode(&mut input, &mut output)?;
    Ok(output)
}
