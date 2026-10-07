//! M6-D3 private incremental Berlekamp-Massey decoder experiment.
//!
//! This module intentionally leaves the frozen D1/D2 references unchanged.
//! It continues BM state as guarded syndrome prefixes grow.

use crate::pinsketch64::{LabError, PinSketch64Lab};

const GF64_REDUCTION: u64 = 0x1B;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum D3Error {
    Lab(LabError),
    HistoricalSyndromeChanged,
    ZeroMetadataChanged,
}

impl From<LabError> for D3Error {
    fn from(value: LabError) -> Self {
        Self::Lab(value)
    }
}

#[derive(Debug, Clone)]
pub struct IncrementalBmDecoder {
    connection: Vec<u64>,
    previous: Vec<u64>,
    length: usize,
    shift: usize,
    previous_discrepancy: u64,
    sequence: Vec<u64>,
    odd_prefix: Vec<u64>,
    zero_present: Option<bool>,
}

impl Default for IncrementalBmDecoder {
    fn default() -> Self {
        Self {
            connection: vec![1],
            previous: vec![1],
            length: 0,
            shift: 1,
            previous_discrepancy: 1,
            sequence: Vec::new(),
            odd_prefix: Vec::new(),
            zero_present: None,
        }
    }
}

impl IncrementalBmDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append only newly available full-syndrome terms and continue BM.
    pub fn extend_to(
        &mut self,
        sketch: &PinSketch64Lab,
        max_elements: usize,
    ) -> Result<(), D3Error> {
        if max_elements > sketch.capacity() {
            return Err(LabError::InvalidDecodeLimit.into());
        }

        match self.zero_present {
            Some(value) if value != sketch.zero_present() => {
                return Err(D3Error::ZeroMetadataChanged);
            }
            None => self.zero_present = Some(sketch.zero_present()),
            _ => {}
        }

        if sketch.odd_syndromes().len() < self.odd_prefix.len()
            || sketch.odd_syndromes()[..self.odd_prefix.len()] != self.odd_prefix
        {
            return Err(D3Error::HistoricalSyndromeChanged);
        }

        let zero_count = usize::from(sketch.zero_present());
        if zero_count > max_elements {
            return Err(LabError::CandidateExceedsCapacity.into());
        }
        let nonzero_limit = max_elements - zero_count;
        if nonzero_limit > sketch.odd_syndromes().len() {
            return Err(LabError::InvalidDecodeLimit.into());
        }

        let target_terms = nonzero_limit
            .checked_mul(2)
            .ok_or(LabError::DecodeFailure)?;
        if target_terms < self.sequence.len() {
            return Err(LabError::InvalidDecodeLimit.into());
        }

        self.odd_prefix
            .extend_from_slice(&sketch.odd_syndromes()[self.odd_prefix.len()..nonzero_limit]);

        while self.sequence.len() < target_terms {
            let exponent = self.sequence.len() + 1;
            let value = if exponent % 2 == 1 {
                sketch.odd_syndromes()[(exponent - 1) / 2]
            } else {
                gf64_square(self.sequence[exponent / 2 - 1])
            };
            self.sequence.push(value);
            self.process_last_term()?;
        }

        Ok(())
    }

    pub fn connection_polynomial(&self) -> Vec<u64> {
        let mut connection = self.connection.clone();
        connection.truncate(self.length + 1);
        trim(&mut connection);
        connection
    }

    pub fn sequence(&self) -> &[u64] {
        &self.sequence
    }

    pub const fn linear_complexity(&self) -> usize {
        self.length
    }

    fn process_last_term(&mut self) -> Result<(), D3Error> {
        let n = self.sequence.len() - 1;
        let mut discrepancy = self.sequence[n];
        for i in 1..=self.length {
            discrepancy ^= gf64_mul(self.connection[i], self.sequence[n - i]);
        }

        if discrepancy == 0 {
            self.shift += 1;
            return Ok(());
        }

        let inverse = gf64_inv(self.previous_discrepancy).ok_or(LabError::DecodeFailure)?;
        let scale = gf64_mul(discrepancy, inverse);
        let old_connection = self.connection.clone();

        let required = self
            .previous
            .len()
            .checked_add(self.shift)
            .ok_or(LabError::DecodeFailure)?;
        if self.connection.len() < required {
            self.connection.resize(required, 0);
        }
        for (index, &coefficient) in self.previous.iter().enumerate() {
            self.connection[index + self.shift] ^= gf64_mul(scale, coefficient);
        }

        if 2 * self.length <= n {
            self.length = n + 1 - self.length;
            self.previous = old_connection;
            self.previous_discrepancy = discrepancy;
            self.shift = 1;
        } else {
            self.shift += 1;
        }

        Ok(())
    }
}

pub fn fresh_full_sequence(
    sketch: &PinSketch64Lab,
    max_elements: usize,
) -> Result<Vec<u64>, D3Error> {
    if max_elements > sketch.capacity() {
        return Err(LabError::InvalidDecodeLimit.into());
    }
    let zero_count = usize::from(sketch.zero_present());
    if zero_count > max_elements {
        return Err(LabError::CandidateExceedsCapacity.into());
    }
    let nonzero_limit = max_elements - zero_count;
    let mut sequence = Vec::with_capacity(nonzero_limit * 2);

    for exponent in 1..=nonzero_limit * 2 {
        let value = if exponent % 2 == 1 {
            sketch.odd_syndromes()[(exponent - 1) / 2]
        } else {
            gf64_square(sequence[exponent / 2 - 1])
        };
        sequence.push(value);
    }

    Ok(sequence)
}

pub fn fresh_connection_from_sequence(sequence: &[u64]) -> Result<Vec<u64>, D3Error> {
    let mut state = IncrementalBmDecoder::new();
    for &value in sequence {
        state.sequence.push(value);
        state.process_last_term()?;
    }
    Ok(state.connection_polynomial())
}

pub fn decode_with_locator(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    locator: &[u64],
) -> Result<Vec<u64>, D3Error> {
    if max_elements > sketch.capacity() {
        return Err(LabError::InvalidDecodeLimit.into());
    }

    let zero_count = usize::from(sketch.zero_present());
    if zero_count > max_elements {
        return Err(LabError::CandidateExceedsCapacity.into());
    }
    let nonzero_limit = max_elements - zero_count;

    if locator.is_empty() {
        return Err(LabError::DecodeFailure.into());
    }

    let degree = locator.len() - 1;
    if degree > nonzero_limit {
        return Err(LabError::DecodeFailure.into());
    }

    let mut roots = if degree == 0 {
        Vec::new()
    } else {
        let mut root_polynomial: Vec<u64> = locator.iter().copied().rev().collect();
        trim(&mut root_polynomial);
        make_monic(&mut root_polynomial)?;
        factor_linear_roots(&root_polynomial)?
    };

    roots.sort_unstable();
    if roots.windows(2).any(|pair| pair[0] == pair[1])
        || roots.contains(&0)
        || roots.len() != degree
    {
        return Err(LabError::InvalidRoots.into());
    }

    if sketch.zero_present() {
        roots.push(0);
        roots.sort_unstable();
    }
    if roots.len() > max_elements {
        return Err(LabError::CandidateExceedsCapacity.into());
    }

    let rebuilt = PinSketch64Lab::from_sorted_unique(sketch.capacity(), &roots)?;
    if rebuilt.odd_syndromes() != sketch.odd_syndromes()
        || rebuilt.zero_present() != sketch.zero_present()
    {
        return Err(LabError::DecodeFailure.into());
    }

    Ok(roots)
}

pub fn prefix(source: &PinSketch64Lab, capacity: usize) -> Result<PinSketch64Lab, D3Error> {
    if capacity == 0 || capacity > source.capacity() {
        return Err(LabError::InvalidCapacity.into());
    }

    let mut bytes = source.encode();
    let expected = PinSketch64Lab::encoded_len_for_capacity(capacity)?;
    bytes[8..10].copy_from_slice(&(capacity as u16).to_le_bytes());
    bytes.truncate(expected);
    Ok(PinSketch64Lab::decode(&bytes)?)
}

fn factor_linear_roots(polynomial: &[u64]) -> Result<Vec<u64>, D3Error> {
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
            return Err(LabError::DecodeFailure.into());
        }

        let mut left = factor_linear_roots(&factor)?;
        let mut right = factor_linear_roots(&quotient)?;
        left.append(&mut right);
        return Ok(left);
    }

    Err(LabError::DecodeFailure.into())
}

fn trace_polynomial_mod(coefficient: u64, modulus: &[u64]) -> Result<Vec<u64>, D3Error> {
    let mut trace = Vec::new();
    let mut term = vec![0, coefficient];

    for _ in 0..64 {
        poly_xor_assign(&mut trace, &term);
        term = poly_mul_mod(&term, &term, modulus)?;
    }

    trim(&mut trace);
    Ok(trace)
}

fn poly_gcd(left: &[u64], right: &[u64]) -> Result<Vec<u64>, D3Error> {
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

fn poly_div_rem(dividend: &[u64], divisor: &[u64]) -> Result<(Vec<u64>, Vec<u64>), D3Error> {
    let mut remainder = dividend.to_vec();
    let mut divisor = divisor.to_vec();
    trim(&mut remainder);
    trim(&mut divisor);

    if divisor.is_empty() {
        return Err(LabError::DecodeFailure.into());
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

fn poly_mul_mod(left: &[u64], right: &[u64], modulus: &[u64]) -> Result<Vec<u64>, D3Error> {
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

fn make_monic(polynomial: &mut Vec<u64>) -> Result<(), D3Error> {
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
