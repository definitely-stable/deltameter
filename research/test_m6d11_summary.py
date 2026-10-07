import unittest
from collections import defaultdict

import m6d11_summary as summary


def metadata():
    return {
        "format": "deltameter.m6d11-fixed-reduction.v1",
        "contract": "frozen_m6d8_reduction_only",
        "source_keys": "8192",
        "samples": "4",
        "corpora": "d4;d5;d6;d7a;d7b",
        "schedule": "1:2;2:3;4:5;8:9",
        "payloads": "17;25;41;73",
    }


def make_row(corpus, scenario, sample, reduction_percent=10.0):
    d = summary.SCENARIOS[scenario]
    final_k, attempts, payload = summary.expected_stage(d)
    control = 1000
    candidate = round(control * (1.0 - reduction_percent / 100.0))
    return {
        "corpus": corpus,
        "scenario": scenario,
        "d": d,
        "sample": sample,
        "outcome": "exact" if d <= 8 else "rejected",
        "final_k": final_k,
        "attempts": attempts,
        "rtts": attempts,
        "payload_bytes": payload,
        "control_ns": control,
        "candidate_ns": candidate,
        "false_success": 0,
    }


def complete_rows(reduction_percent=10.0):
    rows = []
    for corpus in summary.CORPORA:
        for scenario in summary.SCENARIOS:
            for sample in range(4):
                rows.append(make_row(corpus, scenario, sample, reduction_percent))
    return rows


def grouped_three_runs(reduction_percent=10.0):
    totals = defaultdict(list)
    for _ in range(3):
        for row in complete_rows(reduction_percent):
            totals[(row["corpus"], row["scenario"])].append(row.copy())
    return totals


class SummaryValidationTests(unittest.TestCase):
    def test_valid_matrix_passes(self):
        summary.validate_run(metadata(), complete_rows())

    def test_missing_row_fails(self):
        rows = complete_rows()
        rows.pop()
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_false_success_fails(self):
        rows = complete_rows()
        rows[0]["false_success"] = 1
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_protocol_drift_fails(self):
        rows = complete_rows()
        rows[0]["payload_bytes"] += 8
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_frozen_performance_gate_passes_at_twenty_percent(self):
        passed, reasons = summary.evaluate_performance_gate(grouped_three_runs(20.0))
        self.assertTrue(passed)
        self.assertEqual(reasons, [])

    def test_corpus_gate_fails_below_ten_percent(self):
        totals = grouped_three_runs(20.0)
        for row in totals[("d7b", "d8")]:
            row["candidate_ns"] = 920
        passed, reasons = summary.evaluate_performance_gate(totals)
        self.assertFalse(passed)
        self.assertTrue(any("d7b/d8 median" in reason for reason in reasons))

    def test_aggregate_gate_fails_below_fifteen_percent(self):
        totals = grouped_three_runs(14.0)
        passed, reasons = summary.evaluate_performance_gate(totals)
        self.assertFalse(passed)
        self.assertTrue(any("aggregate/d8 median" in reason for reason in reasons))

    def test_guardrail_fails_below_minus_two_percent(self):
        totals = grouped_three_runs(10.0)
        for row in totals[("d4", "d5")]:
            row["candidate_ns"] = 1030
        passed, reasons = summary.evaluate_performance_gate(totals)
        self.assertFalse(passed)
        self.assertTrue(any("d4/d5 median" in reason for reason in reasons))


if __name__ == "__main__":
    unittest.main()
