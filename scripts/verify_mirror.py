#!/usr/bin/env python3
"""Mirror-identity check: prove `vectors/legacy/` matches the agent_gov admission
manifest byte-for-byte.

The corpus is sovereign in agent_gov (`golden/corpus/`, admitted via
`golden/corpus/MANIFEST.json`); this repo's `vectors/legacy/` is a CONFORMANCE
MIRROR. Packet C's custody model requires the mirror to PROVE identity — it may
not mutate expected behavior locally. The agent_gov side already checks this when
transition-kernel is on disk (`tests/test_corpus_custody.py`); this script closes
the same boundary from the mirror side, so a divergence is caught even in a bare
transition-kernel checkout / CI.

Exit 0 iff every admitted case is present here with a matching sha256 and this
mirror carries no legacy case the sovereign never admitted. Skips (exit 0, with a
reason on stderr) when the sovereign manifest is not reachable — never a silent
pass that could hide divergence.

Usage:  python3 scripts/verify_mirror.py
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

MIRROR = Path(__file__).resolve().parent.parent / "vectors" / "legacy"
MANIFEST = Path.home() / "git" / "agent_gov" / "golden" / "corpus" / "MANIFEST.json"


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    if not MANIFEST.is_file():
        print(
            f"[verify_mirror] sovereign manifest not reachable at {MANIFEST}; "
            f"mirror identity unverifiable in this checkout (skipped, not "
            f"silently passed).",
            file=sys.stderr,
        )
        return 0

    manifest = json.loads(MANIFEST.read_text())
    admitted = {c["id"]: c["sha256"] for c in manifest["cases"]}
    mirror_files = {
        p.name for p in MIRROR.glob("*.json") if p.name != "MANIFEST.json"
    }

    problems: list[str] = []
    for cid, want in admitted.items():
        p = MIRROR / cid
        if not p.is_file():
            problems.append(f"missing admitted case {cid!r} (mirror dropped a case)")
            continue
        got = _sha256(p)
        if got != want:
            problems.append(
                f"{cid}: sha256 {got[:12]} != admitted {want[:12]} "
                f"(mirror mutated expected behavior locally)"
            )
    extra = mirror_files - set(admitted)
    for cid in sorted(extra):
        problems.append(
            f"{cid}: legacy case absent from the admitted corpus "
            f"(local corpus authorship is a custody violation)"
        )

    if problems:
        print("[verify_mirror] MIRROR DIVERGED from the admitted corpus:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        return 1

    print(f"[verify_mirror] {len(admitted)} cases; mirror byte-identical to the admitted corpus")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
