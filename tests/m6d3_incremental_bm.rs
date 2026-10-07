#[path = "../examples/support/m6d3_incremental_bm.rs"]
mod incremental_bm;
#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;

use incremental_bm::{
    D3Error, IncrementalBmDecoder, decode_with_locator, fresh_connection_from_sequence,
    fresh_full_sequence, prefix,
};
use pinsketch64::PinSketch64Lab;

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
        values.push(splitmix64(salt ^ 0xD300_5A5A_0000_0000 ^ index as u64));
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

fn difference_prefix(
    left: &PinSketch64Lab,
    right: &PinSketch64Lab,
    capacity: usize,
) -> PinSketch64Lab {
    let mut difference = prefix(left, capacity).unwrap();
    difference.merge(&prefix(right, capacity).unwrap()).unwrap();
    difference
}

#[test]
fn incremental_bm_matches_fresh_state_at_every_stage() {
    let left = base_set(96, 0xD300_0001);

    for difference in [0_usize, 1, 2, 3, 4, 5, 8] {
        let right = derive_source(
            &left,
            difference,
            0xD300_1000 ^ difference as u64,
            difference == 1,
        );
        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
        let mut decoder = IncrementalBmDecoder::new();

        for (limit, stored_capacity) in STAGES {
            let sketch = difference_prefix(&left_full, &right_full, stored_capacity);
            decoder.extend_to(&sketch, limit).unwrap();

            assert_eq!(
                decoder.sequence(),
                fresh_full_sequence(&sketch, limit).unwrap(),
                "sequence d={difference} limit={limit}"
            );
            let fresh_sequence = fresh_full_sequence(&sketch, limit).unwrap();
            let fresh_locator = fresh_connection_from_sequence(&fresh_sequence).unwrap();
            assert_eq!(
                decoder.connection_polynomial(),
                fresh_locator,
                "locator d={difference} limit={limit}"
            );
            assert_eq!(
                decoder.linear_complexity() + 1,
                decoder.connection_polynomial().len(),
                "degree d={difference} limit={limit}"
            );
        }
    }
}

#[test]
fn incremental_candidates_match_d1_and_exact_oracle() {
    let left = base_set(128, 0xD300_2000);

    for difference in [0_usize, 1, 2, 3, 4, 5, 8] {
        let right = derive_source(
            &left,
            difference,
            0xD300_3000 ^ difference as u64,
            difference == 1,
        );
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), difference);

        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
        let mut decoder = IncrementalBmDecoder::new();

        let mut completed = None;
        for (limit, stored_capacity) in STAGES {
            let sketch = difference_prefix(&left_full, &right_full, stored_capacity);
            decoder.extend_to(&sketch, limit).unwrap();

            let d1 = sketch.decode_candidate_with_limit(limit);
            let locator = decoder.connection_polynomial();
            let d3 = decode_with_locator(&sketch, limit, &locator);

            match (d1, d3) {
                (Ok(reference), Ok(candidate)) => {
                    assert_eq!(candidate, reference, "d={difference} limit={limit}");
                    assert_eq!(candidate, expected, "d={difference} limit={limit}");
                    completed = Some(limit);
                    break;
                }
                (Err(_), Err(_)) => {}
                (reference, candidate) => {
                    panic!(
                        "D1/D3 outcome mismatch d={difference} limit={limit}: D1={reference:?} D3={candidate:?}"
                    );
                }
            }
        }

        assert_eq!(
            completed,
            Some(if difference <= 1 {
                1
            } else if difference <= 2 {
                2
            } else if difference <= 4 {
                4
            } else {
                8
            })
        );
    }
}

#[test]
fn over_bound_matrix_remains_rejected() {
    let left = base_set(128, 0xD300_4000);

    for difference in [9_usize, 10, 16] {
        let right = derive_source(&left, difference, 0xD300_5000 ^ difference as u64, false);
        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
        let mut decoder = IncrementalBmDecoder::new();

        for (limit, stored_capacity) in STAGES {
            let sketch = difference_prefix(&left_full, &right_full, stored_capacity);
            decoder.extend_to(&sketch, limit).unwrap();

            assert!(
                sketch.decode_candidate_with_limit(limit).is_err(),
                "D1 unexpectedly accepted d={difference} limit={limit}"
            );
            assert!(
                decoder.decode_current(&sketch, limit).is_err(),
                "D3 unexpectedly accepted d={difference} limit={limit}"
            );
        }
    }
}

#[test]
fn historical_syndrome_or_zero_change_fails_closed() {
    let stable = PinSketch64Lab::from_sorted_unique(9, &[1, 3, 5, u64::MAX]).unwrap();
    let changed = PinSketch64Lab::from_sorted_unique(9, &[1, 3, 7, u64::MAX]).unwrap();
    let with_zero = PinSketch64Lab::from_sorted_unique(9, &[0, 1, 3, 5, u64::MAX]).unwrap();

    let mut decoder = IncrementalBmDecoder::new();
    let first = prefix(&stable, 2).unwrap();
    decoder.extend_to(&first, 1).unwrap();

    let changed_prefix = prefix(&changed, 3).unwrap();
    assert_eq!(
        decoder.extend_to(&changed_prefix, 2).unwrap_err(),
        D3Error::HistoricalSyndromeChanged
    );

    let zero_prefix = prefix(&with_zero, 3).unwrap();
    assert_eq!(
        decoder.extend_to(&zero_prefix, 2).unwrap_err(),
        D3Error::ZeroMetadataChanged
    );
}

#[test]
fn full_width_values_keep_exact_identity_semantics() {
    let left = vec![0, 1, 2, 1_u64 << 63, u64::MAX];
    let right = vec![1, 2, 3, 1_u64 << 63];
    let expected = vec![0, 3, u64::MAX];

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
    let mut decoder = IncrementalBmDecoder::new();

    for (limit, stored_capacity) in STAGES {
        let sketch = difference_prefix(&left_full, &right_full, stored_capacity);
        decoder.extend_to(&sketch, limit).unwrap();
        if let Ok(candidate) = decoder.decode_current(&sketch, limit) {
            assert_eq!(candidate, expected);
            return;
        }
    }

    panic!("full-width exact case did not decode");
}
