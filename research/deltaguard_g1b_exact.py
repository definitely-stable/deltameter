#!/usr/bin/env python3
"""G1B: exact fixed-d lazy-Ehrenfest one-sided guard certification.

All decisions depend only on arbitrary-precision integer arithmetic. The
ideal oracle is assumed to independently assign each distinct input token.
The public BLAKE3 test key is NOT a source of mathematical randomness.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

M = 4096
TS = (32, 64, 256)
LEVELS = (1, 2, 3, 4)
PROFILES = ("original", "unit")
DELTA_DENOMINATOR = 1_000_000
D_RATIO_NUMERATORS = (0, 2, 6, 8, 9)

def denominator(m: int, j: int, profile: str) -> int:
    if not isinstance(m, int) or m < 1 or not isinstance(j, int) or j < 1:
        raise ValueError("m and j must be positive integers")
    if profile not in PROFILES:
        raise ValueError("unknown oracle profile")
    d = m * (1 << (j + (1 if profile == "original" else 0)))
    if m + 1 > d:
        raise ValueError("nonmonotone Markov kernel")
    return d


def next_count_state(old: list[int], m: int, d: int) -> list[int]:
    """Exact integer recurrence for # odd cells, numerator denominator d^(t+1)."""
    if not (1 <= m < d and old and len(old) <= m + 1):
        raise ValueError("invalid state")
    if any(type(v) is not int or v < 0 for v in old):
        raise ValueError("noninteger/negative state")
    result = [0] * min(m + 1, len(old) + 1)
    for s in range(len(result)):
        value = old[s] * (d - m) if s < len(old) else 0
        if s > 0:
            value += old[s-1] * (m - s + 1)
        if s + 1 < len(old):
            value += old[s+1] * (s + 1)
        result[s] = value
    return result


def count_law(m: int, level: int, profile: str, steps: int,
              checkpoints: set[int] | None = None) -> dict[int, tuple[list[int], int]]:
    d = denominator(m, level, profile)
    if not isinstance(steps, int) or not 0 <= steps <= 10000:
        raise ValueError("steps")
    checkpoints = checkpoints if checkpoints is not None else {steps}
    if any(x < 0 or x > steps for x in checkpoints):
        raise ValueError("invalid checkpoint")
    old = [1]
    total = 1
    result = {}
    if 0 in checkpoints:
        result[0] = (old.copy(), 1)
    for t in range(1, steps + 1):
        old = next_count_state(old, m, d)
        total *= d
        assert sum(old) == total, f"probability normalization failed at t={t}"
        if t in checkpoints:
            result[t] = (old.copy(), total)
    return result


def exact_cutoff(numerators: list[int], total: int, delta_den: int) -> int:
    if (not numerators or any(x < 0 for x in numerators) or
        total < 1 or sum(numerators) != total or delta_den < 1):
        raise ValueError("malformed input")
    cutoff = -1
    cumulative = 0
    for s, count in enumerate(numerators):
        cumulative += count
        if delta_den * cumulative <= total:
            cutoff = s
        else:
            break
    assert cutoff < 0 or delta_den*sum(numerators[:cutoff+1]) <= total
    assert cutoff + 1 >= len(numerators) or (
        delta_den*sum(numerators[:cutoff+2]) > total
    )
    return cutoff


def mask_oracle(m: int, j: int, profile: str, steps: int) -> list[int]:
    """Independent 2^m PARITY-MASK DP; no occupancy recurrence."""
    if m > 6 or steps > 8:
        raise ValueError("oracle only accepts tiny states")
    d = denominator(m, j, profile)
    paths = [0] * (1 << m)
    paths[0] = 1
    for _ in range(steps):
        after = [0] * len(paths)
        for bits, numerator in enumerate(paths):
            if not numerator:
                continue
            after[bits] += numerator * (d - m)
            for i in range(m):
                after[bits ^ (1 << i)] += numerator
        paths = after
    counts = [0] * (m + 1)
    for mask, ways in enumerate(paths):
        counts[mask.bit_count()] += ways
    return counts


def tiny_oracles() -> int:
    tested = 0
    for m in (2, 3, 5):
        for j in (1, 2, 3):
            for profile in PROFILES:
                all_laws = count_law(m,j,profile,8,set(range(9)))
                for t in range(9):
                    count, den = all_laws[t]
                    oracle = mask_oracle(m,j,profile,t)
                    assert count == oracle[:len(count)] and not any(oracle[len(count):]), (m,j,profile,t,count,oracle)
                    assert sum(oracle) == den, (m,j,profile,t,den,sum(oracle))
                    tested += 1
    # One-sided independent rejection / missing-state / type errors.
    for invalid in ((0,1,"original"), (5,0,"original"), (5,1,"wrong")):
        try:
            denominator(*invalid)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid profile accepted")
    for invalid in (([1,1],1,100), ([1,-1],1,100), ([1],0,100)):
        try:
            exact_cutoff(*invalid)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid exact probability accepted")
    assert 1_000_000 * 3**33 > 4**33
    return tested


def power_ppm(n: list[int], total: int, cutoff: int) -> int:
    if cutoff < 0:
        return 0
    return (1_000_000 * sum(n[:cutoff + 1])) // total


def calculate() -> tuple[dict,list[str]]:
    checked = tiny_oracles()
    results = []
    tsv = ["# format deltameter.guard-g1b-cutoff.v1",
           "# ideal-oracle exactly certified; fixed T, level j and profile",
           "# T profile level cutoff"]
    negative = 0
    for profile in PROFILES:
        for j in LEVELS:
            d = denominator(M,j,profile)
            needed={x+1 for x in TS}
            for t in TS:
                for num in D_RATIO_NUMERATORS:
                    needed.add(t*num//8)
            max_d=max(needed)
            laws=count_law(M,j,profile,max_d,needed)
            # Stochastic monotonicity is a symbolic kernel fact:
            assert M + 1 <= d
            # Independent full-law finite sanity check for adjacent times:
            for a in (0,1,2,32,64):
                if a+1>max_d:
                    continue
                first, first_den=count_law(M,j,profile,a)[a]
                second,second_den=count_law(M,j,profile,a+1)[a+1]
                running_first=running_second=0
                for s in range(max(len(first),len(second))):
                    if s<len(first):
                        running_first+=first[s]
                    if s<len(second):
                        running_second+=second[s]
                    assert running_second*first_den <= running_first*second_den
            for t in TS:
                n_at_risk,total=laws[t+1]
                cutoff=exact_cutoff(n_at_risk,total,DELTA_DENOMINATOR)
                if cutoff==-1:
                    negative+=1
                # independent exact verifier of all possible cutoff indices
                exact_cumulative=sum(n_at_risk[:cutoff+1])
                assert DELTA_DENOMINATOR*exact_cumulative <= total
                if cutoff+1<len(n_at_risk):
                    assert DELTA_DENOMINATOR*sum(n_at_risk[:cutoff+2]) > total
                metrics={}
                for nratio in D_RATIO_NUMERATORS:
                    td=t*nratio//8
                    n,p_den=laws[td]
                    metrics[str(nratio)]=power_ppm(n,p_den,cutoff)
                rec={"profile":profile,"level":j,"T":t,
                     "m":M,"denominator_per_token":d,
                     "cutoff":cutoff,"alpha":"1/1000000",
                     "exact_error_num":str(exact_cumulative),
                     "exact_error_den":str(total),
                     "exact_tail_bound_comparison":
                         "1000000 * exact_error_num <= exact_error_den",
                     "power_floor_ppm_by_d_over_T_numerator":metrics}
                results.append(rec)
                tsv.append(f"{t} {profile} {j} {cutoff}")
                print(f"G1B_EXACT_CUTOFF T={t} profile={profile} j={j} cutoff={cutoff} "
                      f"power_quarter_ppm={metrics['2']} power_3quarter_ppm={metrics['6']}",
                      flush=True)
    assert len(results)==24 and len(tsv)==27
    return {
        "format":"deltameter.guard-g1b-exact-chain.v1",
        "m":M,"levels":list(LEVELS),"thresholds":list(TS),
        "profiles":list(PROFILES),
        "delta":"1/1000000",
        "tiny_full_bitmask_oracle_cases":checked,
        "impossible_single_level_cutoffs":negative,
        "sto_order_proof":"adjacent chain (m+1)/D<=1, P(S_d<=c) nonincreasing in integer d",
        "token_independence_assumption":"ideal oracle and nonadaptive unique tokens",
        "coeff_profile_key_separation_required":True,
        "confidence_claim":"ideal-oracle single preselected T/j/profile one-sided <=1e-6",
        "computational_security_claim":"NONE",
        "result":"G1B_EXACT_CHAIN_CERT_PASS",
        "entries":results,
    },tsv


def main()->None:
    p=argparse.ArgumentParser()
    p.add_argument("--out-dir",type=Path,required=True)
    a=p.parse_args()
    report,lines=calculate()
    a.out_dir.mkdir(parents=True,exist_ok=True)
    raw=("\n".join(lines)+"\n").encode("ascii")
    (a.out_dir/"cutoffs.tsv").write_bytes(raw)
    report["cutoff_sha256"]=hashlib.sha256(raw).hexdigest()
    (a.out_dir/"certificate.json").write_text(
        json.dumps(report,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(f"G1B_EXACT_CHAIN_CERT_PASS profiles={len(report['entries'])} "
          f"tiny_oracle_cases={report['tiny_full_bitmask_oracle_cases']} "
          f"cutoff_sha256={report['cutoff_sha256']}",flush=True)


if __name__=="__main__":
    main()
