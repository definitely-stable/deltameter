//! M6-D13-B0 private persistent measurement-readiness worker.
//!
//! This is research instrumentation only. It keeps controller I/O outside native
//! phase timers and never changes the public crate or frozen historical modules.

#[allow(dead_code)]
#[path = "support/m6d11_fixed_reduction.rs"]
mod fixed_reduction;
#[allow(dead_code)]
#[path = "support/m6d_pinsketch64.rs"]
mod pinsketch64;
#[allow(dead_code)]
#[path = "support/m6d4_trace_square.rs"]
mod trace_square;
#[allow(dead_code)]
#[path = "support/m6d6_quadratic.rs"]
mod quadratic;

use pinsketch64::PinSketch64Lab;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process::Command;
use std::time::Instant;

const CAPACITY: usize = 9;
const STAGES: [usize; 4] = [1, 2, 4, 8];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Direct,
    D11,
}

#[derive(Clone)]
struct Endpoint {
    keys: Vec<u64>,
    sketch: Option<PinSketch64Lab>,
}

impl Endpoint {
    fn exact(keys: Vec<u64>) -> Self {
        assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
        Self { keys, sketch: None }
    }

    fn build_sketch(&mut self) {
        self.sketch = Some(PinSketch64Lab::from_sorted_unique(CAPACITY, &self.keys).unwrap());
    }

    fn rebuild_matches(&self) -> bool {
        match &self.sketch {
            None => true,
            Some(sketch) => {
                PinSketch64Lab::from_sorted_unique(CAPACITY, &self.keys)
                    .map(|rebuilt| rebuilt == *sketch)
                    .unwrap_or(false)
            }
        }
    }
}

struct State {
    mode: Mode,
    a: Endpoint,
    b: Endpoint,
    clk_tck: u64,
}

fn parse_keys(text: &str) -> Vec<u64> {
    if text == "-" {
        return Vec::new();
    }
    let result: Vec<_> = text
        .split(',')
        .map(|value| u64::from_str_radix(value, 16).unwrap())
        .collect();
    assert!(result.len() <= 1_048_576);
    assert!(result.windows(2).all(|pair| pair[0] < pair[1]));
    result
}

fn encode_list(keys: &[u64]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + keys.len() * 8);
    out.extend_from_slice(&(keys.len() as u64).to_le_bytes());
    for &key in keys {
        out.extend_from_slice(&key.to_le_bytes());
    }
    out
}

fn decode_list(bytes: &[u8]) -> Vec<u64> {
    assert!(bytes.len() >= 8);
    let count = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
    assert_eq!(bytes.len(), 8 + count * 8);
    let mut keys = Vec::with_capacity(count);
    for chunk in bytes[8..].chunks_exact(8) {
        keys.push(u64::from_le_bytes(chunk.try_into().unwrap()));
    }
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    keys
}

fn prefix(sketch: &PinSketch64Lab, capacity: usize) -> PinSketch64Lab {
    let mut bytes = sketch.encode();
    bytes[8..10].copy_from_slice(&(capacity as u16).to_le_bytes());
    bytes.truncate(16 + capacity * 8);
    PinSketch64Lab::decode(&bytes).unwrap()
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

fn toggle_exact(keys: &mut Vec<u64>, values: &[u64]) {
    for &key in values {
        match keys.binary_search(&key) {
            Ok(index) => {
                keys.remove(index);
            }
            Err(index) => {
                keys.insert(index, key);
            }
        }
    }
}

fn toggle_sketch(sketch: &mut PinSketch64Lab, values: &[u64]) {
    for &key in values {
        sketch.toggle(key);
    }
}

fn process_cpu_ticks() -> u64 {
    let stat = fs::read_to_string("/proc/self/stat").unwrap();
    let close = stat.rfind(')').unwrap();
    let fields: Vec<_> = stat[close + 2..].split_whitespace().collect();
    let user: u64 = fields[11].parse().unwrap();
    let system: u64 = fields[12].parse().unwrap();
    user + system
}

fn rss_bytes() -> (u64, u64) {
    let status = fs::read_to_string("/proc/self/status").unwrap();
    let mut rss = 0;
    let mut hwm = 0;
    for line in status.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            rss = value.split_whitespace().next().unwrap().parse::<u64>().unwrap() * 1024;
        } else if let Some(value) = line.strip_prefix("VmHWM:") {
            hwm = value.split_whitespace().next().unwrap().parse::<u64>().unwrap() * 1024;
        }
    }
    (rss, hwm)
}

fn clk_tck() -> u64 {
    let output = Command::new("getconf").arg("CLK_TCK").output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().parse().unwrap()
}

fn kv(name: &str, value: impl std::fmt::Display) -> String {
    format!("{name}={value}")
}

fn sync_direct(state: &mut State) -> String {
    let cpu0 = process_cpu_ticks();
    let serialize_started = Instant::now();
    let payload = encode_list(&state.a.keys);
    let serialize_ns = serialize_started.elapsed().as_nanos();

    let apply_started = Instant::now();
    state.b.keys = decode_list(&payload);
    let apply_ns = apply_started.elapsed().as_nanos();

    assert_eq!(state.a.keys, state.b.keys);
    let native_total_ns = serialize_ns + apply_ns;
    let cpu_ticks = process_cpu_ticks() - cpu0;
    let (rss, hwm) = rss_bytes();
    [
        "sync".to_string(),
        kv("exact", 1),
        kv("fallback", 0),
        kv("final_k", 0),
        kv("serialize_ns", serialize_ns),
        kv("apply_exact_ns", apply_ns),
        kv("prefix_ns", 0),
        kv("decode_ns", 0),
        kv("apply_sketch_ns", 0),
        kv("verification_prepare_ns", 0),
        kv("fallback_serialize_ns", 0),
        kv("fallback_apply_exact_ns", 0),
        kv("fallback_apply_sketch_ns", 0),
        kv("native_total_ns", native_total_ns),
        kv("candidate_capacity", 0),
        kv("cpu_ticks", cpu_ticks),
        kv("clk_tck", state.clk_tck),
        kv("payload_len", payload.len()),
        kv("source_a_len", state.a.keys.len()),
        kv("source_a_cap", state.a.keys.capacity()),
        kv("source_b_len", state.b.keys.len()),
        kv("source_b_cap", state.b.keys.capacity()),
        kv("vmrss_bytes", rss),
        kv("vmhwm_bytes", hwm),
    ]
    .join(" ")
}

fn sync_d11(state: &mut State) -> String {
    let cpu0 = process_cpu_ticks();
    let a_sketch = state.a.sketch.as_ref().unwrap().clone();
    let b_sketch = state.b.sketch.as_ref().unwrap().clone();

    let mut prefix_ns = 0_u128;
    let mut decode_ns = 0_u128;
    let mut roots = None;
    let mut final_k = 8;

    for limit in STAGES {
        let prefix_started = Instant::now();
        let a_prefix = prefix(&a_sketch, limit + 1);
        let b_prefix = prefix(&b_sketch, limit + 1);
        prefix_ns += prefix_started.elapsed().as_nanos();

        let decode_started = Instant::now();
        let mut difference = a_prefix;
        difference.merge(&b_prefix).unwrap();
        let decoded = trace_square::fresh_locator(&difference, limit)
            .ok()
            .and_then(|locator| {
                fixed_reduction::decode_with_locator_fixed_reduction(
                    &difference,
                    limit,
                    &locator,
                )
                .ok()
            });
        decode_ns += decode_started.elapsed().as_nanos();
        final_k = limit;
        if decoded.is_some() {
            roots = decoded;
            break;
        }
    }

    let mut apply_exact_ns = 0_u128;
    let mut apply_sketch_ns = 0_u128;
    let mut verification_prepare_ns = 0_u128;
    let mut fallback_serialize_ns = 0_u128;
    let mut fallback_apply_exact_ns = 0_u128;
    let mut fallback_apply_sketch_ns = 0_u128;
    let mut fallback = 0;
    let candidate_capacity = roots.as_ref().map_or(0, |value| value.capacity());

    if let Some(candidate_roots) = roots {
        let exact_started = Instant::now();
        toggle_exact(&mut state.b.keys, &candidate_roots);
        apply_exact_ns = exact_started.elapsed().as_nanos();

        let sketch_started = Instant::now();
        toggle_sketch(state.b.sketch.as_mut().unwrap(), &candidate_roots);
        apply_sketch_ns = sketch_started.elapsed().as_nanos();

        let verify_started = Instant::now();
        let _candidate_payload = encode_list(&state.b.keys);
        let verified = state.b.keys == state.a.keys;
        verification_prepare_ns = verify_started.elapsed().as_nanos();

        if !verified {
            fallback = 1;
        }
    } else {
        fallback = 1;
    }

    if fallback != 0 {
        let serialize_started = Instant::now();
        let payload = encode_list(&state.a.keys);
        fallback_serialize_ns = serialize_started.elapsed().as_nanos();

        let exact_started = Instant::now();
        let target = decode_list(&payload);
        let delta = symmetric_difference(&state.b.keys, &target);
        state.b.keys = target;
        fallback_apply_exact_ns = exact_started.elapsed().as_nanos();

        let sketch_started = Instant::now();
        toggle_sketch(state.b.sketch.as_mut().unwrap(), &delta);
        fallback_apply_sketch_ns = sketch_started.elapsed().as_nanos();
    }

    assert_eq!(state.a.keys, state.b.keys);
    assert!(state.a.rebuild_matches());
    assert!(state.b.rebuild_matches());
    assert_eq!(state.a.sketch, state.b.sketch);

    let native_total_ns = prefix_ns
        + decode_ns
        + apply_exact_ns
        + apply_sketch_ns
        + verification_prepare_ns
        + fallback_serialize_ns
        + fallback_apply_exact_ns
        + fallback_apply_sketch_ns;
    let cpu_ticks = process_cpu_ticks() - cpu0;
    let (rss, hwm) = rss_bytes();
    [
        "sync".to_string(),
        kv("exact", 1),
        kv("fallback", fallback),
        kv("final_k", final_k),
        kv("serialize_ns", 0),
        kv("apply_exact_ns", apply_exact_ns),
        kv("prefix_ns", prefix_ns),
        kv("decode_ns", decode_ns),
        kv("apply_sketch_ns", apply_sketch_ns),
        kv("verification_prepare_ns", verification_prepare_ns),
        kv("fallback_serialize_ns", fallback_serialize_ns),
        kv("fallback_apply_exact_ns", fallback_apply_exact_ns),
        kv("fallback_apply_sketch_ns", fallback_apply_sketch_ns),
        kv("native_total_ns", native_total_ns),
        kv("candidate_capacity", candidate_capacity),
        kv("cpu_ticks", cpu_ticks),
        kv("clk_tck", state.clk_tck),
        kv("payload_len", 0),
        kv("source_a_len", state.a.keys.len()),
        kv("source_a_cap", state.a.keys.capacity()),
        kv("source_b_len", state.b.keys.len()),
        kv("source_b_cap", state.b.keys.capacity()),
        kv("vmrss_bytes", rss),
        kv("vmhwm_bytes", hwm),
    ]
    .join(" ")
}

fn main() {
    let clock = clk_tck();
    let mut state: Option<State> = None;

    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        assert!(line.len() <= 40_000_000);
        let parts: Vec<_> = line.split_whitespace().collect();
        let output = match parts.as_slice() {
            ["init", mode, keys] => {
                let mode = match *mode {
                    "direct" => Mode::Direct,
                    "d11" => Mode::D11,
                    _ => panic!("invalid mode"),
                };
                let keys = parse_keys(keys);
                let source_started = Instant::now();
                let mut a = Endpoint::exact(keys.clone());
                let mut b = Endpoint::exact(keys);
                let source_build_ns = source_started.elapsed().as_nanos();
                let sketch_started = Instant::now();
                if mode == Mode::D11 {
                    a.build_sketch();
                    b.build_sketch();
                }
                let sketch_build_ns = sketch_started.elapsed().as_nanos();
                let (rss, hwm) = rss_bytes();
                state = Some(State {
                    mode,
                    a,
                    b,
                    clk_tck: clock,
                });
                let state_ref = state.as_ref().unwrap();
                [
                    "ready".to_string(),
                    kv("source_build_ns", source_build_ns),
                    kv("sketch_build_ns", sketch_build_ns),
                    kv("clk_tck", clock),
                    kv("source_len", state_ref.a.keys.len()),
                    kv("source_a_cap", state_ref.a.keys.capacity()),
                    kv("source_b_cap", state_ref.b.keys.capacity()),
                    kv("sketch_payload_bytes", if mode == Mode::D11 { 146 } else { 0 }),
                    kv("vmrss_bytes", rss),
                    kv("vmhwm_bytes", hwm),
                ]
                .join(" ")
            }
            ["update", operation, key] => {
                let state = state.as_mut().unwrap();
                let key = u64::from_str_radix(key, 16).unwrap();
                let before_keys = state.a.keys.clone();
                let before_sketch = state.a.sketch.clone();
                let cpu0 = process_cpu_ticks();

                let exact_started = Instant::now();
                let ok = match *operation {
                    "insert" => match state.a.keys.binary_search(&key) {
                        Ok(_) => false,
                        Err(index) => {
                            state.a.keys.insert(index, key);
                            true
                        }
                    },
                    "delete" => match state.a.keys.binary_search(&key) {
                        Ok(index) => {
                            state.a.keys.remove(index);
                            true
                        }
                        Err(_) => false,
                    },
                    _ => panic!("invalid operation"),
                };
                let exact_ns = exact_started.elapsed().as_nanos();

                let sketch_started = Instant::now();
                if ok {
                    if let Some(sketch) = &mut state.a.sketch {
                        sketch.toggle(key);
                    }
                }
                let sketch_ns = sketch_started.elapsed().as_nanos();

                let ticks = process_cpu_ticks() - cpu0;
                if !ok {
                    assert_eq!(state.a.keys, before_keys);
                    assert_eq!(state.a.sketch, before_sketch);
                }
                [
                    "update".to_string(),
                    kv("ok", u8::from(ok)),
                    kv("exact_ns", exact_ns),
                    kv("sketch_ns", sketch_ns),
                    kv("native_total_ns", exact_ns + sketch_ns),
                    kv("cpu_ticks", ticks),
                    kv("source_len", state.a.keys.len()),
                    kv("source_cap", state.a.keys.capacity()),
                ]
                .join(" ")
            }
            ["sync"] => {
                let state = state.as_mut().unwrap();
                match state.mode {
                    Mode::Direct => sync_direct(state),
                    Mode::D11 => sync_d11(state),
                }
            }
            ["check"] => {
                let state = state.as_ref().unwrap();
                [
                    "check".to_string(),
                    kv("equal", u8::from(state.a.keys == state.b.keys)),
                    kv("a_rebuild", u8::from(state.a.rebuild_matches())),
                    kv("b_rebuild", u8::from(state.b.rebuild_matches())),
                ]
                .join(" ")
            }
            ["quit"] => break,
            _ => panic!("invalid command"),
        };
        println!("{output}");
        io::stdout().flush().unwrap();
    }
}
