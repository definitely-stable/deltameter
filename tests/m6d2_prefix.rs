#[path = "../examples/support/m6d2_prefix.rs"]
mod prefix_lab;

use prefix_lab::{PrefixError, PinSketch64Lab, extend_prefix, prefix};

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
    let remove = difference / 2;
    let add = difference - remove;
    let mut values = base[remove..].to_vec();
    for index in 0..add {
        values.push(splitmix64(salt ^ 0xA5A5_5A5A_0000_0000 ^ index as u64));
    }
    if include_zero {
        values.push(0);
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

#[test]
fn prefixes_are_exact_and_extensions_append_only_suffix_words() {
    let keys = {
        let mut keys = base_set(32, 0xD200_0001);
        keys.push(0);
        keys.sort_unstable();
        keys
    };
    let full = PinSketch64Lab::from_sorted_unique(9, &keys).unwrap();

    let mut received = prefix(&full, 2).unwrap();
    assert_eq!(received.odd_syndromes(), &full.odd_syndromes()[..2]);
    assert_eq!(received.zero_present(), full.zero_present());

    for (new_capacity, expected_added) in [(3, 1), (5, 2), (9, 4)] {
        let before = received.odd_syndromes().to_vec();
        let added = extend_prefix(&mut received, &full, new_capacity).unwrap();
        assert_eq!(added, expected_added);
        assert_eq!(&received.odd_syndromes()[..before.len()], before.as_slice());
        assert_eq!(
            received.odd_syndromes(),
            &full.odd_syndromes()[..new_capacity]
        );
        assert_eq!(received.zero_present(), full.zero_present());
    }

    assert_eq!(received, full);
}

#[test]
fn extension_rejects_mismatched_prefix_or_zero_metadata() {
    let full = PinSketch64Lab::from_sorted_unique(9, &[1, 3, 5]).unwrap();
    let other = PinSketch64Lab::from_sorted_unique(9, &[1, 3, 7]).unwrap();
    let mut prefix = prefix(&full, 2).unwrap();

    assert_eq!(
        extend_prefix(&mut prefix, &other, 3).unwrap_err(),
        PrefixError::PrefixMismatch
    );

    let full_with_zero = PinSketch64Lab::from_sorted_unique(9, &[0, 1, 3, 5]).unwrap();
    assert_eq!(
        extend_prefix(&mut prefix, &full_with_zero, 3).unwrap_err(),
        PrefixError::PrefixMismatch
    );
}

#[test]
fn merged_prefix_matches_prefix_of_full_merged_sketch() {
    let left = base_set(64, 0xD200_0010);
    let right = derive_source(&left, 5, 0xD200_0020, false);

    let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
    let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

    let mut full_difference = left_full.clone();
    full_difference.merge(&right_full).unwrap();

    for (_, stored_capacity) in STAGES {
        let mut prefix_difference = left_prefix(&full, stored_capacity).unwrap();
        prefix_difference
            .merge(&right_prefix(&full, stored_capacity).unwrap())
            .unwrap();
        assert_eq!(
            prefix_difference,
            prefix(&full_difference, stored_capacity).unwrap()
        );
    }
}

#[test]
fn staged_decode_reaches_first_sufficient_guarded_prefix() {
    let left = base_set(128, 0xD200_0100);

    for difference in [0_usize, 1, 2, 3, 4, 5, 8] {
        let right = derive_source(
            &left,
            difference,
            0xD200_0200 ^ difference as u64,
            difference == 1,
        );
        let expected = symmetric_difference(&left, &right);
        let exact_d = expected.len();

        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

        let mut completed = None;
        for (limit, stored_capacity) in STAGES {
            let mut difference_prefix = left_prefix(&full, stored_capacity).unwrap();
            difference_prefix
                .merge(&right_prefix(&full, stored_capacity).unwrap())
                .unwrap();

            if let Ok(candidate) = difference_prefix.decode_candidate_with_limit(limit) {
                assert_eq!(candidate, expected);
                completed = Some(limit);
                break;
            }
        }

        assert_eq!(
            completed,
            Some(if exact_d <= 1 {
                1
            } else if exact_d <= 2 {
                2
            } else if exact_d <= 4 {
                4
            } else {
                8
            }),
            "d={exact_d}"
        );
    }
}

#[test]
fn over_bound_stages_never_claim_exact_oracle_success() {
    let left = base_set(128, 0xD200_1000);

    for difference in [9_usize, 10, 16] {
        let right = derive_source(&left, difference, 0xD200_2000 ^ difference as u64, false);
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), difference);

        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

        for (limit, stored_capacity) in STAGES {
            let mut difference_prefix = left_prefix(&full, stored_capacity).unwrap();
            difference_prefix
                .merge(&right_prefix(&full, stored_capacity).unwrap())
                .unwrap();

            match difference_prefix.decode_candidate_with_limit(limit) {
                Ok(candidate) => assert_ne!(
                    candidate, expected,
                    "over-bound result must not equal the exact oracle: d={difference} limit={limit}"
                ),
                Err(_) => {}
            }
        }
    }
}
