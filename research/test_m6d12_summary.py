import unittest
from collections import defaultdict

import m6d12_summary as summary


def metadata():
    return {
        "format": "deltameter.m6d12-post-d11-profile.v1",
        "contract": "frozen_m6d11_whole_decode_profile",
        "source_keys": "8192",
        "samples": "4",
        "corpora": "d4;d5;d6;d7a;d7b",
        "schedule": "1:2;2:3;4:5;8:9",
        "payloads": "17;25;41;73",
    }


def make_row(kind, corpus, scenario, d, sample, degree):
    final_k, attempts, payload = summary.expected_stage(d)
    outcome = "exact" if d <= 8 else "rejected"
    row = {
        "kind": kind,
        "corpus": corpus,
        "scenario": scenario,
        "d": d,
        "sample": sample,
        "degree": degree,
        "outcome": outcome,
        "final_k": final_k,
        "attempts": attempts,
        "rtts": attempts,
        "payload_bytes": payload,
        "control_ns": 1000 if kind == "scenario" else 0,
        "profile_wall_ns": 1020 if kind == "scenario" else 0,
        "prefix_ns": 100 if kind == "scenario" else 0,
        "locator_ns": 100 if kind == "scenario" else 0,
        "decode_wall_ns": 800 if kind == "scenario" else 0,
        "validation_ns": 10 if kind == "scenario" else 0,
        "factor_wall_ns": 760 if kind == "scenario" else 0,
        "verification_ns": 10 if kind == "scenario" else 0,
        "factor_calls": 0,
        "plan_build_calls": 0,
        "trace_attempts": 0,
        "quadratic_calls": 0,
        "self_ns": 0,
        "plan_build_ns": 0,
        "quadratic_ns": 0,
        "trace_ns": 0,
        "gcd_ns": 0,
        "division_ns": 0,
        "false_success": 0,
    }

    if kind == "degree" and degree == 2 and d >= 2:
        row.update(
            factor_calls=1,
            quadratic_calls=1,
            self_ns=30,
            quadratic_ns=25,
        )
    elif kind == "degree" and degree == 3 and d >= 3:
        row.update(
            factor_calls=1,
            plan_build_calls=1,
            trace_attempts=2,
            self_ns=400,
            plan_build_ns=20,
            trace_ns=280,
            gcd_ns=80,
            division_ns=20,
        )

    return row


def complete_rows():
    rows = []
    for corpus in summary.CORPORA:
        for scenario, d in summary.SCENARIOS.items():
            for sample in range(4):
                rows.append(make_row("scenario", corpus, scenario, d, sample, 0))
                for degree in range(1, 9):
                    rows.append(make_row("degree", corpus, scenario, d, sample, degree))
    return rows


def aggregate(rows):
    top = defaultdict(list)
    degrees = defaultdict(list)
    for _ in range(3):
        for row in rows:
            copy = row.copy()
            if copy["kind"] == "scenario":
                top[(copy["corpus"], copy["scenario"])].append(copy)
            else:
                degrees[(copy["corpus"], copy["scenario"], copy["degree"])].append(copy)
    return top, degrees


class SummaryTests(unittest.TestCase):
    def test_valid_matrix_passes(self):
        summary.validate_run(metadata(), complete_rows())

    def test_missing_row_fails(self):
        rows = complete_rows()
        rows.pop()
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_plan_count_drift_fails(self):
        rows = complete_rows()
        target = next(
            row for row in rows
            if row["kind"] == "degree"
            and row["scenario"] == "d3"
            and row["degree"] == 3
        )
        target["plan_build_calls"] = 0
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_profile_overhead_is_inconclusive(self):
        rows = complete_rows()
        for row in rows:
            if row["kind"] == "scenario" and row["scenario"] == "d8":
                row["profile_wall_ns"] = 1100
        top, degrees = aggregate(rows)
        decision = summary.evaluate_decision(top, degrees)
        self.assertEqual(
            decision["decision"], "INCONCLUSIVE_REDUCE_INSTRUMENTATION"
        )
        self.assertTrue(decision["invalid"])

    def test_trace_common_selects_trace(self):
        rows = complete_rows()
        top, degrees = aggregate(rows)
        decision = summary.evaluate_decision(top, degrees)
        self.assertEqual(decision["decision"], "GO_NARROW_PHASE_PROFILE")
        self.assertEqual(decision["selected"], "trace")
        self.assertIn("trace", decision["common"])

    def test_no_common_phase_stops(self):
        rows = complete_rows()
        for row in rows:
            if row["kind"] == "degree" and row["scenario"] == "d8":
                row["trace_ns"] = 100 if row["degree"] == 3 else 0
                row["self_ns"] = max(
                    row["self_ns"],
                    row["plan_build_ns"]
                    + row["quadratic_ns"]
                    + row["trace_ns"]
                    + row["gcd_ns"]
                    + row["division_ns"],
                )
        for row in rows:
            if row["kind"] == "scenario" and row["scenario"] == "d8":
                row["prefix_ns"] = 100
                row["locator_ns"] = 100
                row["validation_ns"] = 10
                row["verification_ns"] = 10
        top, degrees = aggregate(rows)
        decision = summary.evaluate_decision(top, degrees)
        self.assertEqual(decision["decision"], "STOP_ALGEBRAIC_MICRO_OPT")
        self.assertIsNone(decision["selected"])


if __name__ == "__main__":
    unittest.main()
