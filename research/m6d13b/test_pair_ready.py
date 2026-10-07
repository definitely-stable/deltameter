import unittest

from measure import validate_go_pair_ready, validate_rust_pair_ready


class PairReadyTests(unittest.TestCase):
    def test_rust_pair_uses_distinct_left_right_lengths(self):
        line = (
            "ready source_build_ns=1 sketch_build_ns=2 clk_tck=100 "
            "source_len=11 source_a_cap=11 source_b_cap=10 sketch_payload_bytes=146 "
            "vmrss_bytes=4096 vmhwm_bytes=8192"
        )
        fields = validate_rust_pair_ready(line, "d11", 11, 10)
        self.assertEqual(fields["source_b_cap"], 10)

    def test_rust_pair_rejects_actual_b_capacity_shortfall(self):
        line = (
            "ready source_build_ns=1 sketch_build_ns=0 clk_tck=100 "
            "source_len=11 source_a_cap=11 source_b_cap=9 sketch_payload_bytes=0 "
            "vmrss_bytes=4096 vmhwm_bytes=8192"
        )
        with self.assertRaises(ValueError):
            validate_rust_pair_ready(line, "direct", 11, 10)

    def test_go_pair_uses_distinct_left_right_lengths(self):
        line = (
            "ready source_build_ns=1 clk_tck=100 source_len=11 "
            "source_a_cap=11 source_b_cap=10 runtime_alloc_bytes=1024 "
            "runtime_heap_sys_bytes=4096 vmrss_bytes=8192 vmhwm_bytes=8192"
        )
        fields = validate_go_pair_ready(line, 11, 10)
        self.assertEqual(fields["source_b_cap"], 10)


if __name__ == "__main__":
    unittest.main()
