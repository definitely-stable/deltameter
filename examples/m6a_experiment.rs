//! M6-A private two-process estimator-assisted value experiment.
//!
//! This is an experiment harness, not a public network protocol.
//! Deterministic Energy rows make CI reproducible and are not theorem draws.

use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use deltameter::{
    EnergyConfig, EnergyDeltaMeter, EnergyProfile, EnergyRowHash, FailureTarget, RelativeError,
};

const FRAME_MAX: usize = 2 * 1024 * 1024;
const MESSAGE_MAGIC: [u8; 4] = *b"DM6A";
const FIXTURE_MAGIC: [u8; 8] = *b"DM6AFIX1";
const VERSION: u8 = 1;
const HEADER_LEN: usize = 56;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
const STALL_TIMEOUT: Duration = Duration::from_millis(150);
const DEFAULT_PEER_SIZE: usize = 8192;
const ADMISSION_THRESHOLD: u64 = 256;
const DATASET_ID: u64 = 0xD317_A620_2600_0001;
const PEER_GENERATION: u64 = 7;
const CONFIG_ID: u64 = 0xE6A0_0001_2026_1006;
const SESSION_BASE: u64 = 0x5E55_10A0_0000_0000;

const KIND_SNAPSHOT: u8 = 1;
const KIND_EXACT_SET: u8 = 2;
const KIND_READY: u8 = 128;
const KIND_ESTIMATE: u8 = 129;
const KIND_EXACT_RESULT: u8 = 130;
const KIND_ERROR: u8 = 255;

const ERR_PROTOCOL: u16 = 1;
const ERR_IDENTITY: u16 = 2;
const ERR_REPLAY: u16 = 3;
const ERR_STALE_GENERATION: u16 = 4;
const ERR_MODE: u16 = 5;
const ERR_SNAPSHOT: u16 = 6;
const ERR_EXACT_SET: u16 = 7;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Header {
    kind: u8,
    session_id: u64,
    dataset_id: u64,
    source_generation: u64,
    peer_generation: u64,
    config_id: u64,
    request_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeerMode {
    Direct,
    Sketch,
}

impl PeerMode {
    fn parse(value: &str) -> io::Result<Self> {
        match value {
            "direct" => Ok(Self::Direct),
            "sketch" => Ok(Self::Sketch),
            _ => Err(invalid_data("peer mode must be direct or sketch")),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Sketch => "sketch",
        }
    }
}

#[derive(Debug)]
struct Fixture {
    dataset_id: u64,
    peer_generation: u64,
    config_id: u64,
    keys: Vec<u64>,
}

#[derive(Debug, Clone, Copy)]
struct EstimateResponse {
    point: u128,
    threshold: u64,
    admit: bool,
}

#[derive(Debug, Clone, Copy)]
struct Workload {
    name: &'static str,
    remove: usize,
    add: usize,
}

struct ScenarioData<'a> {
    name: &'a str,
    source_generation: u64,
    source: &'a [u64],
    snapshot: &'a [u8],
    peer_n: usize,
    exact_d: u64,
    source_build_ns: u64,
}

#[derive(Debug)]
struct TempFixture {
    path: PathBuf,
}

impl TempFixture {
    fn create(keys: &[u64]) -> io::Result<Self> {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "deltameter-m6a-{}-{sequence}.bin",
            std::process::id()
        ));
        write_fixture(&path, keys)?;
        Ok(Self { path })
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct PeerProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    rx: Receiver<io::Result<(Vec<u8>, usize)>>,
    session_id: u64,
    dataset_id: u64,
    peer_generation: u64,
    config_id: u64,
    next_request_id: u64,
    tx_bytes: u64,
    rx_bytes: u64,
    round_trips: u64,
    setup_ns: u64,
    alive: bool,
}

impl PeerProcess {
    fn spawn(fixture: &Path, session_id: u64, mode: PeerMode) -> io::Result<Self> {
        let exe = env::current_exe()?;
        let mut child = Command::new(exe)
            .arg("--peer")
            .arg(fixture)
            .arg(session_id.to_string())
            .arg(mode.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| other_error("peer stdin was not piped"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| other_error("peer stdout was not piped"))?;
        let rx = spawn_frame_reader(stdout);

        let mut peer = Self {
            child,
            stdin: Some(stdin),
            rx,
            session_id,
            dataset_id: DATASET_ID,
            peer_generation: PEER_GENERATION,
            config_id: CONFIG_ID,
            next_request_id: 1,
            tx_bytes: 0,
            rx_bytes: 0,
            round_trips: 0,
            setup_ns: 0,
            alive: true,
        };

        let (payload, bytes) = peer.recv_frame(RESPONSE_TIMEOUT)?;
        peer.rx_bytes = peer
            .rx_bytes
            .checked_add(bytes as u64)
            .ok_or_else(|| other_error("rx byte counter overflow"))?;
        let (header, body) = decode_message(&payload)?;
        if header.kind != KIND_READY
            || header.session_id != session_id
            || header.dataset_id != DATASET_ID
            || header.peer_generation != PEER_GENERATION
            || header.config_id != CONFIG_ID
            || header.request_id != 0
        {
            peer.kill_and_wait();
            return Err(invalid_data("invalid peer ready response"));
        }
        let mut cursor = Cursor::new(body);
        peer.setup_ns = cursor.read_u64()?;
        let mode_tag = cursor.read_u8()?;
        if !cursor.finished()
            || mode_tag
                != match mode {
                    PeerMode::Direct => 0,
                    PeerMode::Sketch => 1,
                }
        {
            peer.kill_and_wait();
            return Err(invalid_data("invalid peer ready body"));
        }

        Ok(peer)
    }

    fn request(
        &mut self,
        kind: u8,
        source_generation: u64,
        body: &[u8],
    ) -> io::Result<(Header, Vec<u8>)> {
        let request_id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or_else(|| other_error("request id overflow"))?;
        self.request_with_header(
            Header {
                kind,
                session_id: self.session_id,
                dataset_id: self.dataset_id,
                source_generation,
                peer_generation: self.peer_generation,
                config_id: self.config_id,
                request_id,
            },
            body,
        )
    }

    fn request_with_header(
        &mut self,
        header: Header,
        body: &[u8],
    ) -> io::Result<(Header, Vec<u8>)> {
        let payload = encode_message(header, body)?;
        let written = write_frame(
            self.stdin
                .as_mut()
                .ok_or_else(|| other_error("peer stdin is closed"))?,
            &payload,
        )?;
        self.tx_bytes = self
            .tx_bytes
            .checked_add(written as u64)
            .ok_or_else(|| other_error("tx byte counter overflow"))?;
        self.round_trips = self
            .round_trips
            .checked_add(1)
            .ok_or_else(|| other_error("round-trip counter overflow"))?;

        let (response, read) = self.recv_frame(RESPONSE_TIMEOUT)?;
        self.rx_bytes = self
            .rx_bytes
            .checked_add(read as u64)
            .ok_or_else(|| other_error("rx byte counter overflow"))?;
        let (response_header, response_body) = decode_message(&response)?;

        if response_header.session_id != header.session_id
            || response_header.dataset_id != header.dataset_id
            || response_header.source_generation != header.source_generation
            || response_header.peer_generation != header.peer_generation
            || response_header.config_id != header.config_id
            || response_header.request_id != header.request_id
        {
            return Err(invalid_data("peer response identity mismatch"));
        }

        Ok((response_header, response_body.to_vec()))
    }

    fn recv_frame(&mut self, timeout: Duration) -> io::Result<(Vec<u8>, usize)> {
        match self.rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                self.kill_and_wait();
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "peer response timed out",
                ))
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.kill_and_wait();
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "peer response channel disconnected",
                ))
            }
        }
    }

    fn counters(&self) -> (u64, u64, u64) {
        (self.tx_bytes, self.rx_bytes, self.round_trips)
    }

    fn close(&mut self) -> io::Result<()> {
        self.stdin.take();
        if self.alive {
            let status = self.child.wait()?;
            self.alive = false;
            if !status.success() {
                return Err(other_error("peer exited unsuccessfully"));
            }
        }
        Ok(())
    }

    fn kill_and_wait(&mut self) {
        self.stdin.take();
        if self.alive {
            let _ = self.child.kill();
            let _ = self.child.wait();
            self.alive = false;
        }
    }
}

impl Drop for PeerProcess {
    fn drop(&mut self) {
        self.kill_and_wait();
    }
}

fn main() -> AppResult<()> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--peer") => peer_entry(&args[2..]),
        Some("--stall-peer") => {
            thread::sleep(Duration::from_secs(60));
            Ok(())
        }
        Some(other) => Err(format!("unknown argument {other}").into()),
        None => harness_main(),
    }
}

fn peer_entry(args: &[String]) -> AppResult<()> {
    if args.len() != 3 {
        return Err("usage: --peer FIXTURE SESSION_ID direct|sketch".into());
    }
    let fixture = Path::new(&args[0]);
    let session_id = args[1].parse::<u64>()?;
    let mode = PeerMode::parse(&args[2])?;
    peer_main(fixture, session_id, mode)?;
    Ok(())
}

fn peer_main(path: &Path, session_id: u64, mode: PeerMode) -> io::Result<()> {
    let started = Instant::now();
    let fixture = read_fixture(path)?;
    validate_canonical_set(&fixture.keys)?;

    let local_meter = if mode == PeerMode::Sketch {
        Some(build_meter(&fixture.keys, fixture.config_id)?)
    } else {
        None
    };
    let setup_ns = nanos_u64(started.elapsed())?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    let ready_header = Header {
        kind: KIND_READY,
        session_id,
        dataset_id: fixture.dataset_id,
        source_generation: 0,
        peer_generation: fixture.peer_generation,
        config_id: fixture.config_id,
        request_id: 0,
    };
    let mut ready_body = Vec::with_capacity(9);
    push_u64(&mut ready_body, setup_ns);
    ready_body.push(match mode {
        PeerMode::Direct => 0,
        PeerMode::Sketch => 1,
    });
    let ready = encode_message(ready_header, &ready_body)?;
    write_frame(&mut writer, &ready)?;

    let mut last_request_id = 0_u64;
    let mut max_source_generation = 0_u64;

    while let Some((payload, _)) = read_frame(&mut reader)? {
        let (header, body) = match decode_message(&payload) {
            Ok(value) => value,
            Err(_) => break,
        };

        let identity_ok = header.session_id == session_id
            && header.dataset_id == fixture.dataset_id
            && header.peer_generation == fixture.peer_generation
            && header.config_id == fixture.config_id;

        if !identity_ok {
            send_error(&mut writer, header, ERR_IDENTITY)?;
            continue;
        }
        if header.request_id == 0 || header.request_id <= last_request_id {
            send_error(&mut writer, header, ERR_REPLAY)?;
            continue;
        }
        if header.source_generation == 0 || header.source_generation < max_source_generation {
            send_error(&mut writer, header, ERR_STALE_GENERATION)?;
            continue;
        }

        last_request_id = header.request_id;
        max_source_generation = max_source_generation.max(header.source_generation);

        match header.kind {
            KIND_SNAPSHOT => {
                let Some(local_meter) = local_meter.as_ref() else {
                    send_error(&mut writer, header, ERR_MODE)?;
                    continue;
                };
                match handle_snapshot(local_meter, body) {
                    Ok(estimate) => send_estimate(&mut writer, header, estimate)?,
                    Err(_) => send_error(&mut writer, header, ERR_SNAPSHOT)?,
                }
            }
            KIND_EXACT_SET => match decode_exact_set(body) {
                Ok(source) => {
                    let d = symmetric_difference_count(&source, &fixture.keys);
                    send_exact_result(&mut writer, header, d)?;
                }
                Err(_) => send_error(&mut writer, header, ERR_EXACT_SET)?,
            },
            _ => send_error(&mut writer, header, ERR_PROTOCOL)?,
        }
    }

    writer.flush()
}

fn handle_snapshot(local_meter: &EnergyDeltaMeter, body: &[u8]) -> io::Result<EstimateResponse> {
    let mut cursor = Cursor::new(body);
    let threshold = cursor.read_u64()?;
    let snapshot_len = cursor.read_u32()? as usize;
    let snapshot = cursor.take(snapshot_len)?;
    if !cursor.finished() {
        return Err(invalid_data("trailing snapshot request bytes"));
    }

    let remote = EnergyDeltaMeter::decode_snapshot(snapshot)
        .map_err(|error| invalid_data(format!("snapshot decode failed: {error}")))?;
    let difference = remote
        .difference(local_meter)
        .map_err(|error| invalid_data(format!("snapshot config mismatch: {error}")))?;
    let point = difference.point_estimate();

    Ok(EstimateResponse {
        point,
        threshold,
        admit: point <= u128::from(threshold),
    })
}

fn send_estimate(
    writer: &mut impl Write,
    request: Header,
    estimate: EstimateResponse,
) -> io::Result<()> {
    let mut body = Vec::with_capacity(32);
    body.extend_from_slice(&estimate.point.to_le_bytes());
    push_u64(&mut body, estimate.threshold);
    body.push(u8::from(estimate.admit));
    body.extend_from_slice(&[0; 7]);
    send_response(writer, request, KIND_ESTIMATE, &body)
}

fn send_exact_result(writer: &mut impl Write, request: Header, d: u64) -> io::Result<()> {
    let mut body = Vec::with_capacity(8);
    push_u64(&mut body, d);
    send_response(writer, request, KIND_EXACT_RESULT, &body)
}

fn send_error(writer: &mut impl Write, request: Header, code: u16) -> io::Result<()> {
    send_response(writer, request, KIND_ERROR, &code.to_le_bytes())
}

fn send_response(
    writer: &mut impl Write,
    request: Header,
    kind: u8,
    body: &[u8],
) -> io::Result<()> {
    let response = encode_message(Header { kind, ..request }, body)?;
    write_frame(writer, &response)?;
    Ok(())
}

fn harness_main() -> AppResult<()> {
    let peer_keys = base_set(DEFAULT_PEER_SIZE);
    let fixture = TempFixture::create(&peer_keys)?;

    run_subprocess_guards(&fixture.path, &peer_keys)?;

    let mut direct = PeerProcess::spawn(&fixture.path, SESSION_BASE + 1, PeerMode::Direct)?;
    let mut control = PeerProcess::spawn(&fixture.path, SESSION_BASE + 2, PeerMode::Sketch)?;
    let mut admission = PeerProcess::spawn(&fixture.path, SESSION_BASE + 3, PeerMode::Sketch)?;

    println!("format=deltameter.m6a-value.v1");
    println!("contract=empirical_deterministic_custom_energy_not_coverage_proof");
    println!("peer_size={DEFAULT_PEER_SIZE}");
    println!("admission_threshold={ADMISSION_THRESHOLD}");
    println!("frame_max={FRAME_MAX}");
    println!("rtt_model=round_trips_reported_separately_from_local_elapsed");
    println!("setup,arm,peer_setup_ns,ready_rx_bytes");
    println!("setup,direct,{},{}", direct.setup_ns, direct.rx_bytes);
    println!(
        "setup,snapshot-control,{},{}",
        control.setup_ns, control.rx_bytes
    );
    println!(
        "setup,snapshot-admission,{},{}",
        admission.setup_ns, admission.rx_bytes
    );
    println!(
        "result,arm,scenario,source_n,peer_n,exact_d,threshold,point,decision,outcome,tx_bytes,rx_bytes,total_bytes,round_trips,elapsed_ns,source_sketch_build_ns,exact_bytes_avoided,false_admit,false_reject"
    );

    let workloads = [
        Workload {
            name: "d0",
            remove: 0,
            add: 0,
        },
        Workload {
            name: "d1-add",
            remove: 0,
            add: 1,
        },
        Workload {
            name: "d2",
            remove: 1,
            add: 1,
        },
        Workload {
            name: "d8",
            remove: 4,
            add: 4,
        },
        Workload {
            name: "d32",
            remove: 16,
            add: 16,
        },
        Workload {
            name: "d128",
            remove: 64,
            add: 64,
        },
        Workload {
            name: "larger128",
            remove: 0,
            add: 128,
        },
        Workload {
            name: "smaller128",
            remove: 128,
            add: 0,
        },
        Workload {
            name: "d512",
            remove: 256,
            add: 256,
        },
        Workload {
            name: "d2048",
            remove: 1024,
            add: 1024,
        },
        Workload {
            name: "disjoint",
            remove: DEFAULT_PEER_SIZE,
            add: DEFAULT_PEER_SIZE,
        },
    ];

    for (index, workload) in workloads.iter().enumerate() {
        let source_generation = (index as u64) + 1;
        let source = derive_source(&peer_keys, workload.remove, workload.add);
        let exact_d = symmetric_difference_count(&source, &peer_keys);
        assert_eq!(exact_d as usize, workload.remove + workload.add);

        let build_started = Instant::now();
        let source_meter = build_meter(&source, CONFIG_ID)?;
        let snapshot = source_meter
            .encode_snapshot()
            .map_err(|error| other_error(format!("snapshot encode failed: {error}")))?;
        let source_build_ns = nanos_u64(build_started.elapsed())?;

        let scenario = ScenarioData {
            name: workload.name,
            source_generation,
            source: &source,
            snapshot: &snapshot,
            peer_n: peer_keys.len(),
            exact_d,
            source_build_ns,
        };
        run_direct_arm(&mut direct, &scenario)?;
        run_control_arm(&mut control, &scenario)?;
        run_admission_arm(&mut admission, &scenario)?;
    }

    direct.close()?;
    control.close()?;
    admission.close()?;
    Ok(())
}

fn run_direct_arm(peer: &mut PeerProcess, scenario: &ScenarioData<'_>) -> io::Result<()> {
    let before = peer.counters();
    let body = encode_exact_set(scenario.source)?;
    let started = Instant::now();
    let (header, response) = peer.request(KIND_EXACT_SET, scenario.source_generation, &body)?;
    let elapsed = nanos_u64(started.elapsed())?;
    if header.kind != KIND_EXACT_RESULT {
        return Err(invalid_data(
            "direct exact request did not return exact result",
        ));
    }
    let returned = decode_exact_result(&response)?;
    if returned != scenario.exact_d {
        return Err(invalid_data("direct exact result disagrees with oracle"));
    }
    print_result(
        "direct-exact",
        scenario.name,
        scenario.source.len(),
        scenario.peer_n,
        scenario.exact_d,
        "",
        "",
        "exact",
        delta_counters(before, peer.counters())?,
        elapsed,
        scenario.source_build_ns,
        0,
        false,
        false,
    );
    Ok(())
}

fn run_control_arm(peer: &mut PeerProcess, scenario: &ScenarioData<'_>) -> io::Result<()> {
    let before = peer.counters();
    let started = Instant::now();
    let estimate = request_estimate(peer, scenario.source_generation, scenario.snapshot)?;
    let exact_body = encode_exact_set(scenario.source)?;
    let (header, response) =
        peer.request(KIND_EXACT_SET, scenario.source_generation, &exact_body)?;
    let elapsed = nanos_u64(started.elapsed())?;
    if header.kind != KIND_EXACT_RESULT || decode_exact_result(&response)? != scenario.exact_d {
        return Err(invalid_data(
            "snapshot control exact result disagrees with oracle",
        ));
    }
    print_result(
        "snapshot-control",
        scenario.name,
        scenario.source.len(),
        scenario.peer_n,
        scenario.exact_d,
        &estimate.point.to_string(),
        if estimate.admit { "admit" } else { "reject" },
        "exact",
        delta_counters(before, peer.counters())?,
        elapsed,
        scenario.source_build_ns,
        0,
        estimate.admit && scenario.exact_d > ADMISSION_THRESHOLD,
        !estimate.admit && scenario.exact_d <= ADMISSION_THRESHOLD,
    );
    Ok(())
}

fn run_admission_arm(peer: &mut PeerProcess, scenario: &ScenarioData<'_>) -> io::Result<()> {
    let before = peer.counters();
    let started = Instant::now();
    let estimate = request_estimate(peer, scenario.source_generation, scenario.snapshot)?;
    let exact_bytes = exact_exchange_bytes(scenario.source.len())?;

    let outcome = if estimate.admit {
        let exact_body = encode_exact_set(scenario.source)?;
        let (header, response) =
            peer.request(KIND_EXACT_SET, scenario.source_generation, &exact_body)?;
        if header.kind != KIND_EXACT_RESULT || decode_exact_result(&response)? != scenario.exact_d {
            return Err(invalid_data("admitted exact result disagrees with oracle"));
        }
        "exact"
    } else {
        "rejected"
    };
    let elapsed = nanos_u64(started.elapsed())?;

    print_result(
        "snapshot-admission",
        scenario.name,
        scenario.source.len(),
        scenario.peer_n,
        scenario.exact_d,
        &estimate.point.to_string(),
        if estimate.admit { "admit" } else { "reject" },
        outcome,
        delta_counters(before, peer.counters())?,
        elapsed,
        scenario.source_build_ns,
        if estimate.admit { 0 } else { exact_bytes },
        estimate.admit && scenario.exact_d > ADMISSION_THRESHOLD,
        !estimate.admit && scenario.exact_d <= ADMISSION_THRESHOLD,
    );
    Ok(())
}

fn request_estimate(
    peer: &mut PeerProcess,
    source_generation: u64,
    snapshot: &[u8],
) -> io::Result<EstimateResponse> {
    let snapshot_len =
        u32::try_from(snapshot.len()).map_err(|_| invalid_data("snapshot too large"))?;
    let mut body = Vec::with_capacity(12 + snapshot.len());
    push_u64(&mut body, ADMISSION_THRESHOLD);
    push_u32(&mut body, snapshot_len);
    body.extend_from_slice(snapshot);

    let (header, response) = peer.request(KIND_SNAPSHOT, source_generation, &body)?;
    if header.kind == KIND_ERROR {
        return Err(invalid_data(format!(
            "snapshot request rejected with error {}",
            decode_error_code(&response)?
        )));
    }
    if header.kind != KIND_ESTIMATE {
        return Err(invalid_data(
            "snapshot request returned unexpected response",
        ));
    }
    decode_estimate(&response)
}

#[allow(clippy::too_many_arguments)]
fn print_result(
    arm: &str,
    scenario: &str,
    source_n: usize,
    peer_n: usize,
    exact_d: u64,
    point: &str,
    decision: &str,
    outcome: &str,
    counters: (u64, u64, u64),
    elapsed_ns: u64,
    source_build_ns: u64,
    exact_bytes_avoided: u64,
    false_admit: bool,
    false_reject: bool,
) {
    let (tx, rx, round_trips) = counters;
    println!(
        "result,{arm},{scenario},{source_n},{peer_n},{exact_d},{ADMISSION_THRESHOLD},{point},{decision},{outcome},{tx},{rx},{},{round_trips},{elapsed_ns},{source_build_ns},{exact_bytes_avoided},{false_admit},{false_reject}",
        tx + rx
    );
}

fn run_subprocess_guards(path: &Path, peer_keys: &[u64]) -> io::Result<()> {
    let mut peer = PeerProcess::spawn(path, SESSION_BASE + 99, PeerMode::Direct)?;
    let body = encode_exact_set(peer_keys)?;

    let valid = Header {
        kind: KIND_EXACT_SET,
        session_id: peer.session_id,
        dataset_id: peer.dataset_id,
        source_generation: 1,
        peer_generation: peer.peer_generation,
        config_id: peer.config_id,
        request_id: 1,
    };
    let wrong_identity = Header {
        dataset_id: peer.dataset_id ^ 1,
        ..valid
    };
    let (header, error) = peer.request_with_header(wrong_identity, &body)?;
    assert_error(header, &error, ERR_IDENTITY)?;

    let (header, exact) = peer.request_with_header(valid, &body)?;
    if header.kind != KIND_EXACT_RESULT || decode_exact_result(&exact)? != 0 {
        return Err(invalid_data("valid guard exact request failed"));
    }

    let (header, error) = peer.request_with_header(valid, &body)?;
    assert_error(header, &error, ERR_REPLAY)?;

    let gen2 = Header {
        source_generation: 2,
        request_id: 2,
        ..valid
    };
    let (header, exact) = peer.request_with_header(gen2, &body)?;
    if header.kind != KIND_EXACT_RESULT || decode_exact_result(&exact)? != 0 {
        return Err(invalid_data("generation-2 guard exact request failed"));
    }

    let stale = Header {
        source_generation: 1,
        request_id: 3,
        ..valid
    };
    let (header, error) = peer.request_with_header(stale, &body)?;
    assert_error(header, &error, ERR_STALE_GENERATION)?;
    peer.close()?;

    verify_timeout_cleanup()?;
    Ok(())
}

fn verify_timeout_cleanup() -> io::Result<()> {
    let exe = env::current_exe()?;
    let mut child = Command::new(exe)
        .arg("--stall-peer")
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| other_error("stall peer stdout was not piped"))?;
    let rx = spawn_frame_reader(stdout);

    match rx.recv_timeout(STALL_TIMEOUT) {
        Err(RecvTimeoutError::Timeout) => {
            child.kill()?;
            let _ = child.wait()?;
            Ok(())
        }
        Ok(_) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(invalid_data("stall peer unexpectedly produced output"))
        }
        Err(RecvTimeoutError::Disconnected) => {
            let _ = child.wait();
            Err(invalid_data("stall peer exited before timeout"))
        }
    }
}

fn assert_error(header: Header, body: &[u8], expected: u16) -> io::Result<()> {
    if header.kind != KIND_ERROR || decode_error_code(body)? != expected {
        return Err(invalid_data("unexpected peer error response"));
    }
    Ok(())
}

fn delta_counters(before: (u64, u64, u64), after: (u64, u64, u64)) -> io::Result<(u64, u64, u64)> {
    Ok((
        after
            .0
            .checked_sub(before.0)
            .ok_or_else(|| other_error("tx counter moved backwards"))?,
        after
            .1
            .checked_sub(before.1)
            .ok_or_else(|| other_error("rx counter moved backwards"))?,
        after
            .2
            .checked_sub(before.2)
            .ok_or_else(|| other_error("round-trip counter moved backwards"))?,
    ))
}

fn build_meter(keys: &[u64], config_id: u64) -> io::Result<EnergyDeltaMeter> {
    let config = experiment_config(config_id)?;
    let mut meter = EnergyDeltaMeter::new(config)
        .map_err(|error| other_error(format!("Energy meter construction failed: {error}")))?;
    for &key in keys {
        meter
            .add_unique(key)
            .map_err(|error| other_error(format!("Energy update failed: {error}")))?;
    }
    Ok(meter)
}

fn experiment_config(config_id: u64) -> io::Result<EnergyConfig> {
    let profile = EnergyProfile::new(RelativeError::TwentyPercent, FailureTarget::OneInThousand);
    let mut rows = Vec::with_capacity(profile.tables());
    for row in 0..profile.tables() {
        let mut words = [0_u64; 6];
        for (index, word) in words.iter_mut().enumerate() {
            let ordinal = (row * 6 + index) as u64;
            *word = splitmix64(config_id ^ ordinal.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        }
        rows.push(EnergyRowHash::from_coefficients(
            [words[0], words[1]],
            [words[2], words[3], words[4], words[5]],
        ));
    }
    EnergyConfig::new(profile.buckets(), rows)
        .map_err(|error| other_error(format!("experiment config invalid: {error}")))
}

fn base_set(count: usize) -> Vec<u64> {
    let mut keys: Vec<_> = (0..count)
        .map(|index| splitmix64(0x1000_0000_0000_0000_u64 + index as u64))
        .collect();
    keys.sort_unstable();
    keys
}

fn derive_source(peer: &[u64], remove: usize, add: usize) -> Vec<u64> {
    assert!(remove <= peer.len());
    let mut keys = Vec::with_capacity(peer.len() - remove + add);
    keys.extend_from_slice(&peer[remove..]);
    keys.extend((0..add).map(|index| splitmix64(0x9000_0000_0000_0000_u64 + index as u64)));
    keys.sort_unstable();
    validate_canonical_set(&keys).expect("generated source set must be canonical");
    keys
}

fn symmetric_difference_count(left: &[u64], right: &[u64]) -> u64 {
    let mut i = 0;
    let mut j = 0;
    let mut difference = 0_u64;

    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => {
                difference += 1;
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                difference += 1;
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }
    difference + (left.len() - i) as u64 + (right.len() - j) as u64
}

fn encode_exact_set(keys: &[u64]) -> io::Result<Vec<u8>> {
    validate_canonical_set(keys)?;
    let count = u32::try_from(keys.len()).map_err(|_| invalid_data("exact set too large"))?;
    let mut body = Vec::with_capacity(4 + keys.len() * 8);
    push_u32(&mut body, count);
    for &key in keys {
        push_u64(&mut body, key);
    }
    Ok(body)
}

fn decode_exact_set(body: &[u8]) -> io::Result<Vec<u64>> {
    let mut cursor = Cursor::new(body);
    let count = cursor.read_u32()? as usize;
    let expected = count
        .checked_mul(8)
        .ok_or_else(|| invalid_data("exact set byte length overflow"))?;
    if cursor.remaining() != expected {
        return Err(invalid_data("exact set length mismatch"));
    }
    let mut keys = Vec::with_capacity(count);
    for _ in 0..count {
        keys.push(cursor.read_u64()?);
    }
    validate_canonical_set(&keys)?;
    Ok(keys)
}

fn validate_canonical_set(keys: &[u64]) -> io::Result<()> {
    if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid_data("set keys must be strictly increasing"));
    }
    Ok(())
}

fn exact_exchange_bytes(key_count: usize) -> io::Result<u64> {
    let body_len = 4_usize
        .checked_add(
            key_count
                .checked_mul(8)
                .ok_or_else(|| other_error("exact body size overflow"))?,
        )
        .ok_or_else(|| other_error("exact body size overflow"))?;
    let request = 4_usize
        .checked_add(HEADER_LEN)
        .and_then(|value| value.checked_add(body_len))
        .ok_or_else(|| other_error("exact request size overflow"))?;
    let response = 4_usize
        .checked_add(HEADER_LEN)
        .and_then(|value| value.checked_add(8))
        .ok_or_else(|| other_error("exact response size overflow"))?;
    u64::try_from(request + response).map_err(|_| other_error("exact exchange size overflow"))
}

fn write_fixture(path: &Path, keys: &[u64]) -> io::Result<()> {
    validate_canonical_set(keys)?;
    let mut writer = BufWriter::new(File::create(path)?);
    writer.write_all(&FIXTURE_MAGIC)?;
    writer.write_all(&DATASET_ID.to_le_bytes())?;
    writer.write_all(&PEER_GENERATION.to_le_bytes())?;
    writer.write_all(&CONFIG_ID.to_le_bytes())?;
    writer.write_all(
        &u64::try_from(keys.len())
            .map_err(|_| invalid_data("fixture set too large"))?
            .to_le_bytes(),
    )?;
    for &key in keys {
        writer.write_all(&key.to_le_bytes())?;
    }
    writer.flush()
}

fn read_fixture(path: &Path) -> io::Result<Fixture> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut magic = [0_u8; 8];
    reader.read_exact(&mut magic)?;
    if magic != FIXTURE_MAGIC {
        return Err(invalid_data("invalid fixture magic"));
    }
    let dataset_id = read_u64(&mut reader)?;
    let peer_generation = read_u64(&mut reader)?;
    let config_id = read_u64(&mut reader)?;
    let count = usize::try_from(read_u64(&mut reader)?)
        .map_err(|_| invalid_data("fixture count too large"))?;
    let bytes = count
        .checked_mul(8)
        .ok_or_else(|| invalid_data("fixture byte size overflow"))?;
    let mut raw = vec![0_u8; bytes];
    reader.read_exact(&mut raw)?;
    let mut trailing = [0_u8; 1];
    if reader.read(&mut trailing)? != 0 {
        return Err(invalid_data("fixture has trailing bytes"));
    }
    let mut keys = Vec::with_capacity(count);
    let (chunks, remainder) = raw.as_chunks::<8>();
    debug_assert!(remainder.is_empty());
    for chunk in chunks {
        keys.push(u64::from_le_bytes(*chunk));
    }
    validate_canonical_set(&keys)?;
    Ok(Fixture {
        dataset_id,
        peer_generation,
        config_id,
        keys,
    })
}

fn encode_message(header: Header, body: &[u8]) -> io::Result<Vec<u8>> {
    let capacity = HEADER_LEN
        .checked_add(body.len())
        .ok_or_else(|| invalid_data("message size overflow"))?;
    if capacity > FRAME_MAX {
        return Err(invalid_data("message exceeds frame limit"));
    }
    let mut payload = Vec::with_capacity(capacity);
    payload.extend_from_slice(&MESSAGE_MAGIC);
    payload.push(VERSION);
    payload.push(header.kind);
    payload.extend_from_slice(&0_u16.to_le_bytes());
    push_u64(&mut payload, header.session_id);
    push_u64(&mut payload, header.dataset_id);
    push_u64(&mut payload, header.source_generation);
    push_u64(&mut payload, header.peer_generation);
    push_u64(&mut payload, header.config_id);
    push_u64(&mut payload, header.request_id);
    payload.extend_from_slice(body);
    Ok(payload)
}

fn decode_message(payload: &[u8]) -> io::Result<(Header, &[u8])> {
    if payload.len() < HEADER_LEN {
        return Err(invalid_data("message shorter than header"));
    }
    if payload[..4] != MESSAGE_MAGIC {
        return Err(invalid_data("invalid message magic"));
    }
    if payload[4] != VERSION {
        return Err(invalid_data("unsupported message version"));
    }
    if u16::from_le_bytes([payload[6], payload[7]]) != 0 {
        return Err(invalid_data("message flags must be zero"));
    }

    let mut cursor = Cursor::new(&payload[8..HEADER_LEN]);
    let header = Header {
        kind: payload[5],
        session_id: cursor.read_u64()?,
        dataset_id: cursor.read_u64()?,
        source_generation: cursor.read_u64()?,
        peer_generation: cursor.read_u64()?,
        config_id: cursor.read_u64()?,
        request_id: cursor.read_u64()?,
    };
    Ok((header, &payload[HEADER_LEN..]))
}

fn write_frame(writer: &mut impl Write, payload: &[u8]) -> io::Result<usize> {
    if payload.len() > FRAME_MAX {
        return Err(invalid_data("frame exceeds maximum"));
    }
    let len = u32::try_from(payload.len()).map_err(|_| invalid_data("frame length overflow"))?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(4 + payload.len())
}

fn read_frame(reader: &mut impl Read) -> io::Result<Option<(Vec<u8>, usize)>> {
    let mut prefix = [0_u8; 4];
    match reader.read(&mut prefix[..1])? {
        0 => return Ok(None),
        1 => reader.read_exact(&mut prefix[1..])?,
        _ => unreachable!(),
    }
    let len = u32::from_le_bytes(prefix) as usize;
    if len > FRAME_MAX {
        return Err(invalid_data("incoming frame exceeds maximum"));
    }
    let mut payload = vec![0_u8; len];
    reader.read_exact(&mut payload)?;
    Ok(Some((payload, 4 + len)))
}

fn spawn_frame_reader(stdout: std::process::ChildStdout) -> Receiver<io::Result<(Vec<u8>, usize)>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            match read_frame(&mut reader) {
                Ok(Some(frame)) => {
                    if tx.send(Ok(frame)).is_err() {
                        break;
                    }
                }
                Ok(None) => {
                    let _ = tx.send(Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "peer stdout closed",
                    )));
                    break;
                }
                Err(error) => {
                    let _ = tx.send(Err(error));
                    break;
                }
            }
        }
    });
    rx
}

fn decode_estimate(body: &[u8]) -> io::Result<EstimateResponse> {
    let mut cursor = Cursor::new(body);
    let point = cursor.read_u128()?;
    let threshold = cursor.read_u64()?;
    let admit = match cursor.read_u8()? {
        0 => false,
        1 => true,
        _ => return Err(invalid_data("invalid admission flag")),
    };
    let reserved = cursor.take(7)?;
    if reserved != [0; 7] || !cursor.finished() {
        return Err(invalid_data("invalid estimate response padding"));
    }
    Ok(EstimateResponse {
        point,
        threshold,
        admit,
    })
}

fn decode_exact_result(body: &[u8]) -> io::Result<u64> {
    let mut cursor = Cursor::new(body);
    let value = cursor.read_u64()?;
    if !cursor.finished() {
        return Err(invalid_data("exact result has trailing bytes"));
    }
    Ok(value)
}

fn decode_error_code(body: &[u8]) -> io::Result<u16> {
    if body.len() != 2 {
        return Err(invalid_data("error response must contain one u16 code"));
    }
    Ok(u16::from_le_bytes([body[0], body[1]]))
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn read_u64(reader: &mut impl Read) -> io::Result<u64> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn nanos_u64(duration: Duration) -> io::Result<u64> {
    u64::try_from(duration.as_nanos()).map_err(|_| other_error("duration exceeds u64 nanoseconds"))
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn other_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn finished(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| invalid_data("cursor overflow"))?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid_data("message truncated"))?;
        self.position = end;
        Ok(bytes)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .expect("cursor returned four bytes"),
        ))
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?
                .try_into()
                .expect("cursor returned eight bytes"),
        ))
    }

    fn read_u128(&mut self) -> io::Result<u128> {
        Ok(u128::from_le_bytes(
            self.take(16)?
                .try_into()
                .expect("cursor returned sixteen bytes"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_round_trip_preserves_identity() {
        let header = Header {
            kind: KIND_EXACT_SET,
            session_id: 1,
            dataset_id: 2,
            source_generation: 3,
            peer_generation: 4,
            config_id: 5,
            request_id: 6,
        };
        let encoded = encode_message(header, b"body").unwrap();
        let (decoded, body) = decode_message(&encoded).unwrap();
        assert_eq!(decoded, header);
        assert_eq!(body, b"body");
    }

    #[test]
    fn exact_set_requires_strict_sorting() {
        assert!(decode_exact_set(&encode_exact_set(&[1, 2, 3]).unwrap()).is_ok());

        let mut duplicate = Vec::new();
        push_u32(&mut duplicate, 2);
        push_u64(&mut duplicate, 7);
        push_u64(&mut duplicate, 7);
        assert!(decode_exact_set(&duplicate).is_err());
    }

    #[test]
    fn frame_rejects_oversize_and_truncation() {
        let mut oversize = Vec::new();
        oversize.extend_from_slice(&((FRAME_MAX as u32) + 1).to_le_bytes());
        assert!(read_frame(&mut oversize.as_slice()).is_err());

        let mut truncated = Vec::new();
        truncated.extend_from_slice(&4_u32.to_le_bytes());
        truncated.extend_from_slice(&[1, 2]);
        assert!(read_frame(&mut truncated.as_slice()).is_err());
    }

    #[test]
    fn generated_workload_has_declared_difference() {
        let peer = base_set(128);
        for (remove, add) in [(0, 0), (0, 1), (1, 1), (16, 16), (128, 128)] {
            let source = derive_source(&peer, remove, add);
            assert_eq!(
                symmetric_difference_count(&source, &peer),
                (remove + add) as u64
            );
        }
    }

    #[test]
    fn exact_exchange_accounting_matches_frame_layout() {
        let keys = 10;
        let request = 4 + HEADER_LEN + 4 + keys * 8;
        let response = 4 + HEADER_LEN + 8;
        assert_eq!(
            exact_exchange_bytes(keys).unwrap(),
            (request + response) as u64
        );
    }
}
