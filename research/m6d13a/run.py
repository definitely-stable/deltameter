"""Generate reproducible untimed inventory and validate complete artifacts."""
import argparse
from contextlib import ExitStack
import hashlib
import itertools
import json
from pathlib import Path
import selectors
import subprocess

from protocol import CONTRACT, require, run_session, encode_list, decode_list, Frame

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_hashes():
    paths = ['examples/m6d13a_worker.rs', '.github/workflows/m6d13a.yml']
    paths += [str(p.relative_to(ROOT)) for p in sorted(HERE.glob('*'))
              if p.suffix in ('.py', '.go', '.json')]
    frozen = json.loads((HERE / 'frozen-sources.json').read_text())
    for path, expected in frozen.items():
        require(digest(ROOT / path) == expected, f'frozen source changed: {path}')
    return {p: digest(ROOT / p) for p in sorted(set(paths) | set(frozen))}


class Worker:
    def __init__(self, path):
        self.process = subprocess.Popen([str(Path(path).resolve())], stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, text=True, bufsize=1)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)

    def request(self, line):
        require(self.process.poll() is None, 'worker exited')
        self.process.stdin.write(line + '\n')
        self.process.stdin.flush()
        require(bool(self.selector.select(60)), 'worker timeout')
        line = self.process.stdout.readline(100_000)
        require(line.endswith('\n'), 'worker EOF/oversized result')
        return line.rstrip('\n')

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.process.stdin.close()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        self.selector.close()
        self.process.stdout.close()


def splitmix64(x):
    mask = 2**64 - 1
    x = (x + 0x9E3779B97F4A7C15) & mask
    x = ((x ^ (x >> 30)) * 0xBF58476D1CE4E5B9) & mask
    x = ((x ^ (x >> 27)) * 0x94D049BB133111EB) & mask
    return x ^ (x >> 31)


def matrix():
    return list(itertools.product(CONTRACT['n'], CONTRACT['families'],
                                  CONTRACT['d'], CONTRACT['shapes']))


def fixture(n, family, d, shape):
    # Match the five D11 seed-family labels; use disjoint input ranges to the
    # bijection, so cardinality is exact without collision retries/dedup bias.
    prefix = {212: 0xD400, 213: 0xD500, 214: 0xD600, 215: 0xD700, 216: 0xD701}[family]
    salt = (prefix << 48) | 0xBA5E00000001
    remove = d if shape == 'remove' else 0 if shape == 'add' else d // 2
    add = d - remove
    b = sorted(splitmix64(salt ^ i) for i in range(n))
    a = sorted(b[remove:] + [splitmix64(salt ^ i) for i in range(n, n + add)])
    require(len(set(a) ^ set(b)) == d, 'fixture d')
    return a, b


def check_workers(pin, riblt):
    edge = [0, 1, 2**63, 2**64 - 1]
    for a, b in (([], []), (edge, []), ([], edge), (edge, edge), ([0], [])):
        for lane, worker in ((0, None), (1, pin), (2, riblt)):
            require(run_session(a, b, lane, worker)['final'] == a, 'edge oracle')
    # Deliberately exhausted real RIBLT stream; fallback must complete correctly.
    a, b = fixture(64, 212, 64, 'balanced')
    exhausted = run_session(a, b, 2, riblt, riblt_cap=1)
    require(exhausted['fallbacks'] == 1 and exhausted['rounds'] == 3, 'forced RIBLT fallback')
    return {'edge_cases': 15, 'forced_riblt_fallback': True}


def validate_report(report, expected_cases=None):
    cases = matrix() if expected_cases is None else expected_cases
    require(report['format'] == CONTRACT['format'], 'format')
    require(report['riblt_pin'] == CONTRACT['riblt_pin'], 'pin')
    require(report['source_hashes'] == source_hashes(), 'source provenance')
    require(report['performance_decision'] == 'NOT_MEASURED', 'performance claim')
    require(report['worker_checks'] == {'edge_cases': 15, 'forced_riblt_fallback': True},
            'worker controls')
    expected = {(*c, lane) for c in cases for lane in range(3)}
    seen = set()
    identities = set()
    for row in report['rows']:
        key = (*row['case'], row['lane'])
        require(key in expected and key not in seen, 'unexpected/duplicate row')
        seen.add(key)
        identity = tuple(row['session_identity'])
        require(len(identity) == 3 and all(type(x) is int and 0 < x < 2**64 for x in identity)
                and identity[0] not in identities, 'replayed session identity')
        identities.add(identity[0])
        a, b = fixture(*row['case'])
        require(row['target_sha256'] == hashlib.sha256(encode_list(a)).hexdigest(), 'target')
        require(row['result_sha256'] == row['target_sha256'], 'oracle')
        require(row['cpu_ns'] is None and row['allocated_peak_bytes'] is None, 'unmeasured')
        require(row['performance_decision'] == 'NOT_MEASURED', 'row verdict')
        trace = row['trace']
        require(len(trace) == row['messages'] == 2 * row['rounds'], 'message/round closure')
        require(len(trace) >= 4 and len(trace) % 2 == 0, 'session trace')
        require(row['bytes'] == sum(f['bytes'] for f in trace), 'byte closure')
        require(row['verification_bytes'] == sum(f['bytes'] for f in trace
                if f['phase'] == 'verification'), 'verify closure')
        require(row['candidate_bytes'] + row['verification_bytes'] == row['bytes'], 'subtotal')
        require(row['source_payload_bytes'] == 8 * (len(a) + len(b)), 'source memory')
        require(row['maintained_syndrome_payload_bytes'] == (144 if row['lane'] == 1 else 0),
                'syndrome memory')
        require(row['source_reimports_per_session'] == (2 if row['lane'] == 2 else 0), 'rescan')
        requests = []
        verification = []
        state = 'candidate'
        last_request = None
        for index in range(0, len(trace), 2):
            first, second = trace[index:index + 2]
            for seq, f in ((index, first), (index + 1, second)):
                require(f['sequence'] == seq and f['bytes'] == 48 + f['payload_bytes'], 'frame')
                require(len(f['sha256']) == 64 and all(c in '0123456789abcdef'
                        for c in f['sha256']), 'frame digest')
            require(first['direction'] == 'B>A' and second['direction'] == 'A>B', 'direction')
            if first['kind'] == 1:
                require(state in ('candidate', 'repair'), 'request after terminal verification')
                if state == 'repair':
                    require(first['parameter'] == 0, 'failed candidate requires exact repair')
                if last_request == 0:
                    raise ValueError('request after exact transfer')
                last_request = first['parameter']
                state = 'verify' if last_request == 0 else 'candidate'
                require(second['kind'] == 2 and first['parameter'] == second['parameter'], 'reply')
                require(first['payload_bytes'] == 0, 'request payload')
                require(first['phase'] == second['phase'] == 'candidate', 'data phase')
                requests.append((first['parameter'], second['payload_bytes']))
            else:
                require(state in ('candidate', 'verify') and last_request is not None,
                        'verification without new candidate')
                require(first['kind'] == 3 and second['kind'] == 4 and first['parameter'] == 0,
                        'verification kind')
                require(second['parameter'] in (0, 1) and second['payload_bytes'] == 0, 'ack')
                require(first['phase'] == second['phase'] == 'verification', 'verify phase')
                if second['parameter'] == 0:
                    require(last_request != 0 and row['lane'] != 0, 'failed exact verification')
                    state = 'repair'
                else:
                    state = 'done'
                verification.append(second['parameter'])
                if second['parameter'] == 1:
                    require(first['payload_bytes'] == 8 + 8 * len(a), 'final-list payload')
            for f in (first, second):
                if f['kind'] in (1, 4):
                    payload = b''
                elif f['kind'] == 2 and f['parameter'] == 0:
                    payload = encode_list(a)
                elif f['kind'] == 3 and second['parameter'] == 1:
                    payload = encode_list(a)
                else:
                    payload = bytes.fromhex(f['payload_hex'])
                    if f['kind'] == 3:
                        require(decode_list(payload) != a, 'failed verification payload')
                require(len(payload) == f['payload_bytes'], 'retained payload shape')
                raw = Frame(f['kind'], row['lane'], *identity, f['sequence'],
                            f['parameter'], payload).encode()
                require(hashlib.sha256(raw).hexdigest() == f['sha256'], 'frame provenance')
        require(state == 'done' and trace[-2]['kind'] == 3 and verification in ([1], [0, 1]),
                'terminal verification')
        require(row['false_candidates'] == verification.count(0), 'false candidates')
        direct = [x for x in requests if x[0] == 0]
        require(all(size == 8 + 8 * len(a) for _, size in direct), 'direct bytes')
        if row['lane'] == 0:
            require(requests == [(0, 8 + 8 * len(a))] and verification == [1], 'direct flow')
            require(row['fallbacks'] == 0, 'direct fallback')
        else:
            require(row['fallbacks'] == len(direct) <= 1, 'fallback count')
            if direct:
                require(requests[-1][0] == 0, 'fallback ordering')
            staged = [x for x in requests if x[0] != 0]
            if row['lane'] == 1:
                stages = list(zip([1, 2, 4, 8], [17, 8, 16, 32]))
                require(staged == stages[:len(staged)] and staged, 'prefix schedule')
                require(not direct or verification == [0, 1] or len(staged) == 4, 'early fallback')
            else:
                previous = 0
                for i, (target, size) in enumerate(staged):
                    require(target == 2**i and target <= 1024 and size == 24 * (target-previous),
                            'rateless schedule/batch')
                    previous = target
                require(staged and (not direct or verification == [0, 1] or previous == 1024),
                        'rateless fallback cap')
    require(seen == expected, 'missing rows')
    return 'FOUNDATION_PASS'


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--pin-worker')
    p.add_argument('--riblt-worker')
    p.add_argument('--out', type=Path)
    p.add_argument('--validate', type=Path)
    args = p.parse_args()
    if args.validate:
        print(validate_report(json.loads(args.validate.read_text())))
        return
    require(args.pin_worker and args.riblt_worker and args.out, 'worker/output arguments')
    with ExitStack() as stack:
        pin = stack.enter_context(Worker(args.pin_worker))
        riblt = stack.enter_context(Worker(args.riblt_worker))
        controls = check_workers(pin, riblt)
        rows = []
        for case in matrix():
            a, b = fixture(*case)
            for lane, worker in ((0, None), (1, pin), (2, riblt)):
                row = run_session(a, b, lane, worker)
                row['case'] = list(case)
                row['target_sha256'] = hashlib.sha256(encode_list(a)).hexdigest()
                row['result_sha256'] = hashlib.sha256(encode_list(row.pop('final'))).hexdigest()
                rows.append(row)
        report = dict(format=CONTRACT['format'], riblt_pin=CONTRACT['riblt_pin'],
                      source_hashes=source_hashes(), worker_checks=controls, rows=rows,
                      performance_decision='NOT_MEASURED')
        print(validate_report(report))
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(report, indent=2) + '\n')
        print(f'rows={len(rows)} performance_decision=NOT_MEASURED')


if __name__ == '__main__':
    main()
