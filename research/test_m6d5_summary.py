import unittest

import m6d5_summary as summary


def metadata():
    return {
        "format": "deltameter.m6d5-root-profile.v1",
        "contract": "frozen_m6d4_decoder_profile_only",
        "source_keys": "8192",
        "samples": "4",
        "schedule": "1:2;2:3;4:5;8:9",
        "payloads": "17;25;41;73",
    }


def row(kind, scenario, d, sample, degree):
    final_k, attempts, payload = summary.expected_stage(d)
    outcome = "exact" if d <= 8 else "rejected"
    base = {
        "kind": kind,
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
        "profile_wall_ns": 1200 if kind == "scenario" else 0,
        "factor_wall_ns": 800 if kind == "scenario" else 0,
        "verification_ns": 100 if kind == "scenario" else 0,
        "factor_calls": 0,
        "trace_attempts": 0,
        "square_calls": 0,
        "self_ns": 0,
        "trace_ns": 0,
        "gcd_ns": 0,
        "division_ns": 0,
        "false_success": 0,
    }
    if kind == "degree" and degree == 2 and d >= 2:
        base.update(
            factor_calls=1,
            trace_attempts=1,
            square_calls=64,
            self_ns=30,
            trace_ns=20,
            gcd_ns=5,
            division_ns=5,
        )
    return base


def complete_rows():
    rows = []
    for scenario, d in summary.SCENARIOS.items():
        for sample in range(4):
            rows.append(row("scenario", scenario, d, sample, 0))
            for degree in range(1, 9):
                rows.append(row("degree", scenario, d, sample, degree))
    return rows


class SummaryValidationTests(unittest.TestCase):
    def test_valid_matrix_passes(self):
        summary.validate_run(metadata(), complete_rows())

    def test_missing_degree_row_fails(self):
        rows = complete_rows()
        rows.pop()
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_square_count_mismatch_fails(self):
        rows = complete_rows()
        target = next(
            item
            for item in rows
            if item["kind"] == "degree" and item["scenario"] == "d2" and item["degree"] == 2
        )
        target["square_calls"] = 63
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_false_success_fails(self):
        rows = complete_rows()
        rows[0]["false_success"] = 1
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)


if __name__ == "__main__":
    unittest.main()
