//! M6-D5 private instrumentation for the accepted D4 root path.
//!
//! No factorization algorithm changes are made here. The purpose is to assign
//! non-overlapping local factor work to polynomial degrees and to time final
//! candidate verification separately.

use std::time::Instant;

use crate::pinsketch64::{LabError, PinSketch64Lab};
use crate::trace_square::{D4Error, poly_square_mod_monic};

const GF64_REDUCTION: u64 = 0x1B;
const MAX_DEGREE: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RootProfile {
    pub factor_calls: [u64; MAX_DEGREE + 1],
    pub trace_attempts: [u64; MAX_DEGREE + 1],
    pub square_calls: [u64; MAX_DEGREE + 1],
    pub self_ns: [u128; MAX_DEGREE + 1],
    pub trace_ns: [u128; MAX_DEGREE + 1],
    pub gcd_ns: [u128; MAX_DEGREE + 1],
    pub division_ns: [u128; MAX_DEGREE + 1],
    pub factor_wall_ns: u128,
    pub verification_ns: u128,
}

impl RootProfile {
    pub fn merge(&mut self, other: &Self) {
        for degree in 0..=MAX_DEGREE {
            self.factor_calls[degree] += other.factor_calls[degree];
            self.trace_attempts[degree] += other.trace_attempts[degree];
            self.square_calls[degree] += other.square_calls[degree];
            self.self_ns[degree] += other.self_ns[degree];
            self.trace_ns[degree] += other.trace_ns[degree];
            self.gcd_ns[degree] += other.gcd_ns[degree];
            self.division_ns[degree] += other.division_ns[degree];
        }
        self.factor_wall_ns += other.factor_wall_ns;
        self.verification_ns += other.verification_ns;
    }

    pub fn validate_accounting(&self) -> Result<(), D4Error> {
        for degree in 2..=MAX_DEGREE {
            if self.square_calls[degree] != self.trace_attempts[degree] * 64 {
                return Err(LabError::DecodeFailure.into());
            }
            let measured = self.trace_ns[degree] + self.gcd_ns[degree] + self.division_ns[degree];
            if self.self_ns[degree] < measured {
                return Err(LabError::DecodeFailure.into());
            }
        }
        Ok(())
    }

}

#[derive(Debug)]
pub struct ProfiledDecode {
    pub result: Result<Vec<u64>, D4Error>,
    pub profile: RootProfile,
}

pub fn decode_with_locator_profiled(
    sketch: &PinSketch64Lab,
    max_elements: usize,
    locator: &[u64],
) -> ProfiledDecode {
    let mut profile = RootProfile::default();

    let degree = match validate_locator(sketch, max_elements, locator) {
        Ok(value) => value,
        Err(error) => {
            return ProfiledDecode {
                result: Err(error),
                profile,
            };
        }
    };

    let factor_started = Instant::now();
    let roots = factor_locator_profiled(locator, &mut profile);
    profile.factor_wall_ns = factor_started.elapsed().as_nanos();

    let result = match roots {
        Ok(roots) => {
            let verify_started = Instant::now();
            let result = finish_candidate(sketch, max_elements, degree, roots);
            profile.verification_ns = verify_started.elapsed().as_nanos();
            result
        }
        Err(error) => Err(error),
    };

    ProfiledDecode { result, profile }
}

fn factor_locator_profiled(
    locator: &[u64],
    profile: &mut RootProfile,
) -> Result<Vec<u64>, D4Error> {
    if locator.is_empty() {
        return Err(LabError::DecodeFailure.into());
    }
    if locator.len() == 1 {
        return Ok(Vec::new());
    }

    let mut root_polynomial: Vec<u64> = locator.iter().copied().rev().collect();
    trim(&mut root_polynomial);
    make_monic(&mut root_polynomial)?;
    factor_linear_roots_profiled(&root_polynomial, profile)
}

fn factor_linear_roots_profiled(
    polynomial: &[u64],
    profile: &mut RootProfile,
) -> Result<Vec<u64>, D4Error> {
    let normalize_started = Instant::now();
    let mut polynomial = polynomial.to_vec();
    trim(&mut polynomial);
    make_monic(&mut polynomial)?;
    let degree = polynomial.len().saturating_sub(1);
    if degree > MAX_DEGREE {
        return Err(LabError::DecodeFailure.into());
    }
    profile.factor_calls[degree] += 1;
    profile.self_ns[degree] += normalize_started.elapsed().as_nanos();

    if degree == 0 {
        return Ok(Vec::new());
    }
    if degree == 1 {
        return Ok(vec![polynomial[0]]);
    }

    for bit in 0..64 {
        profile.trace_attempts[degree] += 1;

        let trace_started = Instant::now();
        let coefficient = 1_u64 << bit;
        let trace = trace_polynomial_mod_profiled(coefficient, &polynomial, degree, profile)?;
        let trace_ns = trace_started.elapsed().as_nanos();
        profile.trace_ns[degree] += trace_ns;
        profile.self_ns[degree] += trace_ns;

        let gcd_started = Instant::now();
        let factor = poly_gcd(&polynomial, &trace)?;
        let gcd_ns = gcd_started.elapsed().as_nanos();
        profile.gcd_ns[degree] += gcd_ns;
        profile.self_ns[degree] += gcd_ns;

        let factor_degree = factor.len().saturating_sub(1);
        if factor_degree == 0 || factor_degree == degree {
            continue;
        }

        let division_started = Instant::now();
        let (quotient, remainder) = poly_div_rem(&polynomial, &factor)?;
        let division_ns = division_started.elapsed().as_nanos();
        profile.division_ns[degree] += division_ns;
        profile.self_ns[degree] += division_ns;

        if !remainder.is_empty() {
            return Err(LabError::DecodeFailure.into());
        }

        let mut left = factor_linear_roots_profiled(&factor, profile)?;
        let mut right = factor_linear_roots_profiled(&quotient, profile)?;
        left.append(&mut right);
        return Ok(left);
    }

    Err(LabError::DecodeFailure.into())
}

fn trace_polynomial_mod_profiled(
    coefficient: u64,
    modulus: &[u64],
    degree: usize,
    profile: &mut RootProfile,
) -> Result<Vec<u64>, D4Error> {
    let mut trace = Vec::new();
    let mut term = vec![0, coefficient];

    for _ in 0..64 {
        poly_xor_assign(&mut trace, &term);
        term = poly_square_mod_monic(&term, modulus)?;
        profile.square_calls[degree] += 1;
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
