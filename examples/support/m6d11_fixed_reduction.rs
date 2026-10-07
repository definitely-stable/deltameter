//! M6-D11 private fixed-constant polynomial-reduction candidate.
//!
//! Only multiplication inside degree>=3 trace polynomial reduction differs from
//! accepted D8. Scalar squaring, factorization, GCD/division, field representation
//! and verification remain unchanged.

use crate::pinsketch64::{LabError, PinSketch64Lab};
use crate::quadratic::quadratic_roots_monic;
use crate::trace_square::D4Error;

const GF64_REDUCTION: u64 = 0x1B;

pub fn decode_with_locator_fixed_reduction(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    locator: &[u64],
) -> Result<Vec<u64>, D4Error> {
    let degree = validate_locator(sketch, max_elements, locator)?;
    let roots = factor_locator_fixed_reduction(locator)?;
    finish_candidate(sketch, max_elements, degree, roots)
}

fn factor_locator_fixed_reduction(locator: &[u64]) -> Result<Vec<u64>, D4Error> {
    if locator.is_empty() {
        return Err(LabError::DecodeFailure.into());
    }
    if locator.len() == 1 {
        return Ok(Vec::new());
    }

    let mut root_polynomial: Vec<u64> = locator.iter().copied().rev().collect();
    trim(&mut root_polynomial);
    make_monic(&mut root_polynomial)?;
    factor_linear_roots_fixed_reduction(&root_polynomial)
}

fn factor_linear_roots_fixed_reduction(polynomial: &[u64]) -> Result<Vec<u64>, D4Error> {
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

    let reduction_plan = ReductionPlan::new(&polynomial)?;

    for bit in 0..64 {
        let coefficient = 1_u64 << bit;
        let trace =
            trace_polynomial_mod_fixed_reduction(coefficient, &polynomial, &reduction_plan)?;
        let factor = poly_gcd(&polynomial, &trace)?;
        let factor_degree = factor.len().saturating_sub(1);

        if factor_degree == 0 || factor_degree == degree {
            continue;
        }

        let (quotient, remainder) = poly_div_rem(&polynomial, &factor)?;
        if !remainder.is_empty() {
            return Err(LabError::DecodeFailure.into());
        }

        // The parent-frame reduction plan is no longer needed once the split is
        // fixed. Drop it before recursion so table memory cannot accumulate with
        // factor-tree depth.
        drop(reduction_plan);

        let mut left = factor_linear_roots_fixed_reduction(&factor)?;
        let mut right = factor_linear_roots_fixed_reduction(&quotient)?;
        left.append(&mut right);
        return Ok(left);
    }

    Err(LabError::DecodeFailure.into())
}

fn trace_polynomial_mod_fixed_reduction(
    coefficient: u64,
    modulus: &[u64],
    reduction_plan: &ReductionPlan,
) -> Result<Vec<u64>, D4Error> {
    let mut trace = Vec::new();
    let mut term = vec![0, coefficient];

    for _ in 0..64 {
        poly_xor_assign(&mut trace, &term);
        term = poly_square_mod_monic_fixed_reduction(&term, modulus, reduction_plan)?;
    }

    trim(&mut trace);
    Ok(trace)
}

fn poly_square_mod_monic_fixed_reduction(
    polynomial: &[u64],
    modulus: &[u64],
    reduction_plan: &ReductionPlan,
) -> Result<Vec<u64>, D4Error> {
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
            remainder[index * 2] = gf64_square_candidate(coefficient);
        }
    }
    trim(&mut remainder);

    let modulus_degree = modulus.len() - 1;
    while !remainder.is_empty() && remainder.len() >= modulus.len() {
        let shift = remainder.len() - modulus.len();
        let factor = *remainder.last().ok_or(LabError::DecodeFailure)?;

        remainder[shift + modulus_degree] ^= factor;

        if factor != 0 {
            for (index, &coefficient) in modulus[..modulus_degree].iter().enumerate() {
                if coefficient != 0 {
                    remainder[shift + index] ^= reduction_plan.multiply(index, factor)?;
                }
            }
        }
        trim(&mut remainder);
    }

    Ok(remainder)
}

#[derive(Clone)]
pub struct FixedMultiplier {
    table: [u64; 256],
}

impl FixedMultiplier {
    pub fn new(constant: u64) -> Self {
        let mut basis = [0_u64; 64];
        basis[0] = constant;
        for bit in 1..64 {
            basis[bit] = gf64_mul_x(basis[bit - 1]);
        }

        let mut table = [0_u64; 256];
        for position in 0..16 {
            for nibble in 0..16 {
                let mut value = 0_u64;
                for bit in 0..4 {
                    if nibble & (1 << bit) != 0 {
                        value ^= basis[position * 4 + bit];
                    }
                }
                table[position * 16 + nibble] = value;
            }
        }

        Self { table }
    }

    #[inline]
    pub fn multiply(&self, variable: u64) -> u64 {
        let mut result = 0_u64;
        for position in 0..16 {
            let nibble = ((variable >> (position * 4)) & 0xF) as usize;
            result ^= self.table[position * 16 + nibble];
        }
        result
    }
}

struct ReductionPlan {
    coefficients: Vec<u64>,
    multipliers: Vec<FixedMultiplier>,
}

impl ReductionPlan {
    fn new(modulus: &[u64]) -> Result<Self, D4Error> {
        if modulus.is_empty() || modulus.last() != Some(&1) {
            return Err(D4Error::InvalidMonicModulus);
        }

        let coefficients = modulus[..modulus.len() - 1].to_vec();
        let multipliers = coefficients
            .iter()
            .copied()
            .map(FixedMultiplier::new)
            .collect();

        Ok(Self {
            coefficients,
            multipliers,
        })
    }

    #[inline]
    fn multiply(&self, index: usize, factor: u64) -> Result<u64, D4Error> {
        let coefficient = *self
            .coefficients
            .get(index)
            .ok_or(LabError::DecodeFailure)?;
        if coefficient == 0 {
            return Ok(0);
        }

        self.multipliers
            .get(index)
            .map(|multiplier| multiplier.multiply(factor))
            .ok_or_else(|| LabError::DecodeFailure.into())
    }
}

#[inline]
fn gf64_mul_x(mut value: u64) -> u64 {
    let carry = value >> 63;
    value <<= 1;
    if carry != 0 {
        value ^= GF64_REDUCTION;
    }
    value
}

pub fn gf64_mul_reference_for_d11(left: u64, right: u64) -> u64 {
    gf64_mul(left, right)
}

/// Dedicated carryless square in the frozen polynomial basis.
///
/// First interleave input bits so the carryless square occupies even powers in
/// a 128-bit polynomial. Then fold the high half with
/// x^64 = x^4 + x^3 + x + 1 (reduction constant 0x1B). The first fold can
/// overflow by at most three polynomial degrees; the second fold is therefore
/// bounded below degree 8 and cannot overflow again.
#[inline]
pub fn gf64_square_candidate(value: u64) -> u64 {
    let spread =
        u128::from(spread32(value as u32)) | (u128::from(spread32((value >> 32) as u32)) << 64);
    let low = spread as u64;
    let high = (spread >> 64) as u64;

    let carry = (high >> 63) ^ (high >> 61) ^ (high >> 60);

    low ^ high
        ^ (high << 1)
        ^ (high << 3)
        ^ (high << 4)
        ^ carry
        ^ (carry << 1)
        ^ (carry << 3)
        ^ (carry << 4)
}

#[inline]
fn spread32(value: u32) -> u64 {
    let mut value = u64::from(value);
    value = (value | (value << 16)) & 0x0000_FFFF_0000_FFFF;
    value = (value | (value << 8)) & 0x00FF_00FF_00FF_00FF;
    value = (value | (value << 4)) & 0x0F0F_0F0F_0F0F_0F0F;
    value = (value | (value << 2)) & 0x3333_3333_3333_3333;
    (value | (value << 1)) & 0x5555_5555_5555_5555
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
            base = gf64_square_scalar_reference(base);
        }
    }
    result
}

#[inline]
pub fn gf64_square_scalar_reference(value: u64) -> u64 {
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
