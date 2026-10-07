#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "../examples/support/m6d5_root_profile.rs"]
mod root_profile;
#[path = "../examples/support/m6d4_trace_square.rs"]
mod trace_square;

use pinsketch64::PinSketch64Lab;
use root_profile::decode_with_locator_profiled;
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
        values.push(splitmix64(salt ^ 0xD500_5A5A_0000_0000 ^ index as u64));
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
fn profiled_path_matches_accepted_d4_on_frozen_matrix() {
    let left = base_set(128, 0xD500_1000);

    for difference in [0_usize, 1, 2, 3, 4, 5, 8, 9, 10, 16] {
        let right = derive_source(
            &left,
            difference,
            0xD500_2000 ^ difference as u64,
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

            let generic = decode_with_locator_generic(&sketch, limit, &locator);
            let accepted = decode_with_locator_specialized(&sketch, limit, &locator);
            let profiled = decode_with_locator_profiled(&sketch, limit, &locator);

            assert_eq!(
                generic, accepted,
                "generic/specialized mismatch d={difference} limit={limit}"
            );
            assert_eq!(
                profiled.result, accepted,
                "outcome mismatch d={difference} limit={limit}"
            );
            profiled.profile.validate_accounting().unwrap();

            for degree in 2..=8 {
                assert_eq!(
                    profiled.profile.square_calls[degree],
                    profiled.profile.trace_attempts[degree] * 64,
                    "square accounting d={difference} limit={limit} degree={degree}"
                );
            }

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

#[test]
fn degree_two_work_is_observable_without_parent_double_counting() {
    let left = base_set(64, 0xD500_3000);
    let right = derive_source(&left, 2, 0xD500_4000, false);
    let left_full = PinSketch64Lab::from_sorted_unique(3, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(3, &right).unwrap();

    let sketch = difference_prefix(&left_full, &right_full, 3);
    let locator = fresh_locator(&sketch, 2).unwrap();
    assert_eq!(locator.len() - 1, 2);

    let profiled = decode_with_locator_profiled(&sketch, 2, &locator);
    assert!(profiled.result.is_ok());
    profiled.profile.validate_accounting().unwrap();
    let mut aggregate = root_profile::RootProfile::default();
    aggregate.merge(&profiled.profile);
    assert_eq!(aggregate.factor_calls, profiled.profile.factor_calls);

    assert!(profiled.profile.factor_calls[2] >= 1);
    assert!(profiled.profile.factor_calls[1] >= 2);
    assert!(profiled.profile.trace_attempts[2] >= 1);
    assert_eq!(profiled.profile.trace_attempts[1], 0);
    assert!(profiled.profile.self_ns[2] > 0);
    assert_eq!(profiled.profile.self_ns[3..].iter().sum::<u128>(), 0);
    assert!(profiled.profile.factor_wall_ns >= profiled.profile.self_ns[2]);
    assert!(profiled.profile.verification_ns > 0);
}

#[test]
fn full_width_and_zero_semantics_are_unchanged() {
    let left = vec![0, 1, 2, 1_u64 << 63, u64::MAX];
    let right = vec![1, 2, 3, 1_u64 << 63];
    let expected = vec![0, 3, u64::MAX];

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(&left_full, &right_full, capacity);
        let locator = fresh_locator(&sketch, limit).unwrap();
        let accepted = decode_with_locator_specialized(&sketch, limit, &locator);
        let profiled = decode_with_locator_profiled(&sketch, limit, &locator);
        assert_eq!(profiled.result, accepted);
        profiled.profile.validate_accounting().unwrap();

        if let Ok(candidate) = accepted {
            assert_eq!(candidate, expected);
            return;
        }
    }

    panic!("full-width case did not decode");
}

#[test]
fn frozen_generic_square_control_remains_equivalent() {
    let polynomial = [0x0123_4567_89AB_CDEF, 0xDEAD_BEEF_CAFE_BABE];
    let modulus = [0xA5A5_5A5A_F0F0_0F0F, 0x1357_9BDF_2468_ACE0, 1];
    assert_eq!(
        generic_square_mod(&polynomial, &modulus).unwrap(),
        poly_square_mod_monic(&polynomial, &modulus).unwrap()
    );
}

#[test]
fn invalid_limits_keep_d4_error_precedence() {
    let sketch = PinSketch64Lab::from_sorted_unique(3, &[0, 1]).unwrap();

    for (limit, locator) in [(4, vec![0, 0]), (0, vec![0, 0]), (1, vec![1, 1])] {
        let accepted = decode_with_locator_specialized(&sketch, limit, &locator);
        let profiled = decode_with_locator_profiled(&sketch, limit, &locator);
        assert_eq!(profiled.result, accepted);
        assert_eq!(profiled.profile.factor_wall_ns, 0);
        assert_eq!(profiled.profile.verification_ns, 0);
    }
}
