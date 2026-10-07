"""Frozen M6-D13-B wire/network model."""
from __future__ import annotations

import json
import math
import statistics
from pathlib import Path

CONTRACT = json.loads(Path(__file__).with_name("contract.json").read_text())


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def exact_exchange_bytes(key_count: int) -> int:
    require(key_count >= 0, "negative key count")
    return 2 * CONTRACT["header_bytes"] + CONTRACT["exact_list_count_bytes"] + 8 * key_count


def d11_wire(sync: dict[str, int]) -> tuple[int, int]:
    final_k = sync["final_k"]
    require(final_k in (1, 2, 4, 8), "invalid D11 final_k")
    require(sync["fallback"] in (0, 1), "invalid D11 fallback")
    require(sync["false_candidate"] in (0, 1), "invalid D11 false-candidate flag")
    require(sync["false_candidate"] <= sync["fallback"], "false candidate without fallback")

    stages = (1, 2, 4, 8)
    increments = (17, 8, 16, 32)
    attempts = stages.index(final_k) + 1
    wire_bytes = sum(2 * CONTRACT["header_bytes"] + increments[i] for i in range(attempts))
    rounds = attempts
    n = sync["source_a_len"]

    if sync["fallback"] == 0:
        wire_bytes += exact_exchange_bytes(n)
        rounds += 1
    elif sync["false_candidate"] == 1:
        # B0 does not expose the temporary false-candidate cardinality.
        # Predeclared conservative upper bound: exact cardinality + final_k.
        wire_bytes += exact_exchange_bytes(n + final_k)
        rounds += 1
        wire_bytes += exact_exchange_bytes(n)
        rounds += 1
    else:
        wire_bytes += exact_exchange_bytes(n)
        rounds += 1

    return wire_bytes, rounds


def riblt_wire(sync: dict[str, int], lane: str) -> tuple[int, int, str]:
    require(lane in ("pull", "stream"), "invalid RIBLT lane")
    cells = sync["cells"]
    batches = sync["batches"]
    fallback = sync["fallback"]
    require(cells >= 1, "zero RIBLT cells")
    require(batches >= 1, "zero RIBLT batches")
    require(fallback in (0, 1), "invalid RIBLT fallback")

    header = CONTRACT["header_bytes"]
    cell_bytes = CONTRACT["riblt_cell_bytes"]
    limit = CONTRACT["riblt_limit_cells"]
    n = sync["source_a_len"]

    if lane == "pull":
        wire_bytes = cells * cell_bytes + batches * (2 * header)
        rounds = batches
    else:
        require(batches == 1, "stream_lb must be one receiver-completion stream")
        wire_bytes = header + cells * cell_bytes + header
        rounds = 1

    fallback_kind = "none"
    if fallback == 0:
        wire_bytes += exact_exchange_bytes(n)
        rounds += 1
    elif cells < limit:
        # Decode terminated before the cap, then verification failed.
        fallback_kind = "false_candidate"
        wire_bytes += exact_exchange_bytes(n)
        rounds += 1
        wire_bytes += exact_exchange_bytes(n)
        rounds += 1
    else:
        fallback_kind = "exhaustion"
        wire_bytes += exact_exchange_bytes(n)
        rounds += 1

    return wire_bytes, rounds, fallback_kind


def direct_wire(sync: dict[str, int]) -> tuple[int, int]:
    require(sync["fallback"] == 0, "direct fallback")
    return exact_exchange_bytes(sync["source_a_len"]), 1


def modeled_total_ns(
    session_native_ns: int,
    build_ns: int,
    sessions_per_build: int,
    application_bytes: int,
    rounds: int,
    rtt_ms: int,
    bandwidth_mbps: int,
) -> float:
    require(session_native_ns >= 0 and build_ns >= 0, "negative native time")
    require(sessions_per_build > 0, "invalid sessions/build")
    require(application_bytes >= 0 and rounds >= 0, "negative wire accounting")
    require(rtt_ms >= 0 and bandwidth_mbps > 0, "invalid network cell")
    network_rtt_ns = rounds * rtt_ms * 1_000_000
    serialization_ns = application_bytes * 8_000 / bandwidth_mbps
    value = session_native_ns + build_ns / sessions_per_build + network_rtt_ns + serialization_ns
    require(math.isfinite(value) and value >= 0, "non-finite modeled cost")
    return value


def median(values: list[float | int]) -> float:
    require(bool(values), "empty median")
    result = float(statistics.median(values))
    require(math.isfinite(result), "non-finite median")
    return result
