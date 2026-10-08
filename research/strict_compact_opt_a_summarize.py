#!/usr/bin/env python3
"""Summarize STRICT-COMPACT OPT-A stage-1 J/layout screen."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

J_VALUES = (24, 32, 40, 48, 56, 64)
LAYOUTS = ("row", "level")
RANGE_TIERS = ((1 << 32) - 1, (1 << 48) - 1)


def parse_case(line: str) -> dict | None:
    prefix = "STRICT_COMPACT_OPT_A_CASE "
    if not line.startswith(prefix):
        return None

    fields: dict[str, str] = {}
    for part in line[len(prefix) :].split():
        key, value = part.split("=", 1)
        fields[key] = value

    required = {
        "J",
        "layout",
        "state_bytes",
        "update_ns_per_token",
        "estimate_ns",
        "xor_ns_per_kib",
    }
    if set(fields) != required:
        raise ValueError(f"unexpected case fields: {sorted(fields)}")

    row = {
        "stored_levels": int(fields["J"]),
        "layout": fields["layout"],
        "state_bytes": int(fields["state_bytes"]),
        "update_ns_per_token": float(fields["update_ns_per_token"]),
        "estimate_ns": float(fields["estimate_ns"]),
        "xor_ns_per_kib": float(fields["xor_ns_per_kib"]),
    }
    if row["stored_levels"] not in J_VALUES:
        raise ValueError("unexpected J")
    if row["layout"] not in LAYOUTS:
        raise ValueError("unexpected layout")
    for key in ("update_ns_per_token", "estimate_ns", "xor_ns_per_kib"):
        if not math.isfinite(row[key]) or row[key] <= 0.0:
            raise ValueError(f"invalid timing {key}")
    return row


def choose_layout(rows: list[dict]) -> tuple[str, str]:
    by_layout = {row["layout"]: row for row in rows}
    row = by_layout["row"]
    level = by_layout["level"]

    row_update_10pct_faster = (
        row["update_ns_per_token"] <= 0.90 * level["update_ns_per_token"]
    )
    level_estimate_20pct_better = (
        level["estimate_ns"] <= 0.80 * row["estimate_ns"]
    )
    level_xor_20pct_better = (
        level["xor_ns_per_kib"] <= 0.80 * row["xor_ns_per_kib"]
    )

    if (
        row_update_10pct_faster
        and not level_estimate_20pct_better
        and not level_xor_20pct_better
    ):
        return "row", "row_update>=10pct_faster_without_level_compensation"

    reasons = []
    if not row_update_10pct_faster:
        reasons.append("row_update_not_10pct_faster")
    if level_estimate_20pct_better:
        reasons.append("level_estimate>=20pct_better")
    if level_xor_20pct_better:
        reasons.append("level_xor>=20pct_better")
    return "level", "+".join(reasons)


def first_j_reaching(frontier: list[dict], target: int) -> int | None:
    for row in frontier:
        if row["d_max"] >= target:
            return row["stored_levels"]
    return None


def build_payload(frontier_path: Path, lab_path: Path) -> dict:
    frontier_payload = json.loads(frontier_path.read_text(encoding="utf-8"))
    frontier = frontier_payload["profiles"]

    if [row["stored_levels"] for row in frontier] != list(J_VALUES):
        raise AssertionError("frontier J order mismatch")

    parsed = [
        row
        for line in lab_path.read_text(encoding="utf-8").splitlines()
        if (row := parse_case(line)) is not None
    ]
    if len(parsed) != len(J_VALUES) * len(LAYOUTS):
        raise AssertionError(f"expected 12 timing cases, got {len(parsed)}")

    cases: dict[int, list[dict]] = {levels: [] for levels in J_VALUES}
    seen: set[tuple[int, str]] = set()
    for row in parsed:
        key = (row["stored_levels"], row["layout"])
        if key in seen:
            raise AssertionError(f"duplicate timing case {key}")
        seen.add(key)
        cases[row["stored_levels"]].append(row)

    decisions = []
    for profile in frontier:
        levels = profile["stored_levels"]
        expected_bytes = profile["state_bytes"]
        rows = sorted(cases[levels], key=lambda row: row["layout"])
        if {row["layout"] for row in rows} != set(LAYOUTS):
            raise AssertionError(f"missing layout for J={levels}")
        if any(row["state_bytes"] != expected_bytes for row in rows):
            raise AssertionError(f"physical state-size mismatch for J={levels}")

        selected_layout, reason = choose_layout(rows)
        decisions.append(
            {
                **profile,
                "layout_cases": rows,
                "stage1_selected_layout": selected_layout,
                "layout_reason": reason,
            }
        )

    smallest_nontrivial = next(
        (
            row["stored_levels"]
            for row in frontier
            if row["d_max"] >= frontier_payload["declared_d_min"]
        ),
        None,
    )

    requested = [64]
    if smallest_nontrivial is not None:
        requested.append(smallest_nontrivial)
    for target in RANGE_TIERS:
        match = first_j_reaching(frontier, target)
        if match is not None:
            requested.append(match)

    shortlist = []
    for levels in requested:
        if levels not in shortlist:
            shortlist.append(levels)

    selected = []
    by_j = {row["stored_levels"]: row for row in decisions}
    for levels in shortlist:
        row = by_j[levels]
        layout = row["stage1_selected_layout"]
        timing = next(
            item for item in row["layout_cases"] if item["layout"] == layout
        )
        selected.append(
            {
                "stored_levels": levels,
                "layout": layout,
                "state_bytes": row["state_bytes"],
                "d_max": row["d_max"],
                "first_uncertified_d": row["first_uncertified_d"],
                "update_ns_per_token": timing["update_ns_per_token"],
                "estimate_ns": timing["estimate_ns"],
                "xor_ns_per_kib": timing["xor_ns_per_kib"],
                "layout_reason": row["layout_reason"],
            }
        )

    if 64 not in shortlist:
        raise AssertionError("J=64 control missing")
    if len(shortlist) > 4:
        raise AssertionError("shortlist exceeds frozen maximum")

    return {
        "format": "deltameter.strict-compact-opt-a-stage1.v1",
        "frontier_decision": frontier_payload["decision"],
        "profiles": decisions,
        "shortlist_j": shortlist,
        "shortlist": selected,
        "range_tiers": [str(value) for value in RANGE_TIERS],
        "decision": "OPT_A_STAGE1_PASS",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frontier", type=Path, required=True)
    parser.add_argument("--lab", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    payload = build_payload(args.frontier, args.lab)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print(
        "STRICT_COMPACT_OPT_A_STAGE1_PASS "
        f"shortlist={','.join(map(str, payload['shortlist_j']))}"
    )
    for row in payload["shortlist"]:
        print(
            "STRICT_COMPACT_OPT_A_SHORTLIST "
            f"J={row['stored_levels']} layout={row['layout']} "
            f"state_bytes={row['state_bytes']} d_max={row['d_max']} "
            f"update_ns={row['update_ns_per_token']:.6f} "
            f"estimate_ns={row['estimate_ns']:.3f} "
            f"xor_ns_per_kib={row['xor_ns_per_kib']:.6f}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
