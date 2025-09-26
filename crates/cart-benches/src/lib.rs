#![forbid(unsafe_code)]

use cart_core::{CartError, Result};

/// Placeholder benchmark harness entry.
///
/// # Errors
///
/// Always returns [`CartError::Unimplemented`] until benchmarks are wired.
pub fn run_pending() -> Result<()> {
    Err(CartError::Unimplemented("bench harness"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_harness_is_stubbed() {
        let err = run_pending().unwrap_err();
        assert!(matches!(err, CartError::Unimplemented("bench harness")));
    }
}
