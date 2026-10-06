//! Private M6-D1 PinSketch64-style lab reference.
//!
//! This module is shared by examples/tests only. It is not a supported
//! DeltaMeter API and its byte encoding is not snapshot-v1.

use std::fmt;

const GF64_REDUCTION: u64 = 0x1B;
const LAB_MAGIC: [u8; 8] = *b"DMP64L01";
const LAB_HEADER_LEN: usize = 16;
pub const MAX_LAB_CAPACITY: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabError {
    InvalidCapacity,
    InvalidDecodeLimit,
    CapacityMismatch,
    InvalidEncoding,
    DuplicateInput,
    DecodeFailure,
    CandidateExceedsCapacity,
    InvalidRoots,
}

impl fmt::Display for LabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCapacity => f.write_str("lab stored capacity must be in 1..=9"),
            Self::InvalidDecodeLimit => {
                f.write_str("decode limit must not exceed stored syndrome capacity")
            }
            Self::CapacityMismatch => f.write_str("lab sketch capacities differ"),
            Self::InvalidEncoding => f.write_str("invalid PinSketch64 lab encoding"),
            Self::DuplicateInput => f.write_str("input must be a strictly increasing unique set"),
            Self::DecodeFailure => f.write_str("syndrome decoder failed"),
            Self::CandidateExceedsCapacity => {
                f.write_str("decoded candidate exceeds configured total capacity")
            }
            Self::InvalidRoots => {
                f.write_str("locator polynomial did not yield valid distinct roots")
            }
        }
    }
}

impl std::error::Error for LabError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinSketch64Lab {
    capacity: usize,
    odd_syndromes: Box<[u64]>,
    zero_present: bool,
}

impl PinSketch64Lab {
    pub fn new(capacity: usize) -> Result<Self, LabError> {
        if !(1..=MAX_LAB_CAPACITY).contains(&capacity) {
            return Err(LabError::InvalidCapacity);
        }
        Ok(Self {
            capacity,
            odd_syndromes: vec![0; capacity].into_boxed_slice(),
            zero_present: false,
        })
    }

    pub fn from_sorted_unique(capacity: usize, keys: &[u64]) -> Result<Self, LabError> {
        if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(LabError::DuplicateInput);
        }
        let mut sketch = Self::new(capacity)?;
        for &key in keys {
            sketch.toggle(key);
        }
        Ok(sketch)
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub const fn zero_present(&self) -> bool {
        self.zero_present
    }

    pub fn odd_syndromes(&self) -> &[u64] {
        &self.odd_syndromes
    }

    pub fn toggle(&mut self, key: u64) {
        if key == 0 {
            self.zero_present ^= true;
            return;
        }

        let squared = gf64_square(key);
        let mut odd_power = key;
        for syndrome in &mut self.odd_syndromes {
            *syndrome ^= odd_power;
            odd_power = gf64_mul(odd_power, squared);
        }
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), LabError> {
        if self.capacity != other.capacity {
            return Err(LabError::CapacityMismatch);
        }
        for (left, right) in self
            .odd_syndromes
            .iter_mut()
            .zip(other.odd_syndromes.iter())
        {
            *left ^= *right;
        }
        self.zero_present ^= other.zero_present;
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(LAB_HEADER_LEN + self.capacity * 8);
        bytes.extend_from_slice(&LAB_MAGIC);
        bytes.extend_from_slice(&(self.capacity as u16).to_le_bytes());
        bytes.push(u8::from(self.zero_present));
        bytes.extend_from_slice(&[0; 5]);
        for &value in &self.odd_syndromes {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < LAB_HEADER_LEN || bytes[..8] != LAB_MAGIC {
            return Err(LabError::InvalidEncoding);
        }

        let capacity = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
        if !(1..=MAX_LAB_CAPACITY).contains(&capacity) {
            return Err(LabError::InvalidCapacity);
        }

        let zero_present = match bytes[10] {
            0 => false,
            1 => true,
            _ => return Err(LabError::InvalidEncoding),
        };
        if bytes[11..16] != [0; 5] {
            return Err(LabError::InvalidEncoding);
        }

        let expected = LAB_HEADER_LEN
            .checked_add(capacity.checked_mul(8).ok_or(LabError::InvalidEncoding)?)
            .ok_or(LabError::InvalidEncoding)?;
        if bytes.len() != expected {
            return Err(LabError::InvalidEncoding);
        }

        let mut odd_syndromes = Vec::with_capacity(capacity);
        let (chunks, remainder) = bytes[LAB_HEADER_LEN..].as_chunks::<8>();
        if !remainder.is_empty() {
            return Err(LabError::InvalidEncoding);
        }
        for chunk in chunks {
            odd_syndromes.push(u64::from_le_bytes(*chunk));
        }

        Ok(Self {
            capacity,
            odd_syndromes: odd_syndromes.into_boxed_slice(),
            zero_present,
        })
    }

    /// Decode at most `max_elements` while using any additional stored
    /// syndromes as algebraic consistency guards.
    ///
    /// The guard is not independent final verification, but mirrors the
    /// Minisketch distinction between stored capacity and maximum decoded
    /// elements. A production false-positive bound still requires a separately
    /// justified contract.
    pub fn decode_candidate_with_limit(&self, max_elements: usize) -> Result<Vec<u64>, LabError> {
        if max_elements > self.capacity {
            return Err(LabError::InvalidDecodeLimit);
        }

        let zero_count = usize::from(self.zero_present);
        if zero_count > max_elements {
            return Err(LabError::CandidateExceedsCapacity);
        }
        let nonzero_limit = max_elements - zero_count;
        let sequence = self.full_syndrome_sequence(nonzero_limit);
        let locator = berlekamp_massey(&sequence)?;

        if locator.is_empty() {
            return Err(LabError::DecodeFailure);
        }
        let degree = locator.len() - 1;
        if degree > nonzero_limit {
            return Err(LabError::DecodeFailure);
        }

        let mut roots = if degree == 0 {
            Vec::new()
        } else {
            let mut root_polynomial: Vec<u64> = locator.into_iter().rev().collect();
            trim(&mut root_polynomial);
            make_monic(&mut root_polynomial)?;
            factor_linear_roots(&root_polynomial)?
        };

        roots.sort_unstable();
        if roots.windows(2).any(|pair| pair[0] == pair[1])
            || roots.contains(&0)
            || roots.len() != degree
        {
            return Err(LabError::InvalidRoots);
        }

        if self.zero_present {
            roots.push(0);
            roots.sort_unstable();
        }
        if roots.len() > max_elements {
            return Err(LabError::CandidateExceedsCapacity);
        }

        // Rechecking all stored syndromes is an over-capacity guard, not an
        // independent final verifier.
        let rebuilt = Self::from_sorted_unique(self.capacity, &roots)?;
        if rebuilt.odd_syndromes != self.odd_syndromes || rebuilt.zero_present != self.zero_present
        {
            return Err(LabError::DecodeFailure);
        }

        Ok(roots)
    }

    pub fn encoded_len_for_capacity(capacity: usize) -> Result<usize, LabError> {
        if !(1..=MAX_LAB_CAPACITY).contains(&capacity) {
            return Err(LabError::InvalidCapacity);
        }
        Ok(LAB_HEADER_LEN + capacity * 8)
    }

    fn full_syndrome_sequence(&self, nonzero_limit: usize) -> Vec<u64> {
        debug_assert!(nonzero_limit <= self.capacity);
        let mut sequence = vec![0_u64; nonzero_limit * 2];
        for exponent in 1..=nonzero_limit * 2 {
            sequence[exponent - 1] = if exponent % 2 == 1 {
                self.odd_syndromes[(exponent - 1) / 2]
            } else {
                gf64_square(sequence[exponent / 2 - 1])
            };
        }
        sequence
    }
}

fn berlekamp_massey(sequence: &[u64]) -> Result<Vec<u64>, LabError> {
    let mut connection = vec![1_u64];
    let mut previous = vec![1_u64];
    let mut length = 0_usize;
    let mut shift = 1_usize;
    let mut previous_discrepancy = 1_u64;

    for n in 0..sequence.len() {
        let mut discrepancy = sequence[n];
        for i in 1..=length {
            discrepancy ^= gf64_mul(connection[i], sequence[n - i]);
        }

        if discrepancy == 0 {
            shift += 1;
            continue;
        }

        let inverse = gf64_inv(previous_discrepancy).ok_or(LabError::DecodeFailure)?;
        let scale = gf64_mul(discrepancy, inverse);
        let old_connection = connection.clone();

        let required = previous
            .len()
            .checked_add(shift)
            .ok_or(LabError::DecodeFailure)?;
        if connection.len() < required {
            connection.resize(required, 0);
        }
        for (index, &coefficient) in previous.iter().enumerate() {
            connection[index + shift] ^= gf64_mul(scale, coefficient);
        }

        if 2 * length <= n {
            length = n + 1 - length;
            previous = old_connection;
            previous_discrepancy = discrepancy;
            shift = 1;
        } else {
            shift += 1;
        }
    }

    connection.truncate(length + 1);
    trim(&mut connection);
    if connection.is_empty() {
        return Err(LabError::DecodeFailure);
    }
    Ok(connection)
}

fn factor_linear_roots(polynomial: &[u64]) -> Result<Vec<u64>, LabError> {
    let mut polynomial = polynomial.to_vec();
    trim(&mut polynomial);
    make_monic(&mut polynomial)?;

    let degree = polynomial.len().saturating_sub(1);
    if degree == 0 {
        return Ok(Vec::new());
    }
    if degree == 1 {
        return Ok(vec![polynomial[0]]);
    }

    for bit in 0..64 {
        let coefficient = 1_u64 << bit;
        let trace = trace_polynomial_mod(coefficient, &polynomial)?;
        let factor = poly_gcd(&polynomial, &trace)?;
        let factor_degree = factor.len().saturating_sub(1);

        if factor_degree == 0 || factor_degree == degree {
            continue;
        }

        let (quotient, remainder) = poly_div_rem(&polynomial, &factor)?;
        if !remainder.is_empty() {
            return Err(LabError::DecodeFailure);
        }

        let mut left = factor_linear_roots(&factor)?;
        let mut right = factor_linear_roots(&quotient)?;
        left.append(&mut right);
        return Ok(left);
    }

    Err(LabError::DecodeFailure)
}

fn trace_polynomial_mod(coefficient: u64, modulus: &[u64]) -> Result<Vec<u64>, LabError> {
    let mut trace = Vec::new();
    let mut term = vec![0, coefficient];

    for _ in 0..64 {
        poly_xor_assign(&mut trace, &term);
        term = poly_mul_mod(&term, &term, modulus)?;
    }

    trim(&mut trace);
    Ok(trace)
}

fn poly_gcd(left: &[u64], right: &[u64]) -> Result<Vec<u64>, LabError> {
    let mut a = left.to_vec();
    let mut b = right.to_vec();
    trim(&mut a);
    trim(&mut b);

    while !b.is_empty() {
        let (_, remainder) = poly_div_rem(&a, &b)?;
        a = b;
        b = remainder;
    }

    if a.is_empty() {
        return Ok(a);
    }
    make_monic(&mut a)?;
    Ok(a)
}

fn poly_div_rem(dividend: &[u64], divisor: &[u64]) -> Result<(Vec<u64>, Vec<u64>), LabError> {
    let mut remainder = dividend.to_vec();
    let mut divisor = divisor.to_vec();
    trim(&mut remainder);
    trim(&mut divisor);

    if divisor.is_empty() {
        return Err(LabError::DecodeFailure);
    }
    if remainder.len() < divisor.len() {
        return Ok((Vec::new(), remainder));
    }

    let divisor_inverse =
        gf64_inv(*divisor.last().ok_or(LabError::DecodeFailure)?).ok_or(LabError::DecodeFailure)?;
    let mut quotient = vec![0_u64; remainder.len() - divisor.len() + 1];

    while !remainder.is_empty() && remainder.len() >= divisor.len() {
        let shift = remainder.len() - divisor.len();
        let factor = gf64_mul(
            *remainder.last().ok_or(LabError::DecodeFailure)?,
            divisor_inverse,
        );
        quotient[shift] ^= factor;

        for (index, &coefficient) in divisor.iter().enumerate() {
            remainder[index + shift] ^= gf64_mul(factor, coefficient);
        }
        trim(&mut remainder);
    }

    trim(&mut quotient);
    Ok((quotient, remainder))
}

fn poly_mul_mod(left: &[u64], right: &[u64], modulus: &[u64]) -> Result<Vec<u64>, LabError> {
    if left.is_empty() || right.is_empty() {
        return Ok(Vec::new());
    }

    let mut product = vec![0_u64; left.len() + right.len() - 1];
    for (left_index, &left_value) in left.iter().enumerate() {
        if left_value == 0 {
            continue;
        }
        for (right_index, &right_value) in right.iter().enumerate() {
            if right_value == 0 {
                continue;
            }
            product[left_index + right_index] ^= gf64_mul(left_value, right_value);
        }
    }

    let (_, remainder) = poly_div_rem(&product, modulus)?;
    Ok(remainder)
}

fn poly_xor_assign(left: &mut Vec<u64>, right: &[u64]) {
    if left.len() < right.len() {
        left.resize(right.len(), 0);
    }
    for (index, &value) in right.iter().enumerate() {
        left[index] ^= value;
    }
    trim(left);
}

fn make_monic(polynomial: &mut Vec<u64>) -> Result<(), LabError> {
    trim(polynomial);
    let leading = *polynomial.last().ok_or(LabError::DecodeFailure)?;
    if leading == 1 {
        return Ok(());
    }
    let inverse = gf64_inv(leading).ok_or(LabError::DecodeFailure)?;
    for coefficient in polynomial.iter_mut() {
        *coefficient = gf64_mul(*coefficient, inverse);
    }
    Ok(())
}

fn trim(polynomial: &mut Vec<u64>) {
    while polynomial.last() == Some(&0) {
        polynomial.pop();
    }
}

fn gf64_inv(value: u64) -> Option<u64> {
    if value == 0 {
        return None;
    }
    Some(gf64_pow(value, u64::MAX - 1))
}

fn gf64_pow(mut base: u64, mut exponent: u64) -> u64 {
    let mut result = 1_u64;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = gf64_mul(result, base);
        }
        exponent >>= 1;
        if exponent != 0 {
            base = gf64_square(base);
        }
    }
    result
}

#[inline]
fn gf64_square(value: u64) -> u64 {
    gf64_mul(value, value)
}

#[inline]
fn gf64_mul(mut left: u64, mut right: u64) -> u64 {
    let mut product = 0_u64;

    for _ in 0..64 {
        if right & 1 != 0 {
            product ^= left;
        }
        right >>= 1;

        let carry = left >> 63;
        left <<= 1;
        if carry != 0 {
            left ^= GF64_REDUCTION;
        }
    }

    product
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gf64_multiplication_matches_energy_reference_vectors() {
        let vectors = [
            (
                0x0123_4567_89AB_CDEF,
                0xF0E1_D2C3_B4A5_9687,
                0x287C_26FE_0540_0BC2,
            ),
            (
                0xFFFF_FFFF_FFFF_FFFF,
                0xFFFF_FFFF_FFFF_FFFF,
                0x5555_5555_5555_5513,
            ),
            (
                0x8000_0000_0000_0000,
                0x0000_0000_0000_0002,
                0x0000_0000_0000_001B,
            ),
            (
                0xDEAD_BEEF_CAFE_BABE,
                0x0123_4567_89AB_CDEF,
                0xFBB6_7120_92FD_6A8C,
            ),
        ];

        for (left, right, expected) in vectors {
            assert_eq!(gf64_mul(left, right), expected);
            assert_eq!(gf64_mul(right, left), expected);
        }
    }

    #[test]
    fn polynomial_division_and_gcd_recover_linear_factor() {
        let a = 0x0123_4567_89AB_CDEF;
        let b = 0xDEAD_BEEF_CAFE_BABE;
        let left = vec![a, 1];
        let right = vec![b, 1];
        let product = vec![gf64_mul(a, b), a ^ b, 1];

        let (quotient, remainder) = poly_div_rem(&product, &left).unwrap();
        assert!(remainder.is_empty());
        assert_eq!(quotient, right);
        assert_eq!(poly_gcd(&product, &left).unwrap(), left);
    }

    #[test]
    fn field_inverse_round_trips_edge_values() {
        for value in [
            1,
            2,
            3,
            1_u64 << 63,
            u64::MAX,
            0x0123_4567_89AB_CDEF,
            0xDEAD_BEEF_CAFE_BABE,
        ] {
            let inverse = gf64_inv(value).unwrap();
            assert_eq!(gf64_mul(value, inverse), 1, "value={value:#018x}");
        }
    }

    #[test]
    fn one_element_locator_recovers_root() {
        let key = 0xDEAD_BEEF_CAFE_BABE;
        let sketch = PinSketch64Lab::from_sorted_unique(1, &[key]).unwrap();
        assert_eq!(sketch.decode_candidate_with_limit(1).unwrap(), vec![key]);
    }

    #[test]
    fn zero_is_out_of_band_and_exact() {
        let sketch = PinSketch64Lab::from_sorted_unique(2, &[0, u64::MAX]).unwrap();
        assert!(sketch.zero_present());
        assert_eq!(sketch.decode_candidate_with_limit(2).unwrap(), vec![0, u64::MAX]);
    }

    #[test]
    fn lab_encoding_round_trips_and_rejects_padding() {
        let sketch = PinSketch64Lab::from_sorted_unique(4, &[0, 3, 9]).unwrap();
        let bytes = sketch.encode();
        assert_eq!(PinSketch64Lab::decode(&bytes).unwrap(), sketch);

        let mut bad = bytes.clone();
        bad[11] = 1;
        assert_eq!(
            PinSketch64Lab::decode(&bad).unwrap_err(),
            LabError::InvalidEncoding
        );

        assert!(PinSketch64Lab::decode(&bytes[..bytes.len() - 1]).is_err());
    }
}
