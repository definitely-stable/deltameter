//! Private untimed D13-A worker. Control stdin/stdout is not the modeled wire.
//! Frozen historical modules include controls unused by this adapter; their
//! complete tests remain active in the ordinary Rust workflow.
#[allow(dead_code)]
#[path = "support/m6d11_fixed_reduction.rs"]
mod fixed_reduction;
#[allow(dead_code)]
#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[allow(dead_code)]
#[path = "support/m6d6_quadratic.rs"]
mod quadratic;
#[allow(dead_code)]
#[path = "support/m6d4_trace_square.rs"]
mod trace_square;

use pinsketch64::PinSketch64Lab;
use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};

fn keys(text: &str) -> Vec<u64> {
    let result: Vec<_> = if text == "-" {
        Vec::new()
    } else {
        text.split(',')
            .map(|x| u64::from_str_radix(x, 16).unwrap())
            .collect()
    };
    assert!(result.len() <= 1_048_576);
    assert!(result.windows(2).all(|p| p[0] < p[1]));
    result
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2));
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn from_payload(payload: &[u8]) -> PinSketch64Lab {
    assert!(!payload.is_empty() && payload[0] <= 1);
    assert!((payload.len() - 1).is_multiple_of(8));
    let capacity = (payload.len() - 1) / 8;
    let mut encoded = b"DMP64L01".to_vec();
    encoded.extend_from_slice(&(capacity as u16).to_le_bytes());
    encoded.push(payload[0]);
    encoded.extend_from_slice(&[0; 5]);
    encoded.extend_from_slice(&payload[1..]);
    PinSketch64Lab::decode(&encoded).unwrap()
}

fn payload(sketch: &PinSketch64Lab, capacity: usize) -> Vec<u8> {
    let mut result = vec![u8::from(sketch.zero_present())];
    for word in &sketch.odd_syndromes()[..capacity] {
        result.extend_from_slice(&word.to_le_bytes());
    }
    result
}

fn main() {
    let mut a = None;
    let mut b = None;
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        assert!(line.len() <= 40_000_000);
        let parts: Vec<_> = line.split_whitespace().collect();
        let output = match parts.as_slice() {
            ["init", left, right] => {
                let left = keys(left);
                let right = keys(right);
                let mut maintained = PinSketch64Lab::from_sorted_unique(9, &right).unwrap();
                // Membership-validated changes; both insert and delete toggle once.
                let old: BTreeSet<_> = right.iter().copied().collect();
                let new: BTreeSet<_> = left.iter().copied().collect();
                for &key in old.symmetric_difference(&new) {
                    maintained.toggle(key);
                }
                assert_eq!(
                    maintained,
                    PinSketch64Lab::from_sorted_unique(9, &left).unwrap()
                );
                a = Some(maintained);
                b = Some(PinSketch64Lab::from_sorted_unique(9, &right).unwrap());
                "ready".to_string()
            }
            ["prefix", limit] => {
                let limit: usize = limit.parse().unwrap();
                assert!([1, 2, 4, 8].contains(&limit));
                hex(&payload(a.as_ref().unwrap(), limit + 1))
            }
            ["decode", limit, received] => {
                let limit: usize = limit.parse().unwrap();
                assert!([1, 2, 4, 8].contains(&limit));
                let received = unhex(received);
                assert_eq!(received.len(), 1 + 8 * (limit + 1));
                let mut difference = from_payload(&received);
                difference
                    .merge(&from_payload(&payload(b.as_ref().unwrap(), limit + 1)))
                    .unwrap();
                let decoded = trace_square::fresh_locator(&difference, limit).and_then(|locator| {
                    fixed_reduction::decode_with_locator_fixed_reduction(
                        &difference,
                        limit,
                        &locator,
                    )
                });
                match decoded {
                    Ok(roots) => {
                        let text = if roots.is_empty() {
                            "-".to_string()
                        } else {
                            roots
                                .iter()
                                .map(|r| format!("{r:x}"))
                                .collect::<Vec<_>>()
                                .join(",")
                        };
                        format!("ok {text}")
                    }
                    Err(_) => "reject".to_string(),
                }
            }
            _ => panic!("invalid harness command"),
        };
        println!("{output}");
        io::stdout().flush().unwrap();
    }
}
