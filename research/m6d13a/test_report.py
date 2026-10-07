import copy
import hashlib
import unittest

from protocol import CONTRACT, encode_list, run_session
from run import fixture, source_hashes, validate_report
from test_protocol import RejectWorker


class ReportTests(unittest.TestCase):
    def report(self):
        case = (64, 212, 64, 'balanced')
        a, b = fixture(*case)
        rows = []
        class RatelessReject:
            def request(self, line):
                if line.startswith('init '):
                    self.count = 0
                    return 'ready'
                if line.startswith('next '):
                    target = int(line.split()[1])
                    result = bytes(24 * (target - self.count)).hex()
                    self.count = target
                    return result
                return 'reject'
        for lane, worker in ((0, None), (1, RejectWorker()), (2, RatelessReject())):
            row = run_session(a, b, lane, worker)
            row['case'] = list(case)
            row['target_sha256'] = hashlib.sha256(encode_list(a)).hexdigest()
            row['result_sha256'] = hashlib.sha256(encode_list(row.pop('final'))).hexdigest()
            rows.append(row)
        return dict(format=CONTRACT['format'], riblt_pin=CONTRACT['riblt_pin'],
                    source_hashes=source_hashes(), rows=rows,
                    worker_checks={'edge_cases': 15, 'forced_riblt_fallback': True},
                    performance_decision='NOT_MEASURED'), [case]

    def test_complete_and_mutated_inventory(self):
        report, cases = self.report()
        self.assertEqual(validate_report(report, cases), 'FOUNDATION_PASS')
        mutations = [
            lambda r: r['rows'].pop(),
            lambda r: r['rows'].append(copy.deepcopy(r['rows'][0])),
            lambda r: r.update(riblt_pin='moving-main'),
            lambda r: r.update(source_hashes={}),
            lambda r: r.update(performance_decision='GO'),
            lambda r: r['rows'][1].update(bytes=0),
            lambda r: r['rows'][1].update(rounds=0),
            lambda r: r['rows'][1].update(cpu_ns=0),
            lambda r: r['rows'][1].update(fallbacks=0),
            lambda r: r['rows'][1]['trace'][1].update(payload_bytes=0),
            lambda r: r['rows'][2]['trace'][3].update(parameter=8),
            lambda r: r['rows'][0].update(result_sha256='0'*64),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                damaged = copy.deepcopy(report)
                mutate(damaged)
                with self.assertRaises(ValueError):
                    validate_report(damaged, cases)

    def test_failure_after_fallback_cannot_be_reverified_without_repair(self):
        report, cases = self.report()
        row = report['rows'][1]
        injected = copy.deepcopy(row['trace'][-2:])
        injected[1]['parameter'] = 0
        row['trace'][-2:-2] = injected
        for i, frame in enumerate(row['trace']):
            frame['sequence'] = i
        row['rounds'] += 1
        row['messages'] += 2
        extra = sum(f['bytes'] for f in injected)
        row['bytes'] += extra
        row['verification_bytes'] += extra
        row['false_candidates'] = 1
        with self.assertRaises(ValueError):
            validate_report(report, cases)

    def test_frame_hashes_are_evidence_not_decoration(self):
        report, cases = self.report()
        for frame in report['rows'][0]['trace']:
            frame['sha256'] = '0' * 64
        with self.assertRaises(ValueError):
            validate_report(report, cases)

    def test_full_matrix_unique_and_exact_cardinality(self):
        from run import matrix
        cases = matrix()
        self.assertEqual(len(cases), 450)
        self.assertEqual(len(set(cases)), 450)
        for n in CONTRACT['n']:
            for d in (8, 9):
                for shape in CONTRACT['shapes']:
                    a, b = fixture(n, 212, d, shape)
                    self.assertEqual(len(set(a) ^ set(b)), d)


if __name__ == '__main__':
    unittest.main()
