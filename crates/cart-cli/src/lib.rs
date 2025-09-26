#![forbid(unsafe_code)]

use cart_core::{CartError, Result};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};

/// Stub command dispatcher; real subcommands wire in later.
///
/// # Errors
///
/// Returns [`CartError::Unimplemented`] for each placeholder command until wiring lands.
pub fn run<I>(args: I) -> Result<()>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let program = args.next().unwrap_or_else(|| OsString::from("cart-cli"));

    match args.next() {
        Some(cmd) if cmd == OsStr::new("encode") => Err(CartError::Unimplemented("cli encode")),
        Some(cmd) if cmd == OsStr::new("decode") => Err(CartError::Unimplemented("cli decode")),
        Some(cmd) if cmd == OsStr::new("inspect") => Err(CartError::Unimplemented("cli inspect")),
        Some(cmd) if cmd == OsStr::new("verify") => Err(CartError::Unimplemented("cli verify")),
        Some(cmd) if cmd == OsStr::new("--help") || cmd == OsStr::new("-h") => {
            print_usage(program.as_os_str());
            Ok(())
        }
        None => {
            print_usage(program.as_os_str());
            Ok(())
        }
        _ => Err(CartError::Unimplemented("cli entrypoint")),
    }
}

fn print_usage(program: &OsStr) {
    let mut stdout = io::stdout().lock();
    let _ = writeln!(
        stdout,
        "Usage: {} <command> [options]\nCommands: encode | decode | inspect | verify",
        program.to_string_lossy(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_is_stubbed() {
        let args = vec![OsString::from("cart-cli"), OsString::from("encode")];
        let err = run(args).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("cli encode"));
    }

    #[test]
    fn decode_is_stubbed() {
        let args = vec![OsString::from("cart-cli"), OsString::from("decode")];
        let err = run(args).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("cli decode"));
    }

    #[test]
    fn inspect_is_stubbed() {
        let args = vec![OsString::from("cart-cli"), OsString::from("inspect")];
        let err = run(args).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("cli inspect"));
    }

    #[test]
    fn verify_is_stubbed() {
        let args = vec![OsString::from("cart-cli"), OsString::from("verify")];
        let err = run(args).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("cli verify"));
    }

    #[test]
    fn help_prints_usage_and_succeeds() {
        let args = vec![OsString::from("cart-cli"), OsString::from("--help")];
        assert!(run(args).is_ok());
    }

    #[test]
    fn no_args_prints_usage_and_succeeds() {
        let args = vec![OsString::from("cart-cli")];
        assert!(run(args).is_ok());
    }
}
