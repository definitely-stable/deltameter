#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "../examples/support/m6d6_quadratic.rs"]
mod quadratic;
#[path = "../examples/support/m6d4_trace_square.rs"]
mod trace_square;

use pinsketch64::PinSketch64Lab;
use quadratic::{decode_with_locator_quadratic, quadratic_roots_monic, solve_artin_schreier};
use trace_square::{
    D4Error, decode_with_locator_generic, decode_with_locator_specialized, fresh_locator,
    generic_square_mod, poly_square_mod_monic,
};

const GF64_REDUCTION: u64 = 0x1B;
const STAGES: [(usize, usize); 4] = [(1, 2), (2, 3), (4, 5), (8, 9)];

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

fn gf64_square(value: u64) -> u64 {
    gf64_mul(value, value)
}

fn field_trace(mut value: u64) -> u64 {
    let original = value;
    for _ in 1..64 {
        value = gf64_square(value) ^ original;
    }
    value
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn base_set(count: usize, salt: u64) -> Vec<u64> {
    let mut values: Vec<_> = (0..count)
        .map(|index| splitmix64(salt ^ index as u64))
        .collect();
    values.sort_unstable();
    values.dedup();
    assert_eq!(values.len(), count);
    values
}

fn derive_source(base: &[u64], difference: usize, salt: u64, include_zero: bool) -> Vec<u64> {
    if difference == 0 {
        return base.to_vec();
    }
    if include_zero {
        assert_eq!(difference, 1);
        let mut values = base.to_vec();
        values.push(0);
        values.sort_unstable();
        return values;
    }

    let remove = difference / 2;
    let add = difference - remove;
    let mut values = base[remove..].to_vec();
    for index in 0..add {
        values.push(splitmix64(salt ^ 0xD600_5A5A_0000_0000 ^ index as u64));
    }
    values.sort_unstable();
    values.dedup();
    values
}

fn symmetric_difference(left: &[u64], right: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => {
                result.push(left[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                result.push(right[j]);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    result.extend_from_slice(&left[i..]);
    result.extend_from_slice(&right[j..]);
    result
}

fn prefix(source: &PinSketch64Lab, capacity: usize) -> PinSketch64Lab {
    let mut bytes = source.encode();
    let expected = PinSketch64Lab::encoded_len_for_capacity(capacity).unwrap();
    bytes[8..10].copy_from_slice(&(capacity as u16).to_le_bytes());
    bytes.truncate(expected);
    PinSketch64Lab::decode(&bytes).unwrap()
}

fn difference_prefix(
    left: &PinSketch64Lab,
    right: &PinSketch64Lab,
    capacity: usize,
) -> PinSketch64Lab {
    let mut difference = prefix(left, capacity);
    difference.merge(&prefix(right, capacity)).unwrap();
    difference
}

#[test]
fn artin_schreier_solver_recovers_image_elements() {
    let values = [
        0,
        1,
        2,
        3,
        1_u64 << 63,
        u64::MAX,
        0x0123_4567_89AB_CDEF,
        0xDEAD_BEEF_CAFE_BABE,
    ];

    for value in values {
        let c = gf64_square(value) ^ value;
        let solved = solve_artin_schreier(c).unwrap();
        assert_eq!(gf64_square(solved) ^ solved, c);
        assert_eq!(
            solved & 1,
            0,
            "solver must choose the bit-zero canonical preimage"
        );
    }
}

#[test]
fn artin_schreier_solver_rejects_trace_one_rhs() {
    let inconsistent = (1_u64..=u16::MAX as u64)
        .find(|&candidate| field_trace(candidate) == 1)
        .expect("trace-one field element");
    assert!(solve_artin_schreier(inconsistent).is_err());
}

#[test]
fn quadratic_solver_recovers_deterministic_distinct_root_pairs() {
    let pairs = [
        (1, 2),
        (3, 5),
        (1_u64 << 63, u64::MAX),
        (0x0123_4567_89AB_CDEF, 0xDEAD_BEEF_CAFE_BABE),
        (splitmix64(0xD600_1001), splitmix64(0xD600_1002)),
    ];

    for (left, right) in pairs {
        assert_ne!(left, 0);
        assert_ne!(right, 0);
        assert_ne!(left, right);

        let a = left ^ right;
        let b = gf64_mul(left, right);
        let polynomial = [b, a, 1];

        let mut roots = quadratic_roots_monic(&polynomial).unwrap();
        roots.sort_unstable();
        let mut expected = vec![left, right];
        expected.sort_unstable();
        assert_eq!(roots, expected);

        for root in roots {
            assert_eq!(
                gf64_square(root) ^ gf64_mul(a, root) ^ b,
                0,
                "root must satisfy quadratic"
            );
        }
    }
}

#[test]
fn quadratic_solver_rejects_repeated_root_form() {
    for b in [0, 1, 1_u64 << 63, u64::MAX] {
        assert!(quadratic_roots_monic(&[b, 0, 1]).is_err());
    }
}

#[test]
fn quadratic_decoder_matches_d4_and_exact_oracle() {
    let left = base_set(128, 0xD600_2000);

    for difference in [0_usize, 1, 2, 3, 4, 5, 8, 9, 10, 16] {
        let right = derive_source(
            &left,
            difference,
            0xD600_3000 ^ difference as u64,
            difference == 1,
        );
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), difference);

        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

        let mut completed = false;
        for (limit, capacity) in STAGES {
            let sketch = difference_prefix(&left_full, &right_full, capacity);
            let locator = fresh_locator(&sketch, limit).unwrap();

            let frozen = sketch.decode_candidate_with_limit(limit);
            let generic = decode_with_locator_generic(&sketch, limit, &locator);
            let accepted = decode_with_locator_specialized(&sketch, limit, &locator);
            let candidate = decode_with_locator_quadratic(&sketch, limit, &locator);

            assert_eq!(generic, frozen.clone().map_err(D4Error::from));
            assert_eq!(accepted, frozen.clone().map_err(D4Error::from));
            assert_eq!(
                candidate, accepted,
                "D6/D4 mismatch d={difference} limit={limit}"
            );

            if let Ok(roots) = frozen {
                assert_eq!(roots, expected);
                assert!(difference <= limit);
                completed = true;
                break;
            }
        }

        assert_eq!(completed, difference <= 8, "d={difference}");
    }
}

#[test]
fn full_width_zero_and_guard_semantics_remain_frozen() {
    let left = vec![0, 1, 2, 1_u64 << 63, u64::MAX];
    let right = vec![1, 2, 3, 1_u64 << 63];
    let expected = vec![0, 3, u64::MAX];

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(&left_full, &right_full, capacity);
        let locator = fresh_locator(&sketch, limit).unwrap();
        let accepted = decode_with_locator_specialized(&sketch, limit, &locator);
        let candidate = decode_with_locator_quadratic(&sketch, limit, &locator);
        assert_eq!(candidate, accepted);

        if let Ok(roots) = candidate {
            assert_eq!(roots, expected);
            break;
        }
    }

    let sketch = PinSketch64Lab::from_sorted_unique(3, &[1, u64::MAX]).unwrap();
    let locator = fresh_locator(&sketch, 2).unwrap();
    let mut bytes = sketch.encode();
    bytes[32] ^= 1;
    let damaged = PinSketch64Lab::decode(&bytes).unwrap();
    assert_eq!(fresh_locator(&damaged, 2).unwrap(), locator);
    assert!(decode_with_locator_specialized(&damaged, 2, &locator).is_err());
    assert!(decode_with_locator_quadratic(&damaged, 2, &locator).is_err());
}

#[test]
fn invalid_limits_keep_d4_error_precedence() {
    let sketch = PinSketch64Lab::from_sorted_unique(3, &[0, 1]).unwrap();

    for (limit, locator) in [(4, vec![0, 0]), (0, vec![0, 0]), (1, vec![1, 1])] {
        assert_eq!(
            decode_with_locator_quadratic(&sketch, limit, &locator),
            decode_with_locator_specialized(&sketch, limit, &locator)
        );
    }
}

#[test]
fn accepted_square_control_is_unchanged() {
    let polynomial = [0x0123_4567_89AB_CDEF, 0xDEAD_BEEF_CAFE_BABE];
    let modulus = [0xA5A5_5A5A_F0F0_0F0F, 0x1357_9BDF_2468_ACE0, 1];
    assert_eq!(
        generic_square_mod(&polynomial, &modulus).unwrap(),
        poly_square_mod_monic(&polynomial, &modulus).unwrap()
    );
}
