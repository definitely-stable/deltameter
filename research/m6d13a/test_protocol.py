import unittest
from protocol import Frame, Session, encode_list, decode_list, run_session, model_ns


class RejectWorker:
    def request(self, line):
        if line.startswith('init '):
            return 'ready'
        if line.startswith('prefix '):
            k = int(line.split()[1])
            return (b'\0' + bytes(8 * (k + 1))).hex()
        return 'reject'


class ProtocolTests(unittest.TestCase):
    def test_frame_rejects_every_header_corruption_and_truncation(self):
        frame = Frame(1, 1, 7, 11, 13, 0, 8, b'')
        raw = frame.encode()
        self.assertEqual(len(raw), 48)
        self.assertEqual(Frame.decode(raw, frame), b'')
        for i in range(48):
            bad = bytearray(raw)
            bad[i] ^= 1
            with self.assertRaises(ValueError):
                Frame.decode(bytes(bad), frame)
        for n in range(48):
            with self.assertRaises(ValueError):
                Frame.decode(raw[:n], frame)
        with self.assertRaises(ValueError):
            Frame.decode(raw + b'\0', frame)

    def test_canonical_lists_and_limits(self):
        keys = [0, 1 << 63, (1 << 64) - 1]
        self.assertEqual(decode_list(encode_list(keys)), keys)
        for keys in ([1, 1], [2, 1], [-1], [1 << 64]):
            with self.assertRaises(ValueError):
                encode_list(keys)
        with self.assertRaises(ValueError):
            decode_list((2**64 - 1).to_bytes(8, 'little'))
        with self.assertRaises(ValueError):
            decode_list(encode_list([1]) + b'\0')

    def test_direct_exact_transfer_is_terminal(self):
        row = run_session([1, 2], [2, 3], 0, None)
        self.assertEqual(row['rounds'], 1)
        self.assertEqual(row['bytes'], 2 * 48 + 8 + 2 * 8)
        self.assertEqual(row['verification_bytes'], 0)
        self.assertTrue(row['terminal_exact_transfer'])
        self.assertEqual(row['final'], [1, 2])

    def test_exhaustion_keeps_all_failed_cost(self):
        row = run_session(list(range(9)), [], 1, RejectWorker())
        self.assertEqual(row['rounds'], 5)
        self.assertEqual(row['candidate_bytes'], 5 * 96 + 73 + 80)
        self.assertEqual(row['verification_bytes'], 0)
        self.assertTrue(row['terminal_exact_transfer'])
        self.assertEqual(row['fallbacks'], 1)

    def test_false_success_requires_separate_verification_and_fallback(self):
        class FalseWorker(RejectWorker):
            def request(self, line):
                return 'ok -' if line.startswith('decode ') else super().request(line)
        row = run_session([1], [], 1, FalseWorker())
        self.assertEqual(row['false_candidates'], 1)
        self.assertEqual(row['rounds'], 3)
        self.assertEqual(row['fallbacks'], 1)
        self.assertEqual(row['false_candidates'], 1)
        self.assertTrue(row['terminal_exact_transfer'])
        self.assertEqual(row['final'], [1])

    def test_generation_and_sequence_are_not_reusable(self):
        s = Session(0)
        s.exchange(0, encode_list([]))
        f = Frame(1, 0, 1, 11, 13, 0, 0, b'')
        expected = Frame(1, 0, 1, 11, 13, s.sequence, 0, b'')
        with self.assertRaises(ValueError):
            Frame.decode(f.encode(), expected)

    def test_cross_session_replay_is_rejected(self):
        old = Session(0)
        new = Session(0)
        stale = Frame(1, 0, *old.identity, 0, 0, b'').encode()
        with self.assertRaises(ValueError):
            new.receive(stale, 1, 0, 0)

    def test_model_units(self):
        self.assertEqual(model_ns(1000, 2, 1000, 10, 1), 28_001_000)
        for bandwidth in (0, -1):
            with self.assertRaises(ValueError):
                model_ns(0, 0, 0, 0, bandwidth)


if __name__ == '__main__':
    unittest.main()
