"""Fail-closed helpers for M6-D13-B0 readiness evidence."""
from __future__ import annotations

import json
from pathlib import Path

CONTRACT = json.loads(Path(__file__).with_name("readiness.json").read_text())

SYNC_PHASES = [
    "serialize_ns",
    "apply_exact_ns",
    "prefix_ns",
    "decode_ns",
    "apply_sketch_ns",
    "verification_prepare_ns",
    "fallback_serialize_ns",
    "fallback_apply_exact_ns",
    "fallback_apply_sketch_ns",
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def parse_record(line: str) -> tuple[str, dict[str, int]]:
    parts = line.strip().split()
    require(parts, "empty record")
    kind = parts[0]
    fields: dict[str, int] = {}
    for token in parts[1:]:
        require("=" in token, "malformed token")
        key, value = token.split("=", 1)
        require(key not in fields, f"duplicate field {key}")
        fields[key] = int(value)
        require(fields[key] >= 0, f"negative field {key}")
    return kind, fields


def validate_memory(fields: dict[str, int]) -> None:
    for key in ("vmrss_bytes", "vmhwm_bytes"):
        require(key in fields and fields[key] > 0, f"missing {key}")
    require(fields["vmhwm_bytes"] >= fields["vmrss_bytes"], "VmHWM below VmRSS")


def validate_ready(line: str, mode: str) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "ready", "not ready record")
    for key in (
        "source_build_ns",
        "sketch_build_ns",
        "clk_tck",
        "source_len",
        "source_a_cap",
        "source_b_cap",
        "sketch_payload_bytes",
    ):
        require(key in fields, f"missing {key}")
    require(fields["clk_tck"] > 0, "invalid CLK_TCK")
    require(fields["source_a_cap"] >= fields["source_len"], "A capacity")
    require(fields["source_b_cap"] >= fields["source_len"], "B capacity")
    require(
        fields["sketch_payload_bytes"] == (146 if mode == "d11" else 0),
        "sketch payload",
    )
    validate_memory(fields)
    return fields


def validate_update(line: str, mode: str, expected_ok: bool) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "update", "not update record")
    for key in (
        "ok",
        "exact_ns",
        "sketch_ns",
        "native_total_ns",
        "cpu_ticks",
        "source_len",
        "source_cap",
    ):
        require(key in fields, f"missing {key}")
    require(fields["ok"] == int(expected_ok), "update outcome")
    require(
        fields["native_total_ns"] == fields["exact_ns"] + fields["sketch_ns"],
        "update phase closure",
    )
    if mode == "direct" or not expected_ok:
        require(fields["sketch_ns"] == 0, "unexpected sketch mutation timing")
    require(fields["source_cap"] >= fields["source_len"], "source capacity")
    return fields


def validate_sync(line: str, mode: str) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "sync", "not sync record")
    required = [
        "exact",
        "fallback",
        "final_k",
        *SYNC_PHASES,
        "native_total_ns",
        "candidate_capacity",
        "cpu_ticks",
        "clk_tck",
        "payload_len",
        "source_a_len",
        "source_a_cap",
        "source_b_len",
        "source_b_cap",
    ]
    for key in required:
        require(key in fields, f"missing {key}")
    require(fields["exact"] == 1, "non-exact result")
    require(fields["clk_tck"] > 0, "invalid CLK_TCK")
    require(fields["source_a_len"] == fields["source_b_len"], "source length mismatch")
    require(fields["source_a_cap"] >= fields["source_a_len"], "A capacity")
    require(fields["source_b_cap"] >= fields["source_b_len"], "B capacity")
    require(
        fields["native_total_ns"] == sum(fields[key] for key in SYNC_PHASES),
        "sync phase closure",
    )
    validate_memory(fields)

    if mode == "direct":
        require(fields["fallback"] == 0 and fields["final_k"] == 0, "direct protocol")
        require(fields["prefix_ns"] == fields["decode_ns"] == 0, "direct decoder work")
        require(fields["apply_sketch_ns"] == 0, "direct sketch work")
        require(fields["verification_prepare_ns"] == 0, "direct verification")
        require(fields["fallback_serialize_ns"] == 0, "direct fallback")
        require(fields["fallback_apply_exact_ns"] == 0, "direct fallback apply")
        require(fields["fallback_apply_sketch_ns"] == 0, "direct fallback sketch")
        require(
            fields["payload_len"] == 8 + 8 * fields["source_a_len"],
            "direct payload shape",
        )
    else:
        require(fields["final_k"] in (1, 2, 4, 8), "D11 stage")
        require(fields["serialize_ns"] == 0 and fields["payload_len"] == 0, "D11 direct leak")
        if fields["fallback"] == 0:
            require(fields["fallback_serialize_ns"] == 0, "unexpected fallback serialize")
            require(fields["fallback_apply_exact_ns"] == 0, "unexpected fallback apply")
            require(fields["fallback_apply_sketch_ns"] == 0, "unexpected fallback sketch")
        else:
            require(fields["fallback_serialize_ns"] > 0, "missing fallback serialize")
            require(fields["fallback_apply_exact_ns"] > 0, "missing fallback exact apply")
    return fields



def validate_diagnose(line: str) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "diagnose", "not diagnose record")
    required = (
        "exact_d",
        "a_rebuild",
        "b_rebuild",
        "maintained_decoded",
        "maintained_exact",
        "maintained_k",
        "fresh_decoded",
        "fresh_exact",
        "fresh_k",
        "candidate_equal",
    )
    for key in required:
        require(key in fields, f"missing {key}")
    require(fields["a_rebuild"] == 1, "maintained A differs from fresh rebuild")
    require(fields["b_rebuild"] == 1, "maintained B differs from fresh rebuild")
    require(fields["candidate_equal"] == 1, "maintained/fresh decoder diverged")
    require(fields["maintained_k"] in (1, 2, 4, 8), "maintained diagnostic stage")
    require(fields["fresh_k"] in (1, 2, 4, 8), "fresh diagnostic stage")
    require(
        fields["maintained_decoded"] == fields["fresh_decoded"],
        "maintained/fresh decode outcome differs",
    )
    require(
        fields["maintained_exact"] == fields["fresh_exact"],
        "maintained/fresh exactness differs",
    )
    return fields

def validate_check(line: str) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "check", "not check record")
    require(fields == {"equal": 1, "a_rebuild": 1, "b_rebuild": 1}, "state mismatch")
    return fields


RIBLT_PHASES = [
    "encoder_import_ns",
    "decoder_import_ns",
    "produce_ns",
    "decode_ns",
    "apply_ns",
    "verification_prepare_ns",
    "fallback_ns",
]


def validate_riblt_ready(line: str) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "ready", "not RIBLT ready record")
    for key in (
        "source_build_ns",
        "clk_tck",
        "source_len",
        "source_a_cap",
        "source_b_cap",
        "runtime_alloc_bytes",
        "runtime_heap_sys_bytes",
    ):
        require(key in fields, f"missing {key}")
    require(fields["clk_tck"] > 0, "invalid CLK_TCK")
    require(fields["source_a_cap"] >= fields["source_len"], "RIBLT A capacity")
    require(fields["source_b_cap"] >= fields["source_len"], "RIBLT B capacity")
    require(fields["runtime_heap_sys_bytes"] >= fields["runtime_alloc_bytes"], "Go heap accounting")
    validate_memory(fields)
    return fields


def validate_riblt_update(line: str, expected_ok: bool) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "update", "not RIBLT update record")
    for key in ("ok", "exact_ns", "native_total_ns", "cpu_ticks", "source_len", "source_cap"):
        require(key in fields, f"missing {key}")
    require(fields["ok"] == int(expected_ok), "RIBLT update outcome")
    require(fields["native_total_ns"] == fields["exact_ns"], "RIBLT update phase closure")
    require(fields["source_cap"] >= fields["source_len"], "RIBLT source capacity")
    return fields


def validate_riblt_sync(line: str, lane: str) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "sync", "not RIBLT sync record")
    required = [
        "lane",
        "exact",
        "fallback",
        "cells",
        "batches",
        *RIBLT_PHASES,
        "native_total_ns",
        "cpu_ticks",
        "clk_tck",
        "source_a_len",
        "source_a_cap",
        "source_b_len",
        "source_b_cap",
        "remote_cap",
        "local_cap",
        "runtime_alloc_bytes",
        "runtime_heap_sys_bytes",
    ]
    for key in required:
        require(key in fields, f"missing {key}")
    expected_lane = 0 if lane == "pull" else 1
    require(fields["lane"] == expected_lane, "RIBLT lane")
    require(fields["exact"] == 1, "RIBLT non-exact result")
    require(fields["cells"] >= 1, "RIBLT zero-cell success")
    require(fields["batches"] >= 1, "RIBLT batch count")
    if lane == "stream":
        require(fields["batches"] == 1, "stream lower-bound batch model")
    require(
        fields["native_total_ns"] == sum(fields[key] for key in RIBLT_PHASES),
        "RIBLT phase closure",
    )
    require(fields["source_a_len"] == fields["source_b_len"], "RIBLT source length mismatch")
    require(fields["source_a_cap"] >= fields["source_a_len"], "RIBLT A capacity")
    require(fields["source_b_cap"] >= fields["source_b_len"], "RIBLT B capacity")
    require(fields["runtime_heap_sys_bytes"] >= fields["runtime_alloc_bytes"], "Go heap accounting")
    validate_memory(fields)
    if fields["fallback"] == 0:
        require(fields["fallback_ns"] == 0, "unexpected RIBLT fallback time")
    else:
        require(fields["fallback_ns"] > 0, "missing RIBLT fallback time")
    return fields


def validate_riblt_check(line: str) -> dict[str, int]:
    kind, fields = parse_record(line)
    require(kind == "check", "not RIBLT check record")
    require(fields == {"equal": 1}, "RIBLT state mismatch")
    return fields
