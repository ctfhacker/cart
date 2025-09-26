#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match cart_cli::run(std::env::args_os()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let mut stderr = io::stderr().lock();
            if writeln!(stderr, "{err}").is_err() {
                // stderr is unavailable; nothing sensible to do.
            }
            ExitCode::FAILURE
        }
    }
}
