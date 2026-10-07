#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "../examples/support/m6d6_quadratic.rs"]
mod quadratic;
#[path = "../examples/support/m6d9_residual_profile.rs"]
mod residual_profile;
#[path = "../examples/support/m6d8_square.rs"]
mod square_candidate;
#[path = "../examples/support/m6d4_trace_square.rs"]
mod trace_square;

use pinsketch64::PinSketch64Lab;
use quadratic::decode_with_locator_quadratic;
use residual_profile::decode_with_locator_profiled_d8;
use square_candidate::decode_with_locator_square_candidate;
use trace_square::{
    decode_with_locator_generic, decode_with_locator_specialized, fresh_locator,
    generic_square_mod, poly_square_mod_monic,
};

const STAGES: [(usize, usize); 4] = [(1, 2), (2, 3), (4, 5), (8, 9)];

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

fn derive_source(
    base: &[u64],
    difference: usize,
    salt: u64,
    add_mask: u64,
    include_zero: bool,
) -> Vec<u64> {
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
        values.push(splitmix64(salt ^ add_mask ^ index as u64));
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
fn profiled_post_d8_path_matches_accepted_d8() {
    let corpora = [
        (
            0xD400_BA5E_0000_0001,
            0xD400_D1FF_0000_0000,
            0xD400_5A5A_0000_0000,
        ),
        (
            0xD500_BA5E_0000_0001,
            0xD500_D1FF_0000_0000,
            0xD500_5A5A_0000_0000,
        ),
        (
            0xD600_BA5E_0000_0001,
            0xD600_D1FF_0000_0000,
            0xD600_5A5A_0000_0000,
        ),
    ];

    for (left_salt, right_salt, add_mask) in corpora {
        let left = base_set(128, left_salt);
        for difference in [0_usize, 1, 2, 3, 4, 5, 8, 9, 10, 16] {
            let right = derive_source(
                &left,
                difference,
                right_salt ^ difference as u64,
                add_mask,
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

                let d6 = decode_with_locator_quadratic(&sketch, limit, &locator);
                let accepted = decode_with_locator_square_candidate(&sketch, limit, &locator);
                let profiled = decode_with_locator_profiled_d8(&sketch, limit, &locator);
                assert_eq!(accepted, d6);

                assert_eq!(
                    profiled.result, accepted,
                    "profile/D6 mismatch d={difference} limit={limit}"
                );
                profiled.profile.validate_accounting().unwrap();

                if let Ok(candidate) = accepted {
                    assert_eq!(candidate, expected);
                    assert!(difference <= limit);
                    completed = true;
                    break;
                }
            }

            assert_eq!(completed, difference <= 8, "d={difference}");
        }
    }
}

#[test]
fn degree_two_uses_only_quadratic_solver_accounting() {
    let left = base_set(64, 0xD700_1000);
    let right = derive_source(&left, 2, 0xD700_2002, 0xD700_5A5A_0000_0000, false);
    let left_full = PinSketch64Lab::from_sorted_unique(3, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(3, &right).unwrap();
    let sketch = difference_prefix(&left_full, &right_full, 3);
    let locator = fresh_locator(&sketch, 2).unwrap();

    let profiled = decode_with_locator_profiled_d8(&sketch, 2, &locator);
    assert!(profiled.result.is_ok());
    profiled.profile.validate_accounting().unwrap();
    let mut aggregate = residual_profile::RootProfile::default();
    aggregate.merge(&profiled.profile);
    assert_eq!(aggregate.factor_calls, profiled.profile.factor_calls);

    assert_eq!(profiled.profile.factor_calls[2], 1);
    assert_eq!(profiled.profile.quadratic_calls[2], 1);
    assert_eq!(profiled.profile.trace_attempts[2], 0);
    assert_eq!(profiled.profile.square_calls[2], 0);
    assert_eq!(profiled.profile.coefficient_square_ops[2], 0);
    assert_eq!(profiled.profile.reduction_mul_ops[2], 0);
    assert!(profiled.profile.quadratic_ns[2] > 0);
    assert!(profiled.profile.self_ns[2] >= profiled.profile.quadratic_ns[2]);
}

#[test]
fn higher_degree_trace_accounting_is_preserved() {
    let left = base_set(96, 0xD700_3000);
    let right = derive_source(&left, 8, 0xD700_4008, 0xD700_5A5A_0000_0000, false);
    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
    let sketch = difference_prefix(&left_full, &right_full, 9);
    let locator = fresh_locator(&sketch, 8).unwrap();

    let profiled = decode_with_locator_profiled_d8(&sketch, 8, &locator);
    assert!(profiled.result.is_ok());
    profiled.profile.validate_accounting().unwrap();

    assert!(
        (3..=8).any(|degree| profiled.profile.factor_calls[degree] > 0),
        "d=8 must exercise at least one higher-degree factor frame"
    );
    for degree in 3..=8 {
        assert_eq!(
            profiled.profile.square_calls[degree],
            profiled.profile.trace_attempts[degree] * 64
        );
        if profiled.profile.square_calls[degree] != 0 {
            assert!(
                profiled.profile.coefficient_square_ops[degree]
                    >= profiled.profile.square_calls[degree]
            );
        }
        assert_eq!(profiled.profile.quadratic_calls[degree], 0);
    }
}

#[test]
fn full_width_zero_and_invalid_limit_semantics_remain_frozen() {
    let left = vec![0, 1, 2, 1_u64 << 63, u64::MAX];
    let right = vec![1, 2, 3, 1_u64 << 63];
    let expected = vec![0, 3, u64::MAX];

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(&left_full, &right_full, capacity);
        let locator = fresh_locator(&sketch, limit).unwrap();
        let d6 = decode_with_locator_quadratic(&sketch, limit, &locator);
        let accepted = decode_with_locator_square_candidate(&sketch, limit, &locator);
        let profiled = decode_with_locator_profiled_d8(&sketch, limit, &locator);
        assert_eq!(accepted, d6);
        assert_eq!(profiled.result, accepted);

        if let Ok(candidate) = accepted {
            assert_eq!(candidate, expected);
            break;
        }
    }

    let sketch = PinSketch64Lab::from_sorted_unique(3, &[0, 1]).unwrap();
    for (limit, locator) in [(4, vec![0, 0]), (0, vec![0, 0]), (1, vec![1, 1])] {
        assert_eq!(
            decode_with_locator_profiled_d8(&sketch, limit, &locator).result,
            decode_with_locator_square_candidate(&sketch, limit, &locator)
        );
    }

    // Keep independent D4 controls reachable in this target as regression oracles.
    let direct = PinSketch64Lab::from_sorted_unique(3, &[1, 2]).unwrap();
    let locator = fresh_locator(&direct, 2).unwrap();
    assert_eq!(
        decode_with_locator_generic(&direct, 2, &locator),
        decode_with_locator_specialized(&direct, 2, &locator)
    );

    let polynomial = [0x0123_4567_89AB_CDEF, 0xDEAD_BEEF_CAFE_BABE];
    let modulus = [0xA5A5_5A5A_F0F0_0F0F, 0x1357_9BDF_2468_ACE0, 1];
    assert_eq!(
        generic_square_mod(&polynomial, &modulus).unwrap(),
        poly_square_mod_monic(&polynomial, &modulus).unwrap()
    );
}
