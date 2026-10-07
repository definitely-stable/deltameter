#[path = "../examples/support/m6d3_incremental_bm.rs"]
mod incremental_bm;
#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "../examples/support/m6d4_trace_square.rs"]
mod trace_square;

use incremental_bm::{
    IncrementalBmDecoder, decode_with_locator, fresh_connection_from_sequence, fresh_full_sequence,
    prefix,
};
use pinsketch64::PinSketch64Lab;
use trace_square::{
    D4Error, decode_with_locator_generic, decode_with_locator_specialized, fresh_locator,
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
        values.push(splitmix64(salt ^ 0xD400_5A5A_0000_0000 ^ index as u64));
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
fn specialized_square_mod_matches_generic_control() {
    let corpus = [
        0,
        1,
        2,
        3,
        1_u64 << 63,
        u64::MAX,
        0x0123_4567_89AB_CDEF,
        0xDEAD_BEEF_CAFE_BABE,
    ];

    for degree in 1_usize..=8 {
        for case in 0_u64..32 {
            let mut modulus = Vec::with_capacity(degree + 1);
            for index in 0..degree {
                let mixed = splitmix64(
                    0xD400_0000_0000_0000
                        ^ (degree as u64) << 48
                        ^ case.rotate_left(11)
                        ^ index as u64,
                );
                modulus.push(mixed ^ corpus[(index + case as usize) % corpus.len()]);
            }
            modulus.push(1);

            for poly_len in 0..=degree {
                let mut polynomial = Vec::with_capacity(poly_len);
                for index in 0..poly_len {
                    let mixed = splitmix64(
                        0xD400_1111_0000_0000
                            ^ (degree as u64) << 48
                            ^ case.rotate_left(19)
                            ^ index as u64,
                    );
                    polynomial
                        .push(mixed ^ corpus[(degree + index + case as usize) % corpus.len()]);
                }

                let generic = generic_square_mod(&polynomial, &modulus).unwrap();
                let specialized = poly_square_mod_monic(&polynomial, &modulus).unwrap();

                assert_eq!(
                    specialized, generic,
                    "degree={degree} case={case} poly_len={poly_len}"
                );
                assert!(
                    specialized.len() <= degree,
                    "non-canonical remainder degree={degree} case={case}"
                );
            }
        }
    }
}

#[test]
fn specialized_square_rejects_non_monic_modulus() {
    let err = poly_square_mod_monic(&[1, 2, 3], &[1, 2, 3]).unwrap_err();
    assert_eq!(err, D4Error::InvalidMonicModulus);
}

#[test]
fn specialized_root_path_matches_frozen_decoder_and_oracle() {
    let left = base_set(128, 0xD400_2000);

    for difference in [0_usize, 1, 2, 3, 4, 5, 8] {
        let right = derive_source(
            &left,
            difference,
            0xD400_3000 ^ difference as u64,
            difference == 1,
        );
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), difference);

        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

        let mut completed = None;
        let mut decoder = IncrementalBmDecoder::new();
        for (limit, capacity) in STAGES {
            let sketch = difference_prefix(&left_full, &right_full, capacity);
            decoder.extend_to(&sketch, limit).unwrap();
            let sequence = fresh_full_sequence(&sketch, limit).unwrap();
            assert_eq!(decoder.sequence(), sequence);
            let locator = decoder.connection_polynomial();
            assert_eq!(
                locator,
                fresh_connection_from_sequence(&sequence).unwrap(),
                "locator d={difference} limit={limit}"
            );
            assert_eq!(decoder.linear_complexity() + 1, locator.len());

            assert_eq!(fresh_locator(&sketch, limit).unwrap(), locator);
            let frozen = decode_with_locator(&sketch, limit, &locator);
            let generic = decode_with_locator_generic(&sketch, limit, &locator);
            assert_eq!(
                generic.as_ref().ok(),
                frozen.as_ref().ok(),
                "generic control candidate d={difference} limit={limit}"
            );
            assert_eq!(
                generic.is_err(),
                frozen.is_err(),
                "generic control outcome d={difference} limit={limit}"
            );
            let specialized = decode_with_locator_specialized(&sketch, limit, &locator);

            match (frozen, specialized) {
                (Ok(reference), Ok(candidate)) => {
                    assert_eq!(candidate, reference, "d={difference} limit={limit}");
                    assert_eq!(candidate, expected, "d={difference} limit={limit}");
                    completed = Some(limit);
                    break;
                }
                (Err(_), Err(_)) => {}
                (reference, candidate) => {
                    panic!(
                        "root-path mismatch d={difference} limit={limit}: frozen={reference:?} specialized={candidate:?}"
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
fn specialized_root_path_preserves_over_bound_rejects() {
    let left = base_set(128, 0xD400_4000);

    for difference in [9_usize, 10, 16] {
        let right = derive_source(&left, difference, 0xD400_5000 ^ difference as u64, false);
        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

        let mut decoder = IncrementalBmDecoder::new();
        for (limit, capacity) in STAGES {
            let sketch = difference_prefix(&left_full, &right_full, capacity);
            decoder.extend_to(&sketch, limit).unwrap();
            let sequence = fresh_full_sequence(&sketch, limit).unwrap();
            assert_eq!(decoder.sequence(), sequence);
            let locator = decoder.connection_polynomial();
            assert_eq!(locator, fresh_connection_from_sequence(&sequence).unwrap());

            assert_eq!(fresh_locator(&sketch, limit).unwrap(), locator);
            assert!(decode_with_locator(&sketch, limit, &locator).is_err());
            assert!(decode_with_locator_generic(&sketch, limit, &locator).is_err());
            assert!(decode_with_locator_specialized(&sketch, limit, &locator).is_err());
        }
    }
}

#[test]
fn specialized_root_path_preserves_zero_and_full_width_values() {
    let left = vec![0, 1, 2, 1_u64 << 63, u64::MAX];
    let right = vec![1, 2, 3, 1_u64 << 63];
    let expected = vec![0, 3, u64::MAX];

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

    let mut decoder = IncrementalBmDecoder::new();
    for (limit, capacity) in STAGES {
        let sketch = difference_prefix(&left_full, &right_full, capacity);
        decoder.extend_to(&sketch, limit).unwrap();
        let sequence = fresh_full_sequence(&sketch, limit).unwrap();
        assert_eq!(decoder.sequence(), sequence);
        let locator = decoder.connection_polynomial();
        assert_eq!(locator, fresh_connection_from_sequence(&sequence).unwrap());

        assert_eq!(fresh_locator(&sketch, limit).unwrap(), locator);
        if let Ok(candidate) = decode_with_locator_specialized(&sketch, limit, &locator) {
            assert_eq!(candidate, expected);
            assert_eq!(
                candidate,
                decode_with_locator(&sketch, limit, &locator).unwrap()
            );
            assert_eq!(
                candidate,
                decode_with_locator_generic(&sketch, limit, &locator).unwrap()
            );
            return;
        }
    }

    panic!("specialized full-width case did not decode");
}
