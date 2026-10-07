//! M6-D4 private trace-square specialization.
//!
//! This adapter changes only the Frobenius square/modulus operation inside the
//! frozen deterministic trace root finder. D1/D2/D3 references remain intact.

use crate::pinsketch64::{LabError, PinSketch64Lab};

const GF64_REDUCTION: u64 = 0x1B;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum D4Error {
    Lab(LabError),
    InvalidMonicModulus,
}

impl From<LabError> for D4Error {
    fn from(value: LabError) -> Self {
        Self::Lab(value)
    }
}

/// Frozen generic control: polynomial multiply followed by generic division.
pub fn generic_square_mod(polynomial: &[u64], modulus: &[u64]) -> Result<Vec<u64>, D4Error> {
    poly_mul_mod(polynomial, polynomial, modulus)
}

/// Characteristic-2 square followed by direct reduction against monic modulus.
pub fn poly_square_mod_monic(polynomial: &[u64], modulus: &[u64]) -> Result<Vec<u64>, D4Error> {
    let mut modulus = modulus.to_vec();
    trim(&mut modulus);
    if modulus.is_empty() || modulus.last() != Some(&1) {
        return Err(D4Error::InvalidMonicModulus);
    }
    if polynomial.is_empty() {
        return Ok(Vec::new());
    }

    let squared_len = polynomial
        .len()
        .checked_mul(2)
        .and_then(|value| value.checked_sub(1))
        .ok_or(LabError::DecodeFailure)?;
    let mut remainder = vec![0_u64; squared_len];

    for (index, &coefficient) in polynomial.iter().enumerate() {
        if coefficient != 0 {
            remainder[index * 2] = gf64_square(coefficient);
        }
    }
    trim(&mut remainder);

    let modulus_degree = modulus.len() - 1;
    while !remainder.is_empty() && remainder.len() >= modulus.len() {
        let shift = remainder.len() - modulus.len();
        let factor = *remainder.last().ok_or(LabError::DecodeFailure)?;

        // Cancel factor*x^(shift+degree) against the monic leading term
        // directly: factor * 1 == factor.
        remainder[shift + modulus_degree] ^= factor;

        if factor != 0 {
            for (index, &coefficient) in modulus[..modulus_degree].iter().enumerate() {
                if coefficient != 0 {
                    remainder[shift + index] ^= gf64_mul(factor, coefficient);
                }
            }
        }
        trim(&mut remainder);
    }

    Ok(remainder)
}

pub fn fresh_locator(sketch: &PinSketch64Lab, max_elements: usize) -> Result<Vec<u64>, D4Error> {
    if max_elements > sketch.capacity() {
        return Err(LabError::InvalidDecodeLimit.into());
    }

    let zero_count = usize::from(sketch.zero_present());
    if zero_count > max_elements {
        return Err(LabError::CandidateExceedsCapacity.into());
    }
    let nonzero_limit = max_elements - zero_count;

    let mut sequence = vec![0_u64; nonzero_limit * 2];
    for exponent in 1..=nonzero_limit * 2 {
        sequence[exponent - 1] = if exponent % 2 == 1 {
            sketch.odd_syndromes()[(exponent - 1) / 2]
        } else {
            gf64_square(sequence[exponent / 2 - 1])
        };
    }

    berlekamp_massey(&sequence)
}

pub fn decode_with_locator_generic(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    locator: &[u64],
) -> Result<Vec<u64>, D4Error> {
    let degree = validate_locator(sketch, max_elements, locator)?;
    let roots = factor_locator_generic(locator)?;
    finish_candidate(sketch, max_elements, degree, roots)
}

pub fn decode_with_locator_specialized(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    locator: &[u64],
) -> Result<Vec<u64>, D4Error> {
    let degree = validate_locator(sketch, max_elements, locator)?;
    let roots = factor_locator_specialized(locator)?;
    finish_candidate(sketch, max_elements, degree, roots)
}

fn factor_locator_generic(locator: &[u64]) -> Result<Vec<u64>, D4Error> {
    if locator.is_empty() {
        return Err(LabError::DecodeFailure.into());
    }
    if locator.len() == 1 {
        return Ok(Vec::new());
    }

    let mut root_polynomial: Vec<u64> = locator.iter().copied().rev().collect();
    trim(&mut root_polynomial);
    make_monic(&mut root_polynomial)?;
    factor_linear_roots_generic(&root_polynomial)
}

fn factor_locator_specialized(locator: &[u64]) -> Result<Vec<u64>, D4Error> {
    if locator.is_empty() {
        return Err(LabError::DecodeFailure.into());
    }
    if locator.len() == 1 {
        return Ok(Vec::new());
    }

    let mut root_polynomial: Vec<u64> = locator.iter().copied().rev().collect();
    trim(&mut root_polynomial);
    make_monic(&mut root_polynomial)?;
    factor_linear_roots_specialized(&root_polynomial)
}

// Preserve frozen D1/D3 rejection order before any root-factor work.
fn validate_locator(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    locator: &[u64],
) -> Result<usize, D4Error> {
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

    Ok(degree)
}

fn finish_candidate(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    degree: usize,
    mut roots: Vec<u64>,
) -> Result<Vec<u64>, D4Error> {
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

fn berlekamp_massey(sequence: &[u64]) -> Result<Vec<u64>, D4Error> {
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
        return Err(LabError::DecodeFailure.into());
    }
    Ok(connection)
}

fn factor_linear_roots_generic(polynomial: &[u64]) -> Result<Vec<u64>, D4Error> {
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
        let trace = trace_polynomial_mod_generic(coefficient, &polynomial)?;
        let factor = poly_gcd(&polynomial, &trace)?;
        let factor_degree = factor.len().saturating_sub(1);

        if factor_degree == 0 || factor_degree == degree {
            continue;
        }

        let (quotient, remainder) = poly_div_rem(&polynomial, &factor)?;
        if !remainder.is_empty() {
            return Err(LabError::DecodeFailure.into());
        }

        let mut left = factor_linear_roots_generic(&factor)?;
        let mut right = factor_linear_roots_generic(&quotient)?;
        left.append(&mut right);
        return Ok(left);
    }

    Err(LabError::DecodeFailure.into())
}

fn trace_polynomial_mod_generic(coefficient: u64, modulus: &[u64]) -> Result<Vec<u64>, D4Error> {
    let mut trace = Vec::new();
    let mut term = vec![0, coefficient];

    for _ in 0..64 {
        poly_xor_assign(&mut trace, &term);
        term = poly_mul_mod(&term, &term, modulus)?;
    }

    trim(&mut trace);
    Ok(trace)
}

fn factor_linear_roots_specialized(polynomial: &[u64]) -> Result<Vec<u64>, D4Error> {
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
        let trace = trace_polynomial_mod_specialized(coefficient, &polynomial)?;
        let factor = poly_gcd(&polynomial, &trace)?;
        let factor_degree = factor.len().saturating_sub(1);

        if factor_degree == 0 || factor_degree == degree {
            continue;
        }

        let (quotient, remainder) = poly_div_rem(&polynomial, &factor)?;
        if !remainder.is_empty() {
            return Err(LabError::DecodeFailure.into());
        }

        let mut left = factor_linear_roots_specialized(&factor)?;
        let mut right = factor_linear_roots_specialized(&quotient)?;
        left.append(&mut right);
        return Ok(left);
    }

    Err(LabError::DecodeFailure.into())
}

fn trace_polynomial_mod_specialized(
    coefficient: u64,
    modulus: &[u64],
) -> Result<Vec<u64>, D4Error> {
    let mut trace = Vec::new();
    let mut term = vec![0, coefficient];

    for _ in 0..64 {
        poly_xor_assign(&mut trace, &term);
        term = poly_square_mod_monic(&term, modulus)?;
    }

    trim(&mut trace);
    Ok(trace)
}

fn poly_gcd(left: &[u64], right: &[u64]) -> Result<Vec<u64>, D4Error> {
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

fn poly_div_rem(dividend: &[u64], divisor: &[u64]) -> Result<(Vec<u64>, Vec<u64>), D4Error> {
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

fn poly_mul_mod(left: &[u64], right: &[u64], modulus: &[u64]) -> Result<Vec<u64>, D4Error> {
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

fn make_monic(polynomial: &mut Vec<u64>) -> Result<(), D4Error> {
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
