//! M6-D6 private deterministic quadratic root solver candidate.
//!
//! Only degree-two factorization differs from the accepted D4 path. Higher
//! degrees retain deterministic trace splitting and D4 characteristic-2 square.

use crate::pinsketch64::{LabError, PinSketch64Lab};
use crate::trace_square::{D4Error, poly_square_mod_monic};

const GF64_REDUCTION: u64 = 0x1B;
const RHS_BIT: u64 = 1_u64 << 63;
const COEFFICIENT_MASK: u64 = RHS_BIT - 1;

pub fn decode_with_locator_quadratic(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    locator: &[u64],
) -> Result<Vec<u64>, D4Error> {
    let degree = validate_locator(sketch, max_elements, locator)?;
    let roots = factor_locator_quadratic(locator)?;
    finish_candidate(sketch, max_elements, degree, roots)
}

pub fn quadratic_roots_monic(polynomial: &[u64]) -> Result<Vec<u64>, D4Error> {
    let mut polynomial = polynomial.to_vec();
    trim(&mut polynomial);
    if polynomial.len() != 3 || polynomial[2] != 1 {
        return Err(LabError::DecodeFailure.into());
    }

    let b = polynomial[0];
    let a = polynomial[1];
    if a == 0 {
        // x^2 + b has a repeated root in GF(2^64); the frozen exact-set
        // locator path requires distinct roots.
        return Err(LabError::DecodeFailure.into());
    }

    let inverse_a = gf64_inv(a).ok_or(LabError::DecodeFailure)?;
    let c = gf64_mul(b, gf64_square(inverse_a));
    let y = solve_artin_schreier(c)?;

    let first = gf64_mul(a, y);
    let second = first ^ a;
    if first == second
        || eval_monic_quadratic(b, a, first) != 0
        || eval_monic_quadratic(b, a, second) != 0
    {
        return Err(LabError::DecodeFailure.into());
    }

    Ok(vec![first, second])
}

pub fn solve_artin_schreier(c: u64) -> Result<u64, D4Error> {
    // Variables are basis bits 1..63. Bit zero is fixed to zero because
    // L(1)=1^2+1=0 spans the one-dimensional kernel of L(y)=y^2+y.
    // Each row stores 63 coefficient bits plus one RHS bit.
    let mut rows = [0_u64; 64];

    for basis_bit in 1..64 {
        let basis = 1_u64 << basis_bit;
        let image = gf64_square(basis) ^ basis;
        let variable_bit = 1_u64 << (basis_bit - 1);

        for output_bit in 0..64 {
            if image & (1_u64 << output_bit) != 0 {
                rows[output_bit] ^= variable_bit;
            }
        }
    }

    for output_bit in 0..64 {
        if c & (1_u64 << output_bit) != 0 {
            rows[output_bit] ^= RHS_BIT;
        }
    }

    let mut pivot_row = 0_usize;
    let mut pivot_rows = [usize::MAX; 63];

    for column in 0..63 {
        let mask = 1_u64 << column;
        let pivot = (pivot_row..64)
            .find(|&row| rows[row] & mask != 0)
            .ok_or(LabError::DecodeFailure)?;

        rows.swap(pivot_row, pivot);

        for row in 0..64 {
            if row != pivot_row && rows[row] & mask != 0 {
                rows[row] ^= rows[pivot_row];
            }
        }

        pivot_rows[column] = pivot_row;
        pivot_row += 1;
    }

    for &row in &rows[pivot_row..] {
        if row & COEFFICIENT_MASK == 0 && row & RHS_BIT != 0 {
            return Err(LabError::DecodeFailure.into());
        }
    }

    let mut solution = 0_u64;
    for (column, &row) in pivot_rows.iter().enumerate() {
        if rows[row] & RHS_BIT != 0 {
            solution |= 1_u64 << (column + 1);
        }
    }

    if gf64_square(solution) ^ solution != c {
        return Err(LabError::DecodeFailure.into());
    }

    Ok(solution)
}

fn factor_locator_quadratic(locator: &[u64]) -> Result<Vec<u64>, D4Error> {
    if locator.is_empty() {
        return Err(LabError::DecodeFailure.into());
    }
    if locator.len() == 1 {
        return Ok(Vec::new());
    }

    let mut root_polynomial: Vec<u64> = locator.iter().copied().rev().collect();
    trim(&mut root_polynomial);
    make_monic(&mut root_polynomial)?;
    factor_linear_roots_quadratic(&root_polynomial)
}

fn factor_linear_roots_quadratic(polynomial: &[u64]) -> Result<Vec<u64>, D4Error> {
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
    if degree == 2 {
        return quadratic_roots_monic(&polynomial);
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

        let mut left = factor_linear_roots_quadratic(&factor)?;
        let mut right = factor_linear_roots_quadratic(&quotient)?;
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

fn eval_monic_quadratic(b: u64, a: u64, value: u64) -> u64 {
    gf64_square(value) ^ gf64_mul(a, value) ^ b
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
