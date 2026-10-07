//! M6-D2-only adapter over the frozen D1 PinSketch64 lab reference.
//!
//! The D1 reference remains unchanged. This adapter models monotonic transport
//! prefixes by reusing the private D1 lab encoding as an internal construction
//! mechanism. It is not a public or compatibility format.

#[path = "m6d_pinsketch64.rs"]
mod pinsketch64;

pub use pinsketch64::{LabError, PinSketch64Lab};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefixError {
    InvalidCapacity,
    PrefixMismatch,
    Lab(LabError),
}

impl From<LabError> for PrefixError {
    fn from(value: LabError) -> Self {
        Self::Lab(value)
    }
}

pub fn prefix(source: &PinSketch64Lab, capacity: usize) -> Result<PinSketch64Lab, PrefixError> {
    if capacity == 0 || capacity > source.capacity() {
        return Err(PrefixError::InvalidCapacity);
    }

    let mut bytes = source.encode();
    let expected = PinSketch64Lab::encoded_len_for_capacity(capacity)?;
    bytes[8..10].copy_from_slice(&(capacity as u16).to_le_bytes());
    bytes.truncate(expected);

    let decoded = PinSketch64Lab::decode(&bytes)?;
    if decoded.capacity() != capacity
        || decoded.zero_present() != source.zero_present()
        || decoded.odd_syndromes() != &source.odd_syndromes()[..capacity]
    {
        return Err(PrefixError::PrefixMismatch);
    }
    Ok(decoded)
}

/// Extend an already-received prefix from a compatible full source.
///
/// Returns the number of newly appended syndrome words. The zero-presence bit
/// is part of the initial prefix metadata and may not change on extension.
pub fn extend_prefix(
    received: &mut PinSketch64Lab,
    source: &PinSketch64Lab,
    new_capacity: usize,
) -> Result<usize, PrefixError> {
    if new_capacity < received.capacity() || new_capacity > source.capacity() || new_capacity == 0 {
        return Err(PrefixError::InvalidCapacity);
    }

    if received.zero_present() != source.zero_present()
        || received.odd_syndromes() != &source.odd_syndromes()[..received.capacity()]
    {
        return Err(PrefixError::PrefixMismatch);
    }

    let added = new_capacity - received.capacity();
    if added != 0 {
        *received = prefix(source, new_capacity)?;
    }
    Ok(added)
}

pub const fn prefix_payload_bytes(stored_capacity: usize) -> usize {
    1 + stored_capacity * core::mem::size_of::<u64>()
}
