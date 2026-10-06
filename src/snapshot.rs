use core::fmt;

pub(crate) const BACKEND_ENERGY: u8 = 1;
pub(crate) const BACKEND_PARITY: u8 = 2;

const MAGIC: [u8; 8] = *b"DELTAMTR";
const FORMAT_VERSION: u16 = 1;
const HEADER_LEN: usize = 20;
const CHECKSUM_LEN: usize = 4;
const CRC32C_REVERSED_POLYNOMIAL: u32 = 0x82F6_3B78;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    TooShort,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnsupportedFlags(u8),
    BackendMismatch { expected: u8, actual: u8 },
    LengthOverflow,
    LengthMismatch { declared: u64, actual: usize },
    ChecksumMismatch,
    ProvenanceRequired,
    UnsupportedConfig,
    InvalidPayload,
    NonCanonicalPadding,
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => f.write_str("snapshot is shorter than the v1 envelope"),
            Self::InvalidMagic => f.write_str("snapshot magic is invalid"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported snapshot version {version}")
            }
            Self::UnsupportedFlags(flags) => {
                write!(f, "unsupported snapshot flags 0x{flags:02x}")
            }
            Self::BackendMismatch { expected, actual } => {
                write!(
                    f,
                    "snapshot backend mismatch: expected {expected}, got {actual}"
                )
            }
            Self::LengthOverflow => f.write_str("snapshot length cannot be represented"),
            Self::LengthMismatch { declared, actual } => {
                write!(
                    f,
                    "snapshot length mismatch: payload declares {declared} bytes, total input is {actual} bytes"
                )
            }
            Self::ChecksumMismatch => f.write_str("snapshot CRC32C checksum mismatch"),
            Self::ProvenanceRequired => {
                f.write_str("snapshot carries an Energy Proven profile and requires an explicit uniform-row provenance assumption")
            }
            Self::UnsupportedConfig => {
                f.write_str("configuration cannot be represented by snapshot v1")
            }
            Self::InvalidPayload => f.write_str("snapshot payload is invalid"),
            Self::NonCanonicalPadding => f.write_str("snapshot contains non-zero padding bits"),
        }
    }
}

impl std::error::Error for SnapshotError {}

pub(crate) fn encode_envelope(backend: u8, payload: &[u8]) -> Result<Vec<u8>, SnapshotError> {
    let payload_len = u64::try_from(payload.len()).map_err(|_| SnapshotError::LengthOverflow)?;
    let capacity = HEADER_LEN
        .checked_add(payload.len())
        .and_then(|value| value.checked_add(CHECKSUM_LEN))
        .ok_or(SnapshotError::LengthOverflow)?;

    let mut encoded = Vec::with_capacity(capacity);
    encoded.extend_from_slice(&MAGIC);
    encoded.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    encoded.push(backend);
    encoded.push(0);
    encoded.extend_from_slice(&payload_len.to_le_bytes());
    encoded.extend_from_slice(payload);

    let checksum = crc32c(&encoded);
    encoded.extend_from_slice(&checksum.to_le_bytes());
    Ok(encoded)
}

pub(crate) fn decode_envelope(bytes: &[u8], expected_backend: u8) -> Result<&[u8], SnapshotError> {
    if bytes.len() < HEADER_LEN + CHECKSUM_LEN {
        return Err(SnapshotError::TooShort);
    }
    if bytes[..8] != MAGIC {
        return Err(SnapshotError::InvalidMagic);
    }

    let version = u16::from_le_bytes([bytes[8], bytes[9]]);
    if version != FORMAT_VERSION {
        return Err(SnapshotError::UnsupportedVersion(version));
    }

    let backend = bytes[10];
    if backend != expected_backend {
        return Err(SnapshotError::BackendMismatch {
            expected: expected_backend,
            actual: backend,
        });
    }

    let flags = bytes[11];
    if flags != 0 {
        return Err(SnapshotError::UnsupportedFlags(flags));
    }

    let declared = u64::from_le_bytes(bytes[12..20].try_into().expect("fixed-size envelope field"));
    let payload_len = usize::try_from(declared).map_err(|_| SnapshotError::LengthOverflow)?;
    let expected_total = HEADER_LEN
        .checked_add(payload_len)
        .and_then(|value| value.checked_add(CHECKSUM_LEN))
        .ok_or(SnapshotError::LengthOverflow)?;
    if bytes.len() != expected_total {
        return Err(SnapshotError::LengthMismatch {
            declared,
            actual: bytes.len(),
        });
    }

    let checksum_offset = HEADER_LEN + payload_len;
    let expected_checksum = u32::from_le_bytes(
        bytes[checksum_offset..]
            .try_into()
            .expect("checksum length was validated"),
    );
    if crc32c(&bytes[..checksum_offset]) != expected_checksum {
        return Err(SnapshotError::ChecksumMismatch);
    }

    Ok(&bytes[HEADER_LEN..checksum_offset])
}

pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, SnapshotError> {
        let bytes = self.take(1)?;
        Ok(bytes[0])
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, SnapshotError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(
            bytes.try_into().expect("cursor returned four bytes"),
        ))
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64, SnapshotError> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes(
            bytes.try_into().expect("cursor returned eight bytes"),
        ))
    }

    pub(crate) fn read_i64(&mut self) -> Result<i64, SnapshotError> {
        let bytes = self.take(8)?;
        Ok(i64::from_le_bytes(
            bytes.try_into().expect("cursor returned eight bytes"),
        ))
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], SnapshotError> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(SnapshotError::LengthOverflow)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(SnapshotError::InvalidPayload)?;
        self.position = end;
        Ok(bytes)
    }
}

pub(crate) fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn push_i64(bytes: &mut Vec<u8>, value: i64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;

    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (CRC32C_REVERSED_POLYNOMIAL & mask);
        }
    }

    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32c_matches_castagnoli_reference_vector() {
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
    }

    #[test]
    fn envelope_rejects_trailing_bytes() {
        let mut bytes = encode_envelope(BACKEND_ENERGY, b"payload").unwrap();
        bytes.push(0);

        assert!(matches!(
            decode_envelope(&bytes, BACKEND_ENERGY),
            Err(SnapshotError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn envelope_rejects_corruption() {
        let mut bytes = encode_envelope(BACKEND_ENERGY, b"payload").unwrap();
        bytes[HEADER_LEN] ^= 1;

        assert_eq!(
            decode_envelope(&bytes, BACKEND_ENERGY),
            Err(SnapshotError::ChecksumMismatch)
        );
    }
}
