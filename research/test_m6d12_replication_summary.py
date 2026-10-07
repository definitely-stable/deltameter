import unittest

import m6d12_replication_summary as replication
import m6d12_summary as base


def worker_summary(default=10.0, gcd=30.0, overhead=1.0):
    workers = {}
    for index in range(1, replication.EXPECTED_WORKERS + 1):
        worker = f"m6d12-worker-{index}"
        workers[worker] = {}
        for corpus in base.CORPORA:
            shares = {phase: default for phase in base.SELECTABLE_PHASES}
            shares["gcd"] = gcd
            workers[worker][corpus] = {
                "shares": shares,
                "overhead": overhead,
                "samples": 12,
            }
    return workers


class ReplicationTests(unittest.TestCase):
    def test_reproducible_gcd_selects_gcd(self):
        result = replication.evaluate_replication(worker_summary())
        self.assertEqual(result["decision"], "GO_NARROW_PHASE_PROFILE")
        self.assertEqual(result["selected"], "gcd")
        self.assertEqual(result["stable_common"], ["gcd"])

    def test_one_worker_below_boundary_stops(self):
        workers = worker_summary()
        workers["m6d12-worker-3"]["d6"]["shares"]["gcd"] = 24.9
        result = replication.evaluate_replication(workers)
        self.assertEqual(result["decision"], "STOP_ALGEBRAIC_MICRO_OPT")
        self.assertIsNone(result["selected"])
        self.assertEqual(result["stable_common"], [])

    def test_invalid_worker_is_inconclusive(self):
        workers = worker_summary()
        workers["m6d12-worker-4"]["d7a"]["overhead"] = 5.1
        result = replication.evaluate_replication(workers)
        self.assertEqual(result["decision"], "INCONCLUSIVE_REPLICATION")
        self.assertTrue(result["invalid"])

    def test_largest_stable_phase_wins(self):
        workers = worker_summary(default=10.0, gcd=31.0)
        for corpora in workers.values():
            for item in corpora.values():
                item["shares"]["trace"] = 27.0
        result = replication.evaluate_replication(workers)
        self.assertEqual(result["decision"], "GO_NARROW_PHASE_PROFILE")
        self.assertEqual(result["selected"], "gcd")
        self.assertCountEqual(result["stable_common"], ["gcd", "trace"])

    def test_wrong_worker_count_fails_closed(self):
        workers = worker_summary()
        workers.pop("m6d12-worker-5")
        with self.assertRaises(SystemExit):
            replication.evaluate_replication(workers)


if __name__ == "__main__":
    unittest.main()
