#[path = "../examples/support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[path = "../examples/support/m6d6_quadratic.rs"]
mod quadratic;
#[path = "../examples/support/m6d8_square.rs"]
mod square_candidate;
#[path = "../examples/support/m6d10_trace_internal.rs"]
mod trace_internal;
#[path = "../examples/support/m6d4_trace_square.rs"]
mod trace_square;

use pinsketch64::PinSketch64Lab;
use quadratic::decode_with_locator_quadratic;
use square_candidate::decode_with_locator_square_candidate;
use trace_internal::{
    decode_with_locator_collect, full_square_mod_replay, full_trace_replay,
    prepare_modulus_replay, reduce_unreduced_replay, square_unreduced_replay,
    trace_accumulate_replay,
};
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

fn derive_d8(base: &[u64], salt: u64, add_mask: u64) -> Vec<u64> {
    let remove = 4;
    let add = 4;
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
fn collector_and_replays_match_accepted_d8() {
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
        let right = derive_d8(&left, right_salt ^ 8, add_mask);
        let expected = symmetric_difference(&left, &right);
        assert_eq!(expected.len(), 8);

        let left_full = PinSketch64Lab::from_sorted_unique(9, &left).unwrap();
        let right_full = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();

        let mut all_replays = Vec::new();
        let mut candidate = None;

        for (limit, capacity) in STAGES {
            let sketch = difference_prefix(&left_full, &right_full, capacity);
            let locator = fresh_locator(&sketch, limit).unwrap();

            let d6 = decode_with_locator_quadratic(&sketch, limit, &locator);
            let d8 = decode_with_locator_square_candidate(&sketch, limit, &locator);
            let collected = decode_with_locator_collect(&sketch, limit, &locator);

            assert_eq!(d8, d6);
            assert_eq!(collected.result, d8);
            all_replays.extend(collected.replays);

            if let Ok(roots) = d8 {
                candidate = Some(roots);
                break;
            }
        }

        assert_eq!(candidate.unwrap(), expected);
        assert!(!all_replays.is_empty());

        for replay in &all_replays {
            assert_eq!(replay.terms.len(), 64);
            assert_eq!(full_trace_replay(replay).unwrap(), replay.trace);
            assert_eq!(trace_accumulate_replay(&replay.terms), replay.trace);

            let prepared = prepare_modulus_replay(&replay.modulus).unwrap();
            for (index, term) in replay.terms.iter().enumerate() {
                let full = full_square_mod_replay(term, &replay.modulus).unwrap();
                let unreduced = square_unreduced_replay(term).unwrap();
                let split = reduce_unreduced_replay(unreduced, &prepared).unwrap();
                assert_eq!(split, full);

                if index + 1 < replay.terms.len() {
                    assert_eq!(full, replay.terms[index + 1]);
                }
            }
        }

        // Re-collecting the same deterministic workload must produce identical replay operands.
        let sketch = difference_prefix(&left_full, &right_full, 9);
        let locator = fresh_locator(&sketch, 8).unwrap();
        let first = decode_with_locator_collect(&sketch, 8, &locator);
        let second = decode_with_locator_collect(&sketch, 8, &locator);
        assert_eq!(first.result, second.result);
        assert_eq!(first.replays, second.replays);
    }
}

#[test]
fn frozen_d4_controls_remain_reachable() {
    let sketch = PinSketch64Lab::from_sorted_unique(3, &[1, 2]).unwrap();
    let locator = fresh_locator(&sketch, 2).unwrap();
    let frozen = sketch.decode_candidate_with_limit(2).map_err(D4Error::from);
    assert_eq!(decode_with_locator_generic(&sketch, 2, &locator), frozen);
    assert_eq!(
        decode_with_locator_specialized(&sketch, 2, &locator),
        decode_with_locator_quadratic(&sketch, 2, &locator)
    );

    let polynomial = [0x0123_4567_89AB_CDEF, 0xDEAD_BEEF_CAFE_BABE];
    let modulus = [0xA5A5_5A5A_F0F0_0F0F, 0x1357_9BDF_2468_ACE0, 1];
    assert_eq!(
        generic_square_mod(&polynomial, &modulus).unwrap(),
        poly_square_mod_monic(&polynomial, &modulus).unwrap()
    );
}
