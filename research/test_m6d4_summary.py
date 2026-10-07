"""Fail-closed evidence regression tests; fixtures are synthetic, never evidence."""
import csv
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('m6d4_summary.py')
META = '''format=deltameter.m6d4-trace-square.v1
contract=frozen_m6d2_protocol_root_trace_square_only
source_keys=8192
samples=4
square_repeats=128
schedule=1:2;2:3;4:5;8:9
payloads=17;25;41;73
'''
HEADER = 'record,kind,scenario,d,degree,outcome,operations,final_k,attempts,rtts,payload_bytes,generic_ns,specialized_ns,locator_ns,false_success'


def fixture():
    rows = []
    for degree in (2, 4, 8):
        for _ in range(4):
            rows.append(f'record,square,degree{degree},0,{degree},na,128,0,0,0,0,12800,1280,0,0')
    for d in (0, 1, 2, 3, 4, 5, 8, 9, 10, 16):
        name = 'd1-zero' if d == 1 else f'd{d}'
        k, attempts, size = (1, 1, 17) if d <= 1 else (2, 2, 25) if d == 2 else (4, 3, 41) if d <= 4 else (8, 4, 73)
        outcome = 'exact' if d <= 8 else 'rejected'
        for locator, generic in ((1, 100), (100, 1), (100, 100), (100, 100)):
            rows.append(f'record,decode,{name},{d},0,{outcome},1,{k},{attempts},{attempts},{size},{generic},10,{locator},0')
    rows += [row.replace('record,decode,', 'record,total,').rsplit(',', 2)[0] + ',0,0' for row in rows if row.startswith('record,decode,')]
    return META + HEADER + '\n' + '\n'.join(rows) + '\n'


class EvidenceTests(unittest.TestCase):
    def run_summary(self, bodies):
        with tempfile.TemporaryDirectory() as directory:
            for index, body in enumerate(bodies, 1):
                Path(directory, f'result-{index}.txt').write_text(body)
            return subprocess.run([sys.executable, str(SCRIPT), directory], capture_output=True, text=True)

    def test_valid_complete_matrix(self):
        result = self.run_summary([fixture()] * 3)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_incomplete_run_is_rejected_even_when_union_is_complete(self):
        body = fixture()
        damaged = '\n'.join(line for line in body.splitlines() if not line.startswith('record,decode,d8,')) + '\n'
        result = self.run_summary([body, damaged, body])
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('logical_gate=PASS', result.stdout)

    def test_missing_run_is_rejected(self):
        self.assertNotEqual(self.run_summary([fixture()] * 2).returncode, 0)

    def test_duplicate_row_is_rejected(self):
        body = fixture()
        self.assertNotEqual(self.run_summary([body + body.splitlines()[-1] + '\n'] * 3).returncode, 0)

    def test_invalid_metadata_and_measurements_are_rejected_before_pass(self):
        for old, new in [('source_keys=8192', 'source_keys=8'), ('12800,1280', '-1,1280'),
                         ('na,128,', 'na,0,'), ('samples=4', 'samples=1'),
                         ('12800,1280', '0,1280')]:
            with self.subTest(new=new):
                result = self.run_summary([fixture().replace(old, new)] * 3)
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn('logical_gate=PASS', result.stdout)

    def test_total_is_median_of_paired_sums(self):
        result = self.run_summary([fixture()] * 3)
        self.assertEqual(result.returncode, 0, result.stderr)
        records = list(csv.reader(io.StringIO(result.stdout)))
        header = next(row for row in records if row[:2] == ['decode', 'scenario'])
        row = next(row for row in records if row[:2] == ['decode', 'd8'])
        self.assertEqual(float(dict(zip(header, row))['generic_phase_sum_ns']), 150.5)


if __name__ == '__main__':
    unittest.main()
