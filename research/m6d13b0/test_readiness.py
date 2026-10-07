import unittest

from readiness import (
    parse_record,
    validate_check,
    validate_diagnose,
    validate_ready,
    validate_riblt_check,
    validate_riblt_ready,
    validate_riblt_sync,
    validate_riblt_update,
    validate_sync,
    validate_update,
)


class ReadinessTests(unittest.TestCase):
    def test_parse_rejects_duplicate_and_negative_fields(self):
        with self.assertRaises(ValueError):
            parse_record("sync x=1 x=2")
        with self.assertRaises(ValueError):
            parse_record("sync x=-1")

    def test_ready_contract(self):
        direct = (
            "ready source_build_ns=1 sketch_build_ns=0 clk_tck=100 "
            "source_len=3 source_a_cap=3 source_b_cap=3 sketch_payload_bytes=0 "
            "vmrss_bytes=4096 vmhwm_bytes=8192"
        )
        self.assertEqual(validate_ready(direct, "direct")["source_len"], 3)

    def test_update_phase_closure(self):
        line = (
            "update ok=1 exact_ns=7 sketch_ns=3 native_total_ns=10 cpu_ticks=0 "
            "source_len=4 source_cap=8"
        )
        self.assertEqual(validate_update(line, "d11", True)["native_total_ns"], 10)
        with self.assertRaises(ValueError):
            validate_update(line.replace("native_total_ns=10", "native_total_ns=9"), "d11", True)

    def test_direct_sync_contract(self):
        line = (
            "sync exact=1 fallback=0 final_k=0 serialize_ns=5 apply_exact_ns=7 "
            "prefix_ns=0 decode_ns=0 apply_sketch_ns=0 verification_prepare_ns=0 "
            "fallback_serialize_ns=0 fallback_apply_exact_ns=0 fallback_apply_sketch_ns=0 "
            "native_total_ns=12 candidate_capacity=0 cpu_ticks=0 clk_tck=100 payload_len=32 "
            "source_a_len=3 source_a_cap=3 source_b_len=3 source_b_cap=3 "
            "vmrss_bytes=4096 vmhwm_bytes=8192"
        )
        self.assertEqual(validate_sync(line, "direct")["native_total_ns"], 12)

    def test_d11_sync_contract(self):
        line = (
            "sync exact=1 fallback=0 final_k=1 serialize_ns=0 apply_exact_ns=2 "
            "prefix_ns=3 decode_ns=5 apply_sketch_ns=1 verification_prepare_ns=4 "
            "fallback_serialize_ns=0 fallback_apply_exact_ns=0 fallback_apply_sketch_ns=0 "
            "native_total_ns=15 candidate_capacity=4 cpu_ticks=0 clk_tck=100 payload_len=0 "
            "source_a_len=4 source_a_cap=8 source_b_len=4 source_b_cap=8 "
            "vmrss_bytes=4096 vmhwm_bytes=8192"
        )
        self.assertEqual(validate_sync(line, "d11")["final_k"], 1)


    def test_riblt_contract(self):
        ready = (
            "ready source_build_ns=1 clk_tck=100 source_len=3 source_a_cap=3 "
            "source_b_cap=3 runtime_alloc_bytes=1024 runtime_heap_sys_bytes=4096 "
            "vmrss_bytes=8192 vmhwm_bytes=8192"
        )
        self.assertEqual(validate_riblt_ready(ready)["source_len"], 3)
        update = (
            "update ok=1 exact_ns=5 native_total_ns=5 cpu_ticks=0 "
            "source_len=4 source_cap=8"
        )
        self.assertEqual(validate_riblt_update(update, True)["ok"], 1)
        sync = (
            "sync lane=1 exact=1 fallback=0 cells=3 batches=1 "
            "encoder_import_ns=2 decoder_import_ns=2 produce_ns=3 decode_ns=4 "
            "apply_ns=1 verification_prepare_ns=1 fallback_ns=0 native_total_ns=13 "
            "cpu_ticks=0 clk_tck=100 source_a_len=4 source_a_cap=8 "
            "source_b_len=4 source_b_cap=8 remote_cap=4 local_cap=4 "
            "runtime_alloc_bytes=1024 runtime_heap_sys_bytes=4096 "
            "vmrss_bytes=8192 vmhwm_bytes=8192"
        )
        self.assertEqual(validate_riblt_sync(sync, "stream")["cells"], 3)
        self.assertEqual(validate_riblt_check("check equal=1")["equal"], 1)

    def test_d11_diagnostic_requires_fresh_equivalence(self):
        line = (
            "diagnose exact_d=8 a_rebuild=1 b_rebuild=1 maintained_decoded=1 "
            "maintained_exact=1 maintained_k=8 fresh_decoded=1 fresh_exact=1 "
            "fresh_k=8 candidate_equal=1"
        )
        self.assertEqual(validate_diagnose(line)["exact_d"], 8)
        with self.assertRaises(ValueError):
            validate_diagnose(line.replace("b_rebuild=1", "b_rebuild=0"))
        with self.assertRaises(ValueError):
            validate_diagnose(line.replace("candidate_equal=1", "candidate_equal=0"))

    def test_check_requires_all_oracles(self):
        self.assertEqual(
            validate_check("check equal=1 a_rebuild=1 b_rebuild=1")["equal"], 1
        )
        with self.assertRaises(ValueError):
            validate_check("check equal=1 a_rebuild=1 b_rebuild=0")


if __name__ == "__main__":
    unittest.main()
