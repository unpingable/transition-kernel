#!/usr/bin/env python3
"""Generate the canonical end-to-end receipt bundle (the summit specimen).

Runs the REAL enforce chain (la_cli + transition-cli) and captures four durable bundles:
  success/           consumed_effect_succeeded
  refusal/           a chain refusal (eligibility recombination) — fail-closed, durable, no consume
  crash-reconcile/   crash after effect; reconstruction recovers EffectSucceeded from the marker
  replay-refusal/    a replayed operation refuses via LA AlreadyConsumed

Each dir holds the durable JSONL + a reconstruct.json. Re-run to regenerate; the committed copies are the
canonical specimen.
"""
from __future__ import annotations

import json
import os
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path.home() / "git" / "agent_gov" / "src"))

from governor.runtime.la_subprocess import DEFAULT_LA_CLI, LASubprocess  # noqa: E402
from governor.runtime.transition_enforce import (  # noqa: E402
    CRASH_AFTER_EFFECT, MarkerActuator, enforce_chain, reconstruct, reconstruct_composed,
)
from governor.runtime.transition_subprocess import DEFAULT_TRANSITION_CLI, TransitionSubprocess  # noqa: E402

HERE = Path(__file__).resolve().parent
SCOPE, ELIG, CONTENT = "lab", "sha256:standing-xyz", b"create_marker_v1:fixed-bytes"


def _la():
    la = LASubprocess(os.environ.get("GOVERNOR_LA_CLI", DEFAULT_LA_CLI))
    la.start()
    la.deposit(SCOPE, 5)
    dec = la.request_capacity({
        "request_id": "r1", "actor": "ag:main", "action": "write", "target": "demo", "scope": SCOPE,
        "requested_capacity": 5, "eligibility_reference": ELIG, "eligibility_valid_until": 1000,
        "expires_after": 1000, "idempotency_key": None}, 10)
    assert dec["decision"] == "Granted"
    return la, dec["token_id"]


def _tk():
    t = TransitionSubprocess(binary_path=os.environ.get("GOVERNOR_TRANSITION_CLI", DEFAULT_TRANSITION_CLI))
    t.start()
    return t


def _run(name, *, op, eligibility=ELIG, crash_at=None, sandbox=None):
    d = HERE / name
    d.mkdir(parents=True, exist_ok=True)
    durable = d / "durable.jsonl"
    if durable.exists():
        durable.unlink()
    # Reproducible from clean: the create_new effect must not collide with a marker left by a prior run.
    shutil.rmtree(d / "sandbox", ignore_errors=True)
    la, token = _la()
    tk = _tk()
    try:
        act = MarkerActuator(sandbox or (d / "sandbox"), op, CONTENT)
        res = enforce_chain(
            tk, la, token_handle=token, scope=SCOPE, target="write", effect_class="create_marker_v1",
            operation_hash=op, consumer="ag:main", eligibility_reference=eligibility, capability_id=op,
            valid_until=1000, now=10, durable_path=durable, actuator=act, crash_at=crash_at)
        (d / "result.json").write_text(json.dumps(res, indent=2, sort_keys=True) + "\n")
        (d / "reconstruct.json").write_text(json.dumps(reconstruct(durable), indent=2) + "\n")
        (d / "reconstruct_composed.json").write_text(
            json.dumps(reconstruct_composed(durable), indent=2, sort_keys=True) + "\n")
        return res, la, tk
    finally:
        la.close()


def main():
    # 1. success
    _run("success", op="op-success")[0]
    # 2. refusal — eligibility recombination; chain refuses before consume (durable infra_refusal).
    _run("refusal", op="op-refusal", eligibility="sha256:WRONG")
    # 3. crash after effect; reconcile recovers success from the marker.
    _run("crash-reconcile", op="op-crash", crash_at=CRASH_AFTER_EFFECT)
    # 4. replay refusal — consume once, then replay the same operation.
    la, token = _la()
    tk = _tk()
    d = HERE / "replay-refusal"
    d.mkdir(parents=True, exist_ok=True)
    durable = d / "durable.jsonl"
    if durable.exists():
        durable.unlink()
    shutil.rmtree(d / "sandbox", ignore_errors=True)
    try:
        common = dict(token_handle=token, scope=SCOPE, target="write", effect_class="create_marker_v1",
                      operation_hash="op-replay", consumer="ag:main", eligibility_reference=ELIG,
                      capability_id="op-replay", valid_until=1000, now=10, durable_path=durable)
        first = enforce_chain(tk, la, actuator=MarkerActuator(d / "sandbox", "op-replay", CONTENT), **common)
        second = enforce_chain(tk, la, actuator=MarkerActuator(d / "sandbox", "op-replay", CONTENT), **common)
        (d / "result.json").write_text(json.dumps({"first": first, "second_replay": second}, indent=2, sort_keys=True) + "\n")
        (d / "reconstruct.json").write_text(json.dumps(reconstruct(durable), indent=2) + "\n")
        (d / "reconstruct_composed.json").write_text(
            json.dumps(reconstruct_composed(durable), indent=2, sort_keys=True) + "\n")
    finally:
        la.close()
    print("receipt bundle generated under", HERE)


if __name__ == "__main__":
    main()
