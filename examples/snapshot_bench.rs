//! Diagnostic serialization measurements; deterministic inputs are not theorem draws.
use std::hint::black_box;
use std::time::Instant;

use deltameter::{
    EnergyConfig, EnergyDeltaMeter, EnergyProfile, EnergyRowHash, FailureTarget, ParityConfig,
    ParityDeltaMeter, RelativeError, SnapshotError,
};

fn main() {
    let samples = argument(1, 5);
    let repeats = argument(2, 16);
    assert!(
        samples > 0 && repeats > 0,
        "samples and repeats must be positive"
    );
    println!("format=deltameter.snapshot-perf.v1");
    println!("contract=diagnostic_only_deterministic_fixtures_not_coverage_evidence");
    println!("timing=encode_or_decode_including_output_drop_excluding_fixture_setup");
    println!("metric,case,content,bytes,sample,repeats,total_ns");

    for (name, profile) in [
        ("energy-default", EnergyProfile::DEFAULT),
        (
            "energy-small",
            EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand),
        ),
    ] {
        for content in ["empty", "sequential", "full-width"] {
            let rows = (0..profile.tables())
                .map(|i| {
                    let c: Vec<_> = (0..6).map(|j| mix((i * 6 + j) as u64)).collect();
                    EnergyRowHash::from_coefficients([c[0], c[1]], [c[2], c[3], c[4], c[5]])
                })
                .collect();
            // Use a custom config: a deterministic generator is not a Proven draw.
            let mut meter =
                EnergyDeltaMeter::new(EnergyConfig::new(profile.buckets(), rows).unwrap()).unwrap();
            for key in keys(content) {
                meter.add_unique(key).unwrap();
            }
            let bytes = meter.encode_snapshot().unwrap();
            measure(
                "encode",
                name,
                content,
                bytes.len(),
                samples,
                repeats,
                || {
                    drop(black_box(black_box(&meter).encode_snapshot().unwrap()));
                },
            );
            measure(
                "decode",
                name,
                content,
                bytes.len(),
                samples,
                repeats,
                || {
                    drop(black_box(
                        EnergyDeltaMeter::decode_snapshot(black_box(&bytes)).unwrap(),
                    ));
                },
            );
        }
    }
    {
        const ROWS: usize = 4095;
        let rows = (0..ROWS)
            .map(|i| {
                let c: Vec<_> = (0..6).map(|j| mix((i * 6 + j) as u64)).collect();
                EnergyRowHash::from_coefficients([c[0], c[1]], [c[2], c[3], c[4], c[5]])
            })
            .collect();
        let meter = EnergyDeltaMeter::new(EnergyConfig::new(1, rows).unwrap()).unwrap();
        let mut bytes = meter.encode_snapshot().unwrap();
        bytes[24..32].copy_from_slice(&2_u64.to_le_bytes());
        rewrite_snapshot_crc32c(&mut bytes);
        assert!(matches!(
            EnergyDeltaMeter::decode_snapshot(&bytes),
            Err(SnapshotError::InvalidPayload)
        ));
        measure(
            "decode-reject",
            "energy-valid-crc-shape",
            "bucket-count-mismatch",
            bytes.len(),
            samples,
            repeats,
            || {
                assert!(matches!(
                    EnergyDeltaMeter::decode_snapshot(black_box(&bytes)),
                    Err(SnapshotError::InvalidPayload)
                ));
            },
        );
    }

    for (name, rows, levels) in [("parity-standard", 256, 64), ("parity-padded", 17, 13)] {
        for content in ["empty", "sequential", "full-width"] {
            let mut meter = ParityDeltaMeter::new(ParityConfig::new(rows, levels, 7).unwrap());
            for key in keys(content) {
                black_box(meter.toggle(key));
            }
            let bytes = meter.encode_snapshot().unwrap();
            measure(
                "encode",
                name,
                content,
                bytes.len(),
                samples,
                repeats,
                || {
                    drop(black_box(black_box(&meter).encode_snapshot().unwrap()));
                },
            );
            measure(
                "decode",
                name,
                content,
                bytes.len(),
                samples,
                repeats,
                || {
                    drop(black_box(
                        ParityDeltaMeter::decode_snapshot(black_box(&bytes)).unwrap(),
                    ));
                },
            );
        }
    }
}

fn measure(
    metric: &str,
    case: &str,
    content: &str,
    bytes: usize,
    samples: usize,
    repeats: usize,
    mut operation: impl FnMut(),
) {
    for _ in 0..repeats {
        operation();
    }
    for sample in 0..samples {
        let start = Instant::now();
        for _ in 0..repeats {
            operation();
        }
        let elapsed = start.elapsed().as_nanos();
        println!("{metric},{case},{content},{bytes},{sample},{repeats},{elapsed}");
    }
}

fn keys(content: &str) -> impl Iterator<Item = u64> {
    let count = if content == "empty" { 0 } else { 256 };
    let full_width = content == "full-width";
    (0..count).map(move |key| if full_width { mix(key) } else { key })
}

fn rewrite_snapshot_crc32c(bytes: &mut [u8]) {
    const POLYNOMIAL: u32 = 0x82F6_3B78;

    let checksum_offset = bytes.len() - core::mem::size_of::<u32>();
    let mut crc = !0_u32;
    for &byte in &bytes[..checksum_offset] {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (POLYNOMIAL & mask);
        }
    }
    bytes[checksum_offset..].copy_from_slice(&(!crc).to_le_bytes());
}

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn argument(index: usize, default: usize) -> usize {
    std::env::args()
        .nth(index)
        .map(|s| s.parse().expect("positive integer expected"))
        .unwrap_or(default)
}
