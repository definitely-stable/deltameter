import unittest

import m6d6_summary as summary


def metadata():
    return {
        "format": "deltameter.m6d6-quadratic.v1",
        "contract": "frozen_m6d4_degree2_only",
        "source_keys": "8192",
        "samples": "4",
        "quadratic_repeats": "128",
        "corpora": "d4;d5;d6",
        "schedule": "1:2;2:3;4:5;8:9",
        "payloads": "17;25;41;73",
    }


def total_row(corpus, scenario, d, sample):
    final_k, attempts, payload = summary.expected_stage(d)
    return {
        "kind": "total",
        "corpus": corpus,
        "scenario": scenario,
        "d": d,
        "sample": sample,
        "outcome": "exact" if d <= 8 else "rejected",
        "final_k": final_k,
        "attempts": attempts,
        "rtts": attempts,
        "payload_bytes": payload,
        "d4_ns": 1000,
        "d6_ns": 800,
        "false_success": 0,
    }


def micro_row(case, sample):
    return {
        "kind": "quadratic",
        "corpus": "micro",
        "scenario": case,
        "d": 2,
        "sample": sample,
        "outcome": "exact",
        "final_k": 2,
        "attempts": 1,
        "rtts": 0,
        "payload_bytes": 0,
        "d4_ns": 0,
        "d6_ns": 12800,
        "false_success": 0,
    }


def complete_rows():
    rows = []
    for corpus in summary.CORPORA:
        for scenario, d in summary.SCENARIOS.items():
            for sample in range(4):
                rows.append(total_row(corpus, scenario, d, sample))
    for case in summary.QUADRATICS:
        for sample in range(4):
            rows.append(micro_row(case, sample))
    return rows


class SummaryValidationTests(unittest.TestCase):
    def test_valid_matrix_passes(self):
        summary.validate_run(metadata(), complete_rows())

    def test_missing_corpus_row_fails(self):
        rows = complete_rows()
        rows.pop()
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_protocol_change_fails(self):
        rows = complete_rows()
        target = next(
            row
            for row in rows
            if row["kind"] == "total"
            and row["corpus"] == "d5"
            and row["scenario"] == "d8"
        )
        target["payload_bytes"] = 74
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_false_success_fails(self):
        rows = complete_rows()
        rows[0]["false_success"] = 1
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)

    def test_zero_timing_fails(self):
        rows = complete_rows()
        target = next(row for row in rows if row["kind"] == "total")
        target["d6_ns"] = 0
        with self.assertRaises(SystemExit):
            summary.validate_run(metadata(), rows)


if __name__ == "__main__":
    unittest.main()
