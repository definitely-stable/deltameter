"""Private D13-A simulator. No timers; exact oracle is outside strategy selection."""
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import struct

CONTRACT = json.loads(Path(__file__).with_name('protocol.json').read_text())
HEADER = struct.Struct('<4sBBBBQQQIIQ')
MAX_KEYS = CONTRACT['max_keys']
MAX_PAYLOAD = 8 * MAX_KEYS + 8


def require(condition, message):
    if not condition:
        raise ValueError(message)


@dataclass(frozen=True)
class Frame:
    kind: int
    lane: int
    session: int
    generation_a: int
    generation_b: int
    sequence: int
    parameter: int
    payload: bytes

    def encode(self):
        require(1 <= self.kind <= 4 and 0 <= self.lane <= 2, 'kind/lane')
        require(len(self.payload) <= MAX_PAYLOAD, 'payload budget')
        return HEADER.pack(b'D13A', 1, self.kind, self.lane, 0, self.session,
                           self.generation_a, self.generation_b, self.sequence,
                           len(self.payload), self.parameter) + self.payload

    @staticmethod
    def decode(raw, expected):
        require(len(raw) >= HEADER.size, 'short header')
        fields = HEADER.unpack_from(raw)
        require(fields[9] <= MAX_PAYLOAD and len(raw) == HEADER.size + fields[9],
                'length/budget')
        require(fields == HEADER.unpack_from(expected.encode()), 'header/identity/order')
        return raw[HEADER.size:]


def encode_list(keys):
    require(len(keys) <= MAX_KEYS, 'key budget')
    require(all(type(k) is int and 0 <= k < 2**64 for k in keys), 'key domain')
    require(all(a < b for a, b in zip(keys, keys[1:])), 'canonical keys')
    return struct.pack('<Q', len(keys)) + b''.join(struct.pack('<Q', k) for k in keys)


def decode_list(raw):
    require(len(raw) >= 8, 'short list')
    count, = struct.unpack_from('<Q', raw)
    require(count <= MAX_KEYS and len(raw) == 8 + 8 * count, 'list shape/budget')
    keys = [x[0] for x in struct.iter_unpack('<Q', raw[8:])]
    require(all(a < b for a, b in zip(keys, keys[1:])), 'canonical keys')
    return keys


def key_text(keys):
    return ','.join(f'{k:x}' for k in keys) or '-'


def parse_keys(text):
    keys = [] if text == '-' else [int(k, 16) for k in text.split(',')]
    encode_list(keys)
    return keys


class Session:
    def __init__(self, lane):
        self.lane = lane
        self.sequence = 0
        self.rounds = 0
        self.trace = []

    def transfer(self, kind, parameter, payload, direction, phase):
        frame = Frame(kind, self.lane, 1, 11, 13, self.sequence, parameter, payload)
        raw = frame.encode()
        received = Frame.decode(raw, frame)
        self.trace.append(dict(sequence=self.sequence, kind=kind, parameter=parameter,
                               direction=direction, phase=phase, bytes=len(raw),
                               payload_bytes=len(received),
                               sha256=hashlib.sha256(raw).hexdigest()))
        self.sequence += 1
        return received

    def exchange(self, parameter, payload):
        self.transfer(1, parameter, b'', 'B>A', 'candidate')
        data = self.transfer(2, parameter, payload, 'A>B', 'candidate')
        self.rounds += 1
        return data

    def verify(self, candidate, target):
        # Charged protocol verification, NOT the external experimental oracle.
        received = self.transfer(3, 0, encode_list(candidate), 'B>A', 'verification')
        equal = decode_list(received) == target
        self.transfer(4, int(equal), b'', 'A>B', 'verification')
        self.rounds += 1
        return equal


def run_session(a, b, lane, worker, riblt_cap=None):
    encode_list(a)
    encode_list(b)
    require(lane in (0, 1, 2), 'lane')
    s = Session(lane)
    candidate = None
    fallbacks = 0
    false_candidates = 0
    if lane:
        require(worker.request(f'init {key_text(a)} {key_text(b)}') == 'ready', 'worker init')
    if lane == 1:
        received_prefix = b''
        for k in CONTRACT['pin_stages']:
            full_prefix = bytes.fromhex(worker.request(f'prefix {k}'))
            require(len(full_prefix) == 1 + 8 * (k + 1), 'prefix shape')
            require(full_prefix[0] in (0, 1), 'zero bit')
            require(full_prefix.startswith(received_prefix), 'nonmonotone prefix')
            increment = full_prefix[len(received_prefix):]
            received_prefix += s.exchange(k, increment)
            result = worker.request(f'decode {k} {received_prefix.hex()}')
            if result.startswith('ok '):
                roots = parse_keys(result[3:])
                require(len(roots) <= k, 'candidate capacity')
                candidate = sorted(set(b).symmetric_difference(roots))
                break
            require(result == 'reject', 'worker error')
    elif lane == 2:
        cap = CONTRACT['riblt_max_cells'] if riblt_cap is None else riblt_cap
        require(cap >= 1 and cap <= CONTRACT['riblt_max_cells'] and cap & (cap - 1) == 0,
                'cell cap')
        count = 0
        target = 1
        while target <= cap:
            payload = bytes.fromhex(worker.request(f'next {target}'))
            require(len(payload) == 24 * (target - count), 'cell batch length')
            received = s.exchange(target, payload)
            result = worker.request(f'consume {received.hex()}')
            count = target
            if result.startswith('ok '):
                # Worker supplies directional lists, membership validated here.
                remote_text, local_text = result[3:].split(' ')
                remote, local = parse_keys(remote_text), parse_keys(local_text)
                require(not set(remote) & set(b) and set(local) <= set(b), 'direction')
                require(not set(remote) & set(local), 'conflicting directions')
                candidate = sorted((set(b) - set(local)) | set(remote))
                break
            require(result == 'reject', 'worker error')
            target *= 2
    if candidate is None:
        fallbacks = int(lane != 0)
        candidate = decode_list(s.exchange(0, encode_list(a)))
    if not s.verify(candidate, a):
        false_candidates += 1
        require(lane != 0 and fallbacks == 0, 'direct/fallback verification failed')
        fallbacks += 1
        candidate = decode_list(s.exchange(0, encode_list(a)))
        require(s.verify(candidate, a), 'fallback verification failed')
    # External exact oracle used only after all strategy choices and verification.
    require(candidate == a, 'oracle mismatch')
    total = sum(f['bytes'] for f in s.trace)
    verification = sum(f['bytes'] for f in s.trace if f['phase'] == 'verification')
    return dict(lane=lane, final=candidate, bytes=total, candidate_bytes=total-verification,
                verification_bytes=verification, rounds=s.rounds, messages=len(s.trace),
                fallbacks=fallbacks, false_candidates=false_candidates, trace=s.trace,
                cpu_ns=None, allocated_peak_bytes=None, performance_decision='NOT_MEASURED',
                source_payload_bytes=8 * (len(a) + len(b)),
                maintained_syndrome_payload_bytes=144 if lane == 1 else 0,
                source_reimports_per_session=2 if lane == 2 else 0)


def model_ns(cpu_ns, rounds, byte_count, rtt_ms, bandwidth_mbps):
    require(all(x >= 0 for x in (cpu_ns, rounds, byte_count, rtt_ms)) and bandwidth_mbps > 0,
            'network model domain')
    return cpu_ns + rounds * rtt_ms * 1_000_000 + byte_count * 8 * 1000 / bandwidth_mbps
