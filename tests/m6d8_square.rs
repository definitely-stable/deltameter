#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "../examples/support/m6d6_quadratic.rs"]
mod quadratic;
#[path = "../examples/support/m6d8_square.rs"]
mod square_candidate;
#[path = "../examples/support/m6d4_trace_square.rs"]
mod trace_square;

use pinsketch64::PinSketch64Lab;
use quadratic::{decode_with_locator_quadratic, gf64_square_reference_for_d8};
use square_candidate::{decode_with_locator_square_candidate, gf64_square_candidate};
use trace_square::fresh_locator;

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
fn candidate_square_matches_frozen_scalar_on_basis() {
    for bit in 0..64 {
        let value = 1_u64 << bit;
        assert_eq!(
            gf64_square_candidate(value),
            gf64_square_reference_for_d8(value),
            "basis bit {bit}"
        );
    }
}

#[test]
fn candidate_square_matches_edges_and_deterministic_vectors() {
    for value in [0, 1, 2, 3, 1_u64 << 63, u64::MAX, 0x0123_4567_89AB_CDEF] {
        assert_eq!(
            gf64_square_candidate(value),
            gf64_square_reference_for_d8(value),
            "edge {value:#018x}"
        );
    }

    for index in 0..4096_u64 {
        let value = splitmix64(0xD800_5A5A_0000_0000 ^ index);
        assert_eq!(
            gf64_square_candidate(value),
            gf64_square_reference_for_d8(value),
            "vector {index}"
        );
    }
}

#[test]
fn candidate_decoder_matches_accepted_d6_across_five_corpora() {
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
        (
            0xD700_BA5E_0000_0001,
            0xD700_D1FF_0000_0000,
            0xD700_5A5A_0000_0000,
        ),
        (
            0xD701_BA5E_0000_0001,
            0xD701_D1FF_0000_0000,
            0xD701_5A5A_0000_0000,
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

                let control = decode_with_locator_quadratic(&sketch, limit, &locator);
                let candidate =
                    decode_with_locator_square_candidate(&sketch, limit, &locator);

                assert_eq!(
                    candidate, control,
                    "candidate/D6 mismatch d={difference} limit={limit}"
                );

                if let Ok(roots) = control {
                    assert_eq!(roots, expected);
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
fn full_width_zero_and_invalid_limit_semantics_remain_frozen() {
    let left = vec![0, 1, 2, 1_u64 << 63, u64::MAX];
    let right = vec![1, 2, 3, 1_u64 << 63];
    let expected = vec![0, 3, u64::MAX];

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(&left_full, &right_full, capacity);
        let locator = fresh_locator(&sketch, limit).unwrap();

        let control = decode_with_locator_quadratic(&sketch, limit, &locator);
        let candidate = decode_with_locator_square_candidate(&sketch, limit, &locator);
        assert_eq!(candidate, control);

        if let Ok(roots) = control {
            assert_eq!(roots, expected);
            break;
        }
    }

    let sketch = PinSketch64Lab::from_sorted_unique(3, &[0, 1]).unwrap();
    for (limit, locator) in [(4, vec![0, 0]), (0, vec![0, 0]), (1, vec![1, 1])] {
        assert_eq!(
            decode_with_locator_square_candidate(&sketch, limit, &locator),
            decode_with_locator_quadratic(&sketch, limit, &locator)
        );
    }
}
