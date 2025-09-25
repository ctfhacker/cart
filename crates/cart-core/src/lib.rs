#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{self, Display};
use std::io::{Read, Seek, Write};

/// Result alias used throughout cart-core.
pub type Result<T> = std::result::Result<T, CartError>;

/// Placeholder error enum until real implementations land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartError {
    Unimplemented(&'static str),
}

impl Display for CartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unimplemented(component) => {
                write!(f, "{component} is not implemented yet")
            }
        }
    }
}

impl Error for CartError {}

/// Stub encode result; populated fields arrive with the real implementation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EncodeReport;

/// Stub decode result; populated fields arrive with the real implementation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DecodeReport;

/// Stub metadata view; will expose header/footer fields once wired.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MetadataView;

/// Encode a `CaRT` archive once implemented.
///
/// # Errors
///
/// Always returns [`CartError::Unimplemented`] until the encoder is wired.
pub fn encode<R, W>(_input: &mut R, _output: &mut W) -> Result<EncodeReport>
where
    R: Read,
    W: Write,
{
    Err(CartError::Unimplemented("encode"))
}

/// Decode a `CaRT` archive once implemented.
///
/// # Errors
///
/// Always returns [`CartError::Unimplemented`] until the decoder is wired.
pub fn decode<R, W>(_input: &mut R, _output: &mut W) -> Result<DecodeReport>
where
    R: Read,
    W: Write,
{
    Err(CartError::Unimplemented("decode"))
}

/// Peek metadata without full decode.
///
/// # Errors
///
/// Always returns [`CartError::Unimplemented`] until metadata parsing exists.
pub fn metadata<R>(_input: &mut R) -> Result<MetadataView>
where
    R: Read + Seek,
{
    Err(CartError::Unimplemented("metadata"))
}

/// Check whether a stream looks like a `CaRT` payload.
///
/// # Errors
///
/// Always returns [`CartError::Unimplemented`] until detection is wired.
pub fn is_cart<R>(_input: &mut R) -> Result<bool>
where
    R: Read,
{
    Err(CartError::Unimplemented("is_cart"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encode_is_stubbed() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Cursor::new(Vec::<u8>::new());
        let err = encode(&mut input, &mut output).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("encode"));
    }

    #[test]
    fn decode_is_stubbed() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Cursor::new(Vec::<u8>::new());
        let err = decode(&mut input, &mut output).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("decode"));
    }

    #[test]
    fn metadata_is_stubbed() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let err = metadata(&mut input).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("metadata"));
    }

    #[test]
    fn is_cart_is_stubbed() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let err = is_cart(&mut input).unwrap_err();
        assert_eq!(err, CartError::Unimplemented("is_cart"));
    }
}
