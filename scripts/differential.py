#!/usr/bin/env python3
"""A1 differential: Rust transition-cli vs the frozen agent_governor.corpus.v1 contract.

The frozen `expected_verdict` in each corpus file IS the Python decision contract (agent_gov's own
`test_corpus_contract.py` asserts the live Python chain reproduces it). So the always-on differential is
**Rust CLI vs frozen contract** on the seven fields. If `agent_gov` is importable (its venv is active),
we ALSO run the live Python projection and diff it against the frozen contract, to catch Python drift.

A mismatch emits a classified divergence receipt and makes this script exit non-zero, unless the case is
listed in vectors/legacy/divergence_manifest.json (operator-approved). No articulate green tests.
"""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LEGACY = REPO / "vectors" / "legacy"
CLI = REPO / "target" / "debug" / "transition-cli"
FIELDS = [
    "outcome", "refusal_kind", "refusing_seam",
    "effect_count", "consumed", "operational", "proposal_packet_present",
]


def load_manifest() -> set[str]:
    path = LEGACY / "divergence_manifest.json"
    if not path.exists():
        return set()
    return set(json.loads(path.read_text()).get("accepted_cases", []))


def rust_verdict(case: dict) -> dict:
    proc = subprocess.run([str(CLI)], input=json.dumps(case), capture_output=True, text=True)
    out = json.loads(proc.stdout)
    if "verdict" not in out:
        raise SystemExit(f"transition-cli fail-closed on {case.get('case')}: {out}")
    return {k: out["verdict"][k] for k in FIELDS}


def project(a: dict) -> dict:
    return {k: a[k] for k in FIELDS}


def receipt(case: str, lhs_name: str, lhs: dict, rhs_name: str, rhs: dict) -> dict:
    diffs = {k: {lhs_name: lhs[k], rhs_name: rhs[k]} for k in FIELDS if lhs[k] != rhs[k]}
    return {"divergence_receipt": {"case": case, "classification": "unclassified", "fields": diffs}}


def main() -> int:
    if not CLI.exists():
        raise SystemExit(f"build first: {CLI} missing (cargo build)")
    accepted = load_manifest()

    # Optional: live Python projection.
    live = None
    try:
        sys.path.insert(0, str(Path.home() / "git" / "agent_gov"))
        from tests.test_corpus_contract import _live_chain, _verdict_tuple  # type: ignore

        def live(full_case: dict) -> dict:
            with tempfile.TemporaryDirectory() as td:
                result = _live_chain(full_case, Path(td))
            # _verdict_tuple already returns a dict keyed by the seven fields.
            return {k: _verdict_tuple(result)[k] for k in FIELDS}
    except Exception as exc:  # noqa: BLE001 — best-effort, not required for A1
        print(f"[differential] live Python projection unavailable ({exc}); Rust-vs-frozen only")

    unaccepted = 0
    for path in sorted(LEGACY.glob("*.json")):
        if path.name == "divergence_manifest.json":
            continue
        case = json.loads(path.read_text())
        name = case["case"]
        frozen = project(case["expected_verdict"])

        rust = rust_verdict(case)
        if rust != frozen:
            print(json.dumps(receipt(name, "rust", rust, "frozen", frozen), indent=2))
            if name in accepted:
                print(f"  (accepted by operator divergence manifest)")
            else:
                unaccepted += 1

        if live is not None:
            py = live(case)
            if py != frozen:
                print(json.dumps(receipt(name, "live_python", py, "frozen", frozen), indent=2))
                if name not in accepted:
                    unaccepted += 1

    total = len([p for p in LEGACY.glob("*.json") if p.name != "divergence_manifest.json"])
    print(f"[differential] {total} cases; {unaccepted} unaccepted divergence(s)")
    return 1 if unaccepted else 0


if __name__ == "__main__":
    raise SystemExit(main())
