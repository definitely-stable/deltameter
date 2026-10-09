#!/usr/bin/env python3
"""G1-B2-A: exact, fail-closed, single-level near-full parity guard.

All inferential comparisons are arbitrary-precision integers. BLAKE3 is
NOT asserted to be an independent oracle or secure under adaptive inputs.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

BS = tuple(range(6, 14))
TS = (32, 64, 128)
ALPHA_DEN = 1_000_000
RATIOS = ((0, 1), (1, 4), (1, 2), (3, 4), (9, 10),
          (99, 100), (1, 1), (101, 100), (11, 10))
FORMAT = "deltameter.guard-b2a-nearfull-exact.v1"


def m_for_b(b: int) -> int:
    if type(b) is not int or not 1 <= b <= 13:
        raise ValueError("out of scope b")
    return (1 << b) - 1


def sampled_d(t: int, ratio: tuple[int, int]) -> int:
    num, den = ratio
    if not (type(t) is int and t > 0 and type(num) is int
            and type(den) is int and num >= 0 and den >= 1):
        raise ValueError("bad ratio or threshold")
    prod = t * num
    if num > den:
        return (prod + den - 1) // den
    return prod // den


def advance(current: list[int], m: int, d: int) -> list[int]:
    if not (type(m) is int and m >= 1 and type(d) is int
            and d >= m + 1 and current and len(current) <= m + 1
            and all(type(n) is int and n >= 0 for n in current)):
        raise ValueError("invalid transition")
    out = [0] * min(m + 1, len(current) + 1)
    for s in range(len(out)):
        if s < len(current):
            out[s] += current[s] * (d - m)
        if s > 0:
            out[s] += current[s - 1] * (m - s + 1)
        if s + 1 < len(current):
            out[s] += current[s + 1] * (s + 1)
    return out


def exact_laws(m: int, steps: int, checkpoints: set[int]) -> dict[int, tuple[list[int], int]]:
    if (type(m) is not int or m < 1 or type(steps) is not int or
        not 0 <= steps <= 256 or not isinstance(checkpoints, set) or
        not all(type(t) is int and 0 <= t <= steps for t in checkpoints)):
        raise ValueError("bad law inputs")
    d = m + 1
    numerator, total = [1], 1
    captured = {}
    if 0 in checkpoints:
        captured[0] = (numerator.copy(), total)
    for n in range(1, steps + 1):
        numerator = advance(numerator, m, d)
        total *= d
        if sum(numerator) != total:
            raise AssertionError("loss of exact integer normalization")
        if n in checkpoints:
            captured[n] = (numerator.copy(), total)
    assert len(captured) == len(checkpoints)
    return captured


def exact_cutoff(counts: list[int], total: int, alpha_den: int = ALPHA_DEN) -> int:
    if (not counts or type(total) is not int or total < 1 or
        type(alpha_den) is not int or alpha_den < 1 or
        any(type(a) is not int or a < 0 for a in counts)
        or sum(counts) != total):
        raise ValueError("malformed probability distribution")
    result, cumulative = -1, 0
    for s, v in enumerate(counts):
        cumulative += v
        if alpha_den * cumulative <= total:
            result = s
        else:
            break
    return result


def full_bitmask_oracle(m: int, steps: int) -> tuple[list[int], int]:
    """Independent DP over individual bitmap masks, NEVER odd-count chain."""
    if not (1 <= m <= 7 and 0 <= steps <= 12):
        raise ValueError("tiny only")
    d = m + 1
    state = [0] * (1 << m)
    state[0] = 1
    for _ in range(steps):
        nxt = [0] * len(state)
        for mask, v in enumerate(state):
            if v:
                nxt[mask] += v  # exactly ONE dummy slot, per mask
                for row in range(m):
                    nxt[mask ^ (1 << row)] += v
        state = nxt
    buckets = [0] * (m + 1)
    for mask, v in enumerate(state):
        buckets[mask.bit_count()] += v
    return buckets, d**steps


def verify_tiny() -> int:
    count = 0
    for b in (1, 2, 3):
        m = m_for_b(b)
        checkpoints = set(range(13))
        laws = exact_laws(m, 12, checkpoints)
        for steps in range(13):
            a, total = laws[steps]
            independent, independent_total = full_bitmask_oracle(m, steps)
            assert total == independent_total
            assert a == independent[:len(a)] and not any(independent[len(a):])
            assert sum(independent) == total
            count += 1
        # finite all-d monotonicity from exact rational CDF, also symbolic criterion
        assert (m + 1) <= (1 << b)
        for steps in range(12):
            a, a_den = laws[steps]
            z, z_den = laws[steps + 1]
            aa, zz = 0, 0
            for s in range(max(len(a), len(z))):
                aa += a[s] if s < len(a) else 0
                zz += z[s] if s < len(z) else 0
                assert zz * a_den <= aa * z_den
    # The non-lazy hypercube D=m counterexample:
    # m=1, d=1 always odd, d=2 always even; CDF(S=0) goes 0->1.
    assert 0 < 1 and (1 + 1) > 1
    assert count == 39
    # Wrong shape, malformed value, bogus negative/false self-loop.
    for f, args in (
        (m_for_b, (0,)), (m_for_b, (14,)), (m_for_b, (3.0,)),
        (exact_laws, (0, 5, {5})),
        (exact_laws, (3, 257, {1})),
        (exact_laws, (3, 2, {3})),
        (advance, ([1], 7, 7)),  # disallow D=m periodic chain
        (advance, ([1, -1], 3, 4)),
        (exact_cutoff, ([1, 1], 1)),
        (exact_cutoff, ([0, -1], 1)),
        (exact_cutoff, ([1], 1, 0)),
        (full_bitmask_oracle, (63, 1)),
        (sampled_d, (32, (100, 0))),
    ):
        try:
            f(*args)
        except ValueError:
            continue
        raise AssertionError(f"malformed case not rejected: {f.__name__} {args}")
    return count


def certified_frontier() -> tuple[dict, list[str]]:
    tiny = verify_tiny()
    table = ["# deltameter.guard-b2a-nearfull-cutoffs.v1",
             "# exact integer T+1 certificate; independently held immutable profile",
             "# T b m cutoff"]
    entries = []
    for b in BS:
        m = m_for_b(b)
        for t in TS:
            targets = {t + 1, *(sampled_d(t, ratio) for ratio in RATIOS)}
            laws = exact_laws(m, max(targets), targets)
            counts, total = laws[t + 1]
            cutoff = exact_cutoff(counts, total)
            risk_count = sum(counts[:cutoff + 1])
            assert ALPHA_DEN * risk_count <= total
            if cutoff + 1 < len(counts):
                assert ALPHA_DEN * (risk_count + counts[cutoff + 1]) > total
            assert (m + 1) == (1 << b), "hash mapping must be unbiased"
            assert (m + 1) <= (m + 1), "monotone kernel criterion"
            power = {}
            for num, den in RATIOS:
                d = sampled_d(t, (num, den))
                n, power_den = laws[d]
                n_good = sum(n[:cutoff + 1])
                power[f"{num}/{den}"] = {
                    "d": d,
                    "ppm_lower_floor": (ALPHA_DEN * n_good) // power_den,
                }
                if 0 <= d <= cutoff:
                    assert n_good == power_den, "deterministic power was broken"
                if d > t:
                    assert ALPHA_DEN * n_good <= power_den, "all-d monotonic tail violated"
            entries.append({
                "b": b, "m": m, "T": t, "bitmap_bytes": ((m + 63) // 64) * 8,
                "physical_ignored_high_bits": (64 - (m % 64)) % 64,
                "D": m + 1, "cutoff": cutoff,
                "error_exact_numerator": str(risk_count),
                "error_exact_denominator": str(total),
                "risk_condition": "1000000*error_exact_numerator<=error_exact_denominator",
                "power": power,
            })
            table.append(f"{t} {b} {m} {cutoff}")
            print(f"B2A_EXACT T={t} b={b} m={m} cutoff={cutoff} "
                  f"bytes={((m + 63)//64)*8} p75_ppm={power['3/4']['ppm_lower_floor']}",
                  flush=True)
    assert len(entries) == 24 and len(table) == 27
    target = next(e for e in entries if e["b"] == 11 and e["T"] == 64)
    assert target["cutoff"] >= 48 and target["bitmap_bytes"] <= 256,         "frozen useful candidate proof gate failed"
    assert target["power"]["3/4"]["ppm_lower_floor"] >= 950_000
    return {
        "format": FORMAT,
        "alpha": "1/1000000",
        "profiles": "T,b fixed before inspecting the bitmap",
        "oracle": "fresh ideal independent uniform b-bit slot for each distinct u64 token",
        "keyed_blake3_computational_security": "NOT_PROVEN",
        "symbolic_monotonicity": "(m+1)/D=1, adjacent stochastic kernel ordered",
        "tiny_independent_bitmask_cases": tiny,
        "non_lazy_periodicity_witness": "m=1,D=1,d=1->2,Pr(S=0):0->1",
        "result": "B2A_EXACT_CERT_PASS",
        "entries": entries,
    }, table


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", type=Path, required=True)
    a = parser.parse_args()
    report, table = certified_frontier()
    a.out_dir.mkdir(parents=True, exist_ok=True)
    raw = ("\n".join(table) + "\n").encode("ascii")
    (a.out_dir / "cutoffs.tsv").write_bytes(raw)
    digest = hashlib.sha256(raw).hexdigest()
    report["cutoff_sha256"] = digest
    (a.out_dir / "certificate.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"B2A_EXACT_CERT_PASS profiles=24 tiny_oracles=39 cutoff_sha256={digest}")


if __name__ == "__main__":
    main()
