import unittest
from collections import defaultdict

import m6d10_summary as summary


def metadata():
    return {
        "format": "deltameter.m6d10-trace-internal.v1",
        "contract": "frozen_m6d8_offline_replay",
        "source_keys": "8192",
        "samples": "4",
        "repeats": "8",
        "corpora": "d4;d5;d6;d7a;d7b",
        "scenario": "d8",
        "schedule": "1:2;2:3;4:5;8:9",
        "payload_bytes": "73",
    }


def make_row(corpus, sample):
    return {
        "corpus": corpus,
        "sample": sample,
        "trace_attempts": 100,
        "term_cases": 6400,
        "repeats": 8,
        "full_trace_ns": 1000,
        "full_square_mod_ns": 850,
        "prepare_ns": 80,
        "square_build_ns": 170,
        "reduction_with_copy_ns": 650,
        "clone_ns": 50,
        "trace_accumulate_ns": 80,
    }


def complete_rows():
    return [
        make_row(corpus, sample)
        for corpus in summary.CORPORA
        for sample in range(summary.SAMPLES)
    ]


def grouped(rows):
    result = defaultdict(list)
    for _ in range(3):
        for row in rows:
            result[row["corpus"]].append(row.copy())
    return result


class SummaryTests(unittest.TestCase):
    def test_valid_matrix_passes(self):
        summary.validate_run(metadata(), complete_rows())

    def test_inventory_drift_fails(self):
        rows = complete_rows()
        rows[0]["term_cases"] += 1
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_reduction_baseline_fails_closed(self):
        rows = complete_rows()
        rows[0]["clone_ns"] = rows[0]["reduction_with_copy_ns"]
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_reduction_selector_wins(self):
        decision, _, invalid = summary.evaluate(grouped(complete_rows()))
        self.assertEqual(decision, "GO_REDUCTION_EXPERIMENT")
        self.assertEqual(invalid, [])

    def test_bad_closure_is_inconclusive(self):
        rows = complete_rows()
        for row in rows:
            row["prepare_ns"] = 400
            row["square_build_ns"] = 400
            row["reduction_with_copy_ns"] = 800
            row["clone_ns"] = 10
        decision, _, invalid = summary.evaluate(grouped(rows))
        self.assertEqual(decision, "INCONCLUSIVE_REPLAY_CLOSURE")
        self.assertTrue(invalid)

    def test_square_build_selector_when_reduction_below_threshold(self):
        rows = complete_rows()
        for row in rows:
            row["prepare_ns"] = 100
            row["square_build_ns"] = 300
            row["reduction_with_copy_ns"] = 450
            row["clone_ns"] = 100
        decision, _, invalid = summary.evaluate(grouped(rows))
        self.assertEqual(decision, "GO_SQUARE_BUILD_EXPERIMENT")
        self.assertEqual(invalid, [])


if __name__ == "__main__":
    unittest.main()
