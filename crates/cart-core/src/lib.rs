#![forbid(unsafe_code)]

use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::num::TryFromIntError;

const CART_MAGIC: &[u8; 4] = b"CART";
const TRAC_MAGIC: &[u8; 4] = b"TRAC";
const VERSION_V1: u16 = 1;
const RESERVED_DEFAULT: u64 = 0;
const ARC4_KEY_LEN: usize = 16;
const HEADER_LEN: usize = 4 + 2 + 8 + ARC4_KEY_LEN + 8;
const FOOTER_LEN: usize = 4 + 8 + 8 + 8;
pub const DEFAULT_ARC4_KEY: [u8; ARC4_KEY_LEN] = [
    0x03, 0x01, 0x04, 0x01, 0x05, 0x09, 0x02, 0x06, 0x03, 0x01, 0x04, 0x01, 0x05, 0x09, 0x02, 0x06,
];

/// Result alias used throughout cart-core.
pub type Result<T> = std::result::Result<T, CartError>;

/// Errors produced by cart-core APIs.
#[derive(Debug)]
pub enum CartError {
    Io(io::Error),
    InvalidMagic,
    UnsupportedVersion(u16),
    ReservedMismatch(u64),
    Truncated(&'static str),
    InvalidMetadata(&'static str),
    LengthOverflow(&'static str),
    /// Raised for operations that are still being built out.
    Unimplemented(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MandatoryFooter {
    reserved: u64,
    optional_footer_pos: u64,
    optional_footer_len: u64,
}

impl MandatoryFooter {
    fn read<R: Read>(mut reader: R) -> Result<Self> {
        let mut buf = [0u8; FOOTER_LEN];
        if let Err(err) = reader.read_exact(&mut buf) {
            return if err.kind() == io::ErrorKind::UnexpectedEof {
                Err(CartError::Truncated("MandatoryFooter::read buffer"))
            } else {
                Err(err.into())
            };
        }

        let magic: [u8; 4] = buf[0..4]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryFooter::read magic"))?;
        if magic != *TRAC_MAGIC {
            return Err(CartError::InvalidMagic);
        }

        let reserved_bytes: [u8; 8] = buf[4..12]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryFooter::read reserved"))?;
        let reserved = u64::from_le_bytes(reserved_bytes);
        if reserved != RESERVED_DEFAULT {
            return Err(CartError::ReservedMismatch(reserved));
        }

        let optional_footer_pos_bytes: [u8; 8] = buf[12..20]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryFooter::read footer pos"))?;
        let optional_footer_pos = u64::from_le_bytes(optional_footer_pos_bytes);
        let optional_footer_len_bytes: [u8; 8] = buf[20..28]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryFooter::read footer len"))?;
        let optional_footer_len = u64::from_le_bytes(optional_footer_len_bytes);

        Ok(Self {
            reserved,
            optional_footer_pos,
            optional_footer_len,
        })
    }
}

impl Display for CartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::InvalidMagic => write!(f, "input did not contain the CART magic bytes"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported CART version: {version}")
            }
            Self::ReservedMismatch(reserved) => {
                write!(f, "reserved header field expected 0 but was {reserved}")
            }
            Self::Truncated(section) => write!(f, "truncated {section}"),
            Self::InvalidMetadata(section) => write!(f, "invalid metadata in {section}"),
            Self::LengthOverflow(section) => {
                write!(f, "{section} length does not fit into usize")
            }
            Self::Unimplemented(component) => {
                write!(f, "{component} is not implemented yet")
            }
        }
    }
}

impl Error for CartError {}

impl From<io::Error> for CartError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<TryFromIntError> for CartError {
    fn from(_: TryFromIntError) -> Self {
        Self::LengthOverflow("optional section")
    }
}

/// Mandatory header fields at the front of every `CaRT` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MandatoryHeader {
    pub version: u16,
    pub reserved: u64,
    pub arc4_key: [u8; ARC4_KEY_LEN],
    pub optional_header_len: u64,
}

impl MandatoryHeader {
    fn read<R: Read>(mut reader: R) -> Result<Self> {
        let mut buf = [0u8; HEADER_LEN];
        if let Err(err) = reader.read_exact(&mut buf) {
            return if err.kind() == io::ErrorKind::UnexpectedEof {
                Err(CartError::Truncated("MandatoryHeader::read buffer"))
            } else {
                Err(err.into())
            };
        }

        let magic: [u8; 4] = buf[0..4]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryHeader::read magic"))?;
        if magic != *CART_MAGIC {
            return Err(CartError::InvalidMagic);
        }

        let version_bytes: [u8; 2] = buf[4..6]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryHeader::read version"))?;
        let version = u16::from_le_bytes(version_bytes);
        if version != VERSION_V1 {
            return Err(CartError::UnsupportedVersion(version));
        }

        let reserved_bytes: [u8; 8] = buf[6..14]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryHeader::read reserved"))?;
        let reserved = u64::from_le_bytes(reserved_bytes);
        if reserved != RESERVED_DEFAULT {
            return Err(CartError::ReservedMismatch(reserved));
        }

        let arc4_key: [u8; ARC4_KEY_LEN] = buf[14..30]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryHeader::read arc4 key"))?;

        let optional_header_len_bytes: [u8; 8] = buf[30..38]
            .try_into()
            .map_err(|_| CartError::Truncated("MandatoryHeader::read optional len"))?;
        let optional_header_len = u64::from_le_bytes(optional_header_len_bytes);

        Ok(Self {
            version,
            reserved,
            arc4_key,
            optional_header_len,
        })
    }
}

/// Stub encode result; populated fields arrive with the real implementation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EncodeReport;

/// Stub decode result; populated fields arrive with the real implementation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DecodeReport;

/// Metadata view derived from the mandatory header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataView {
    pub header: MandatoryHeader,
    pub optional_header_json: Option<String>,
    pub optional_footer_json: Option<String>,
}

impl Default for MetadataView {
    fn default() -> Self {
        Self {
            header: MandatoryHeader {
                version: VERSION_V1,
                reserved: RESERVED_DEFAULT,
                arc4_key: [0; ARC4_KEY_LEN],
                optional_header_len: 0,
            },
            optional_header_json: None,
            optional_footer_json: None,
        }
    }
}

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
/// Returns [`CartError::Truncated`] when the stream does not contain a complete
/// mandatory header, or validation errors when the header fields deviate from
/// the `CaRT` v1 specification.
pub fn metadata<R>(input: &mut R) -> Result<MetadataView>
where
    R: Read + Seek,
{
    let header = MandatoryHeader::read(&mut *input)?;
    let optional_header_json = read_optional_section(
        input,
        header.optional_header_len,
        &header.arc4_key,
        "metadata optional header",
    )?;

    let total_len = input.seek(SeekFrom::End(0))?;
    if total_len < FOOTER_LEN as u64 {
        return Err(CartError::Truncated("metadata footer size"));
    }

    let footer_len =
        i64::try_from(FOOTER_LEN).map_err(|_| CartError::LengthOverflow("metadata footer size"))?;
    input.seek(SeekFrom::End(-footer_len))?;
    let footer = MandatoryFooter::read(&mut *input)?;

    if footer.optional_footer_len == 0 {
        return Ok(MetadataView {
            header,
            optional_header_json,
            optional_footer_json: None,
        });
    }

    let footer_start = total_len - FOOTER_LEN as u64;
    if footer.optional_footer_pos + footer.optional_footer_len > footer_start {
        return Err(CartError::Truncated("metadata optional footer bounds"));
    }

    input.seek(SeekFrom::Start(footer.optional_footer_pos))?;
    let optional_footer_json = read_optional_section(
        input,
        footer.optional_footer_len,
        &header.arc4_key,
        "metadata optional footer",
    )?;

    Ok(MetadataView {
        header,
        optional_header_json,
        optional_footer_json,
    })
}

/// Check whether a stream looks like a `CaRT` payload.
///
/// # Errors
///
/// Returns a detection error only when underlying I/O fails for reasons other
/// than short input; short inputs simply yield `Ok(false)`.
pub fn is_cart<R>(input: &mut R) -> Result<bool>
where
    R: Read,
{
    let mut buf = [0u8; HEADER_LEN];
    match input.read_exact(&mut buf) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(false),
        Err(err) => return Err(CartError::Io(err)),
    }

    let magic: [u8; 4] = buf[0..4]
        .try_into()
        .map_err(|_| CartError::Truncated("is_cart magic slice"))?;
    if magic != *CART_MAGIC {
        return Ok(false);
    }

    let version_bytes: [u8; 2] = buf[4..6]
        .try_into()
        .map_err(|_| CartError::Truncated("is_cart version slice"))?;
    let version = u16::from_le_bytes(version_bytes);
    if version != VERSION_V1 {
        return Ok(false);
    }

    let reserved_bytes: [u8; 8] = buf[6..14]
        .try_into()
        .map_err(|_| CartError::Truncated("is_cart reserved slice"))?;
    let reserved = u64::from_le_bytes(reserved_bytes);

    Ok(reserved == RESERVED_DEFAULT)
}

struct Arc4 {
    state: [u8; 256],
    i: u8,
    j: u8,
}

impl Arc4 {
    fn new(key: &[u8]) -> Self {
        assert!(!key.is_empty(), "RC4 key must not be empty");
        let mut state = [0u8; 256];
        for (idx, slot) in state.iter_mut().enumerate() {
            *slot = u8::try_from(idx).expect("index within RC4 state range");
        }

        let mut j: u8 = 0;
        for idx in 0..256 {
            let key_byte = key[idx % key.len()];
            j = j.wrapping_add(state[idx]).wrapping_add(key_byte);
            state.swap(idx, j as usize);
        }

        Self { state, i: 0, j: 0 }
    }

    fn apply_keystream(&mut self, data: &mut [u8]) {
        for byte in data {
            self.i = self.i.wrapping_add(1);
            self.j = self.j.wrapping_add(self.state[self.i as usize]);
            self.state.swap(self.i as usize, self.j as usize);
            let idx = self.state[self.i as usize].wrapping_add(self.state[self.j as usize]);
            let k = self.state[idx as usize];
            *byte ^= k;
        }
    }
}

fn read_optional_section<R>(
    reader: &mut R,
    len: u64,
    key: &[u8; ARC4_KEY_LEN],
    label: &'static str,
) -> Result<Option<String>>
where
    R: Read,
{
    if len == 0 {
        return Ok(None);
    }

    let len_usize = usize::try_from(len)?;
    let mut encrypted = vec![0u8; len_usize];
    reader.read_exact(&mut encrypted)?;

    let mut cipher = Arc4::new(key);
    cipher.apply_keystream(&mut encrypted);

    let decrypted = String::from_utf8(encrypted).map_err(|_| CartError::InvalidMetadata(label))?;
    Ok(Some(decrypted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn new_header_bytes(optional_header_len: u64) -> Vec<u8> {
        let mut data = Vec::with_capacity(HEADER_LEN);
        data.extend_from_slice(CART_MAGIC);
        data.extend_from_slice(&VERSION_V1.to_le_bytes());
        data.extend_from_slice(&RESERVED_DEFAULT.to_le_bytes());
        data.extend_from_slice(&DEFAULT_ARC4_KEY);
        data.extend_from_slice(&optional_header_len.to_le_bytes());
        data
    }

    fn build_cart(optional_header: Option<&str>, optional_footer: Option<&str>) -> Vec<u8> {
        let mut buffer = Vec::new();
        let optional_header_bytes = optional_header.map(|json| encrypt_json(json.as_bytes()));
        let optional_header_len = optional_header_bytes
            .as_ref()
            .map_or(0, |bytes| bytes.len() as u64);

        buffer.extend_from_slice(CART_MAGIC);
        buffer.extend_from_slice(&VERSION_V1.to_le_bytes());
        buffer.extend_from_slice(&RESERVED_DEFAULT.to_le_bytes());
        buffer.extend_from_slice(&DEFAULT_ARC4_KEY);
        buffer.extend_from_slice(&optional_header_len.to_le_bytes());

        if let Some(bytes) = optional_header_bytes {
            buffer.extend_from_slice(&bytes);
        }

        let opt_footer_payload = optional_footer.map(|json| encrypt_json(json.as_bytes()));
        let opt_footer_len = opt_footer_payload
            .as_ref()
            .map_or(0, |bytes| bytes.len() as u64);
        let opt_footer_pos = buffer.len() as u64;

        if let Some(bytes) = opt_footer_payload {
            buffer.extend_from_slice(&bytes);
        }

        buffer.extend_from_slice(TRAC_MAGIC);
        buffer.extend_from_slice(&RESERVED_DEFAULT.to_le_bytes());
        buffer.extend_from_slice(&opt_footer_pos.to_le_bytes());
        buffer.extend_from_slice(&opt_footer_len.to_le_bytes());

        buffer
    }

    fn encrypt_json(plaintext: &[u8]) -> Vec<u8> {
        let mut data = plaintext.to_vec();
        let mut cipher = Arc4::new(&DEFAULT_ARC4_KEY);
        cipher.apply_keystream(&mut data);
        data
    }

    #[test]
    fn encode_is_stubbed() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Cursor::new(Vec::<u8>::new());
        let err = encode(&mut input, &mut output).unwrap_err();
        assert!(matches!(err, CartError::Unimplemented("encode")));
    }

    #[test]
    fn decode_is_stubbed() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Cursor::new(Vec::<u8>::new());
        let err = decode(&mut input, &mut output).unwrap_err();
        assert!(matches!(err, CartError::Unimplemented("decode")));
    }

    #[test]
    fn metadata_reads_mandatory_header() {
        let data = build_cart(None, None);
        let mut cursor = Cursor::new(data);
        let view = metadata(&mut cursor).expect("metadata header parsed");
        assert_eq!(view.header.version, VERSION_V1);
        assert_eq!(view.header.optional_header_len, 0);
        assert_eq!(view.header.arc4_key, DEFAULT_ARC4_KEY);
        assert!(view.optional_header_json.is_none());
        assert!(view.optional_footer_json.is_none());
    }

    #[test]
    fn metadata_rejects_bad_magic() {
        let mut cart = build_cart(None, None);
        cart[0] = b'X';
        let mut cursor = Cursor::new(cart);
        let err = metadata(&mut cursor).unwrap_err();
        assert!(matches!(err, CartError::InvalidMagic));
    }

    #[test]
    fn metadata_reads_optional_sections() {
        let cart = build_cart(Some("{\"name\":\"file.bin\"}"), Some("{\"md5\":\"abc\"}"));
        let mut cursor = Cursor::new(cart);
        let view = metadata(&mut cursor).expect("metadata parsed");
        assert_eq!(
            view.optional_header_json.as_deref(),
            Some("{\"name\":\"file.bin\"}")
        );
        assert_eq!(
            view.optional_footer_json.as_deref(),
            Some("{\"md5\":\"abc\"}")
        );
    }

    #[test]
    fn is_cart_detects_valid_header() {
        let data = new_header_bytes(0);
        let mut cursor = Cursor::new(&data);
        let result = is_cart(&mut cursor).expect("detection succeeded");
        assert!(result);
    }

    #[test]
    fn is_cart_rejects_invalid_inputs() {
        let mut data = new_header_bytes(0);
        data[0] = b'X';
        let mut cursor = Cursor::new(&data);
        let result = is_cart(&mut cursor).expect("detection completed");
        assert!(!result);

        let mut short = Cursor::new(vec![0u8; 3]);
        let result = is_cart(&mut short).expect("short input should return false");
        assert!(!result);
    }
}
