import unittest
from collections import defaultdict

import m6d9_summary as summary


def metadata():
    return {
        "format": "deltameter.m6d9-post-d8-profile.v1",
        "contract": "frozen_m6d8_profile_only",
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
        "factor_wall_ns": 800 if kind == "scenario" else 0,
        "verification_ns": 20 if kind == "scenario" else 0,
        "factor_calls": 0,
        "trace_attempts": 0,
        "square_calls": 0,
        "coefficient_square_ops": 0,
        "reduction_mul_ops": 0,
        "quadratic_calls": 0,
        "self_ns": 0,
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
            trace_attempts=2,
            square_calls=128,
            coefficient_square_ops=192,
            reduction_mul_ops=64,
            self_ns=80,
            trace_ns=60,
            gcd_ns=15,
            division_ns=5,
        )

    return row


def complete_rows():
    rows = []
    for corpus in summary.CORPORA:
        for scenario, d in summary.SCENARIOS.items():
            for sample in range(4):
                rows.append(make_row("scenario", corpus, scenario, d, sample, 0))
                for degree in range(1, 9):
                    rows.append(
                        make_row("degree", corpus, scenario, d, sample, degree)
                    )
    return rows


class SummaryValidationTests(unittest.TestCase):
    def test_valid_matrix_passes(self):
        summary.validate_run(metadata(), complete_rows())

    def test_missing_row_fails(self):
        rows = complete_rows()
        rows.pop()
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_degree_two_trace_work_fails(self):
        rows = complete_rows()
        target = next(
            row
            for row in rows
            if row["kind"] == "degree"
            and row["corpus"] == "d4"
            and row["scenario"] == "d2"
            and row["degree"] == 2
        )
        target["trace_attempts"] = 1
        target["square_calls"] = 64
        target["coefficient_square_ops"] = 64
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_higher_degree_square_accounting_fails(self):
        rows = complete_rows()
        target = next(
            row
            for row in rows
            if row["kind"] == "degree"
            and row["corpus"] == "d4"
            and row["scenario"] == "d3"
            and row["degree"] == 3
        )
        target["square_calls"] = 127
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_false_success_fails(self):
        rows = complete_rows()
        rows[0]["false_success"] = 1
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_high_degree_phase_aggregation_sums_all_degrees(self):
        rows = complete_rows()
        tops = [
            row
            for row in rows
            if row["kind"] == "scenario"
            and row["corpus"] == "d4"
            and row["scenario"] == "d8"
        ]
        degree_rows = {
            degree: [
                row
                for row in rows
                if row["kind"] == "degree"
                and row["corpus"] == "d4"
                and row["scenario"] == "d8"
                and row["degree"] == degree
            ]
            for degree in range(3, 9)
        }

        for row in degree_rows[4]:
            row["trace_ns"] = 40
            row["gcd_ns"] = 10
            row["division_ns"] = 2

        totals = summary.aggregate_high_degree_phases(tops, degree_rows)

        self.assertEqual(totals["control_ns"], 1000)
        self.assertEqual(totals["trace_ns"], 100)
        self.assertEqual(totals["gcd_ns"], 25)
        self.assertEqual(totals["division_ns"], 7)

    def test_high_degree_phase_aggregation_fails_closed_on_missing_degree(self):
        rows = complete_rows()
        tops = [
            row
            for row in rows
            if row["kind"] == "scenario"
            and row["corpus"] == "d4"
            and row["scenario"] == "d8"
        ]
        degree_rows = {
            degree: [
                row
                for row in rows
                if row["kind"] == "degree"
                and row["corpus"] == "d4"
                and row["scenario"] == "d8"
                and row["degree"] == degree
            ]
            for degree in range(3, 8)
        }

        with self.assertRaises(SystemExit):
            summary.aggregate_high_degree_phases(tops, degree_rows)


    def test_decision_stops_when_no_common_material_phase(self):
        top = defaultdict(list)
        degrees = defaultdict(list)
        for row in complete_rows():
            if row["kind"] == "scenario":
                top[(row["corpus"], row["scenario"])].append(row)
            else:
                degrees[(row["corpus"], row["scenario"], row["degree"])].append(row)

        decision = summary.evaluate_decision(top, degrees)
        self.assertEqual(decision["decision"], "STOP_ALGEBRAIC_MICRO_OPT")
        self.assertEqual(decision["invalid"], [])

    def test_decision_authorizes_trace_internal_profile_when_common(self):
        rows = complete_rows()
        for row in rows:
            if row["kind"] == "degree" and row["scenario"] == "d8" and row["degree"] == 3:
                row["trace_ns"] = 300
                row["self_ns"] = 320

        top = defaultdict(list)
        degrees = defaultdict(list)
        for row in rows:
            if row["kind"] == "scenario":
                top[(row["corpus"], row["scenario"])].append(row)
            else:
                degrees[(row["corpus"], row["scenario"], row["degree"])].append(row)

        decision = summary.evaluate_decision(top, degrees)
        self.assertEqual(decision["decision"], "GO_TRACE_INTERNAL_PROFILE")
        self.assertIn("trace_ns", decision["common"])

    def test_profiler_overhead_makes_decision_inconclusive(self):
        rows = complete_rows()
        for row in rows:
            if row["kind"] == "scenario" and row["corpus"] == "d4" and row["scenario"] == "d8":
                row["profile_wall_ns"] = 1100

        top = defaultdict(list)
        degrees = defaultdict(list)
        for row in rows:
            if row["kind"] == "scenario":
                top[(row["corpus"], row["scenario"])].append(row)
            else:
                degrees[(row["corpus"], row["scenario"], row["degree"])].append(row)

        decision = summary.evaluate_decision(top, degrees)
        self.assertEqual(
            decision["decision"], "INCONCLUSIVE_REDUCE_INSTRUMENTATION"
        )
        self.assertTrue(decision["invalid"])

if __name__ == "__main__":
    unittest.main()
