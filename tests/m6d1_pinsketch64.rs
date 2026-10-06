#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;

use pinsketch64::{LabError, PinSketch64Lab};

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

fn mutate(base: &[u64], remove: usize, add: usize, salt: u64, include_zero: bool) -> Vec<u64> {
    assert!(remove <= base.len());
    let mut result = base[remove..].to_vec();
    for index in 0..add {
        result.push(splitmix64(salt ^ 0xA5A5_5A5A_1234_0000 ^ index as u64));
    }
    if include_zero {
        result.push(0);
    }
    result.sort_unstable();
    result.dedup();
    result
}

#[test]
fn exact_merge_decode_grid_with_full_width_keys() {
    for max_elements in [1, 2, 4, 8] {
        let stored_capacity = max_elements + 1;
        let left = base_set(24, 0xC001_D00D ^ max_elements as u64);

        for difference in 0..=max_elements {
            for case in 0_u64..4 {
                let remove = difference / 2;
                let add = difference - remove;
                let right = mutate(
                    &left,
                    remove,
                    add,
                    0xD1FF_0000 ^ max_elements as u64 ^ difference as u64 ^ case.rotate_left(17),
                    false,
                );
                let expected = symmetric_difference(&left, &right);
                assert_eq!(expected.len(), difference);

                let mut combined =
                    PinSketch64Lab::from_sorted_unique(stored_capacity, &left).unwrap();
                let other = PinSketch64Lab::from_sorted_unique(stored_capacity, &right).unwrap();
                combined.merge(&other).unwrap();

                assert_eq!(
                    combined.decode_candidate_with_limit(max_elements).unwrap(),
                    expected,
                    "stored_capacity={stored_capacity} max_elements={max_elements} d={difference} case={case}"
                );
            }
        }
    }
}

#[test]
fn fixed_tiny_vectors_are_stable() {
    let one = PinSketch64Lab::from_sorted_unique(1, &[1]).unwrap();
    assert_eq!(one.odd_syndromes(), &[1]);
    assert_eq!(
        one.encode(),
        [
            b'D', b'M', b'P', b'6', b'4', b'L', b'0', b'1', 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0,
        ]
    );
    assert_eq!(one.decode_candidate().unwrap(), vec![1]);

    let two = PinSketch64Lab::from_sorted_unique(2, &[1, 2]).unwrap();
    assert_eq!(two.odd_syndromes(), &[3, 9]);
    assert_eq!(two.decode_candidate().unwrap(), vec![1, 2]);
}

#[test]
fn exact_merge_decode_handles_zero_and_high_bit_values() {
    let left = vec![0, 1, 1_u64 << 63, u64::MAX];
    let right = vec![1, 2, 1_u64 << 63];
    let expected = vec![0, 2, u64::MAX];

    let mut combined = PinSketch64Lab::from_sorted_unique(4, &left).unwrap();
    let other = PinSketch64Lab::from_sorted_unique(4, &right).unwrap();
    combined.merge(&other).unwrap();

    assert_eq!(combined.decode_candidate().unwrap(), expected);
}

#[test]
fn direction_is_derived_only_from_exact_local_membership() {
    let local = vec![1, 2, 5, 8, 13];
    let remote = vec![2, 3, 5, 13, 21];
    let expected = symmetric_difference(&local, &remote);

    let mut combined = PinSketch64Lab::from_sorted_unique(4, &local).unwrap();
    let other = PinSketch64Lab::from_sorted_unique(4, &remote).unwrap();
    combined.merge(&other).unwrap();
    let recovered = combined.decode_candidate().unwrap();
    assert_eq!(recovered, expected);

    let local_only: Vec<_> = recovered
        .iter()
        .copied()
        .filter(|key| local.binary_search(key).is_ok())
        .collect();
    let remote_only: Vec<_> = recovered
        .iter()
        .copied()
        .filter(|key| local.binary_search(key).is_err())
        .collect();

    assert_eq!(local_only, vec![1, 8]);
    assert_eq!(remote_only, vec![3, 21]);
}

#[test]
fn serialization_is_separate_and_merge_stable() {
    let left = PinSketch64Lab::from_sorted_unique(4, &[0, 3, 7, u64::MAX]).unwrap();
    let right = PinSketch64Lab::from_sorted_unique(4, &[3, 5, u64::MAX]).unwrap();

    let left = PinSketch64Lab::decode(&left.encode()).unwrap();
    let right = PinSketch64Lab::decode(&right.encode()).unwrap();

    let mut combined = left;
    combined.merge(&right).unwrap();
    assert_eq!(combined.decode_candidate().unwrap(), vec![0, 5, 7]);
}

#[test]
fn capacity_and_input_contract_fail_closed() {
    assert_eq!(
        PinSketch64Lab::new(0).unwrap_err(),
        LabError::InvalidCapacity
    );
    assert_eq!(
        PinSketch64Lab::from_sorted_unique(2, &[1, 1]).unwrap_err(),
        LabError::DuplicateInput
    );

    let mut one = PinSketch64Lab::new(1).unwrap();
    let two = PinSketch64Lab::new(2).unwrap();
    assert_eq!(one.merge(&two).unwrap_err(), LabError::CapacityMismatch);
}

#[test]
fn one_extra_syndrome_rejects_fixed_over_capacity_vectors() {
    for max_elements in [1, 2, 4, 8] {
        let stored_capacity = max_elements + 1;
        for case in 0_u64..32 {
            let keys = base_set(
                max_elements + 1,
                0x6A17_0000_0000_0000 ^ (max_elements as u64) << 32 ^ case,
            );
            let sketch = PinSketch64Lab::from_sorted_unique(stored_capacity, &keys).unwrap();
            assert!(
                sketch.decode_candidate_with_limit(max_elements).is_err(),
                "guarded false success: max_elements={max_elements} case={case}"
            );
        }
    }
}

#[test]
fn decode_limit_must_fit_stored_capacity() {
    let sketch = PinSketch64Lab::new(2).unwrap();
    assert_eq!(
        sketch.decode_candidate_with_limit(3).unwrap_err(),
        LabError::InvalidDecodeLimit
    );
}

#[test]
fn over_capacity_never_becomes_oracle_success() {
    for capacity in [1, 2, 4] {
        let keys: Vec<u64> = (0..=capacity)
            .map(|index| splitmix64(0xBAD0_0000 ^ index as u64 ^ capacity as u64))
            .collect();
        let mut keys = keys;
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), capacity + 1);

        let sketch = PinSketch64Lab::from_sorted_unique(capacity, &keys).unwrap();
        match sketch.decode_candidate() {
            Ok(candidate) => assert_ne!(
                candidate, keys,
                "over-capacity candidate must not be treated as exact oracle success"
            ),
            Err(
                LabError::DecodeFailure
                | LabError::CandidateExceedsCapacity
                | LabError::InvalidRoots,
            ) => {}
            Err(other) => panic!("unexpected over-capacity error: {other:?}"),
        }
    }
}

#[test]
fn encoded_size_is_fixed_by_capacity() {
    for capacity in 1..=9 {
        let sketch = PinSketch64Lab::new(capacity).unwrap();
        assert_eq!(sketch.capacity(), capacity);
        assert_eq!(sketch.odd_syndromes().len(), capacity);
        assert_eq!(
            PinSketch64Lab::encoded_len_for_capacity(capacity).unwrap(),
            16 + capacity * 8
        );
    }
}
