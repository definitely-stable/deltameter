import unittest

from model import (
    d11_wire,
    direct_wire,
    exact_exchange_bytes,
    modeled_total_ns,
    riblt_wire,
)


class SystemModelTests(unittest.TestCase):
    def test_direct_wire(self):
        sync = {"fallback": 0, "source_a_len": 10}
        self.assertEqual(exact_exchange_bytes(10), 184)
        self.assertEqual(direct_wire(sync), (184, 1))

    def test_d11_success_charges_stage_and_full_verification(self):
        sync = {
            "final_k": 1,
            "fallback": 0,
            "false_candidate": 0,
            "source_a_len": 10,
        }
        self.assertEqual(d11_wire(sync), (96 + 17 + 184, 2))

    def test_d11_false_candidate_uses_conservative_verification_bound(self):
        sync = {
            "final_k": 1,
            "fallback": 1,
            "false_candidate": 1,
            "source_a_len": 10,
        }
        self.assertEqual(d11_wire(sync), (113 + exact_exchange_bytes(11) + 184, 3))

    def test_d11_reject_through_k8_charges_all_stages_and_fallback(self):
        sync = {
            "final_k": 8,
            "fallback": 1,
            "false_candidate": 0,
            "source_a_len": 10,
        }
        stage = (96 + 17) + (96 + 8) + (96 + 16) + (96 + 32)
        self.assertEqual(d11_wire(sync), (stage + 184, 5))

    def test_riblt_pull_success(self):
        sync = {
            "cells": 5,
            "batches": 3,
            "fallback": 0,
            "source_a_len": 10,
        }
        self.assertEqual(riblt_wire(sync, "pull"), (5 * 24 + 3 * 96 + 184, 4, "none"))

    def test_riblt_stream_success(self):
        sync = {
            "cells": 5,
            "batches": 1,
            "fallback": 0,
            "source_a_len": 10,
        }
        self.assertEqual(riblt_wire(sync, "stream"), (96 + 5 * 24 + 184, 2, "none"))

    def test_riblt_cap_exhaustion_skips_candidate_verification(self):
        sync = {
            "cells": 1024,
            "batches": 11,
            "fallback": 1,
            "source_a_len": 10,
        }
        self.assertEqual(
            riblt_wire(sync, "pull"),
            (1024 * 24 + 11 * 96 + 184, 12, "exhaustion"),
        )

    def test_network_model_units(self):
        value = modeled_total_ns(
            session_native_ns=1_000_000,
            build_ns=10_000_000,
            sessions_per_build=10,
            application_bytes=1000,
            rounds=2,
            rtt_ms=5,
            bandwidth_mbps=1,
        )
        self.assertEqual(value, 20_000_000.0)


if __name__ == "__main__":
    unittest.main()
