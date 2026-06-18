#!/usr/bin/env python3
"""GAP-2 C4: the two-step AG-on-AG trajectory specimen.

Not new mechanism — the C3 mechanism walked as a trajectory. One supervised agent executes a bounded
governed step, earns *exactly one* further governed step through a single-use continuation grant, and the
full two-step trajectory is replay-legible:

    step 1:  governed effect executes (initial delegation; no continuation required yet)
             -> composed Stage 3c snapshot + terminal outcome recorded
    bridge:  request next step -> grant issued -> durable BURN before the next transition gate
    step 2:  the next governed action reaches the transition gate ONLY because the grant was burned
             -> terminal chain recorded
    replay:  reconstructs both steps and the continuation bridge

Run over the REAL binaries (la_cli + transition-cli) and the real agent_gov orchestration
(transition_enforce + continuation_enforce). Produces three committed bundles:

    two-step-success/         the clean trajectory above
    no-grant-refused/         step 2 attempted WITHOUT a grant -> refused before the transition gate;
                              the legacy loop is unreachable (step 2 never consumes or acts)
    burned-but-effect-refused/  grant burned, but the downstream effect gate refuses -> the grant
                              remains spent; re-presenting it is `already_consumed` (no retry-by-renarration)

The continuation grant gates *entry* to step 2's transition gate; the effect itself still crosses the
normal LA-consume + bounded-effect chain. This script embodies that gate order (it runs step-2 enforce
only when the burn admitted), exactly as the C3 supervisor enforce branch does.
"""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path.home() / "git" / "agent_gov" / "src"))

from governor.runtime.continuation_enforce import (  # noqa: E402
    C_ADMITTED, finalize_continuation, present_continuation, reconstruct_continuation,
)
from governor.runtime.la_subprocess import DEFAULT_LA_CLI, LASubprocess  # noqa: E402
from governor.runtime.transition_enforce import MarkerActuator, enforce_chain, reconstruct  # noqa: E402
from governor.runtime.transition_subprocess import DEFAULT_TRANSITION_CLI, TransitionSubprocess  # noqa: E402

HERE = Path(__file__).resolve().parent
SCOPE, ELIG, CONTENT = "lab", "sha256:standing-xyz", b"create_marker_v1:fixed-bytes"
KERNEL_VERSION, POLICY_VERSION = "0.0.0", "policy.v1"
SESSION, ACTOR, NEXT_CLASS = "sess-trajectory", "ag:main", "create_marker_v1"
POLICY = {"current_policy_version": POLICY_VERSION,
          "admissible_next_step_classes": [NEXT_CLASS], "grant_ttl": 100}


def _la():
    la = LASubprocess(os.environ.get("GOVERNOR_LA_CLI", DEFAULT_LA_CLI))
    la.start()
    la.deposit(SCOPE, 9)
    dec = la.request_capacity({
        "request_id": "r1", "actor": ACTOR, "action": "write", "target": "write", "scope": SCOPE,
        "requested_capacity": 9, "eligibility_reference": ELIG, "eligibility_valid_until": 1000,
        "expires_after": 1000, "idempotency_key": None}, 10)
    assert dec["decision"] == "Granted", dec
    return la, dec["token_id"]


def _tk():
    return TransitionSubprocess(binary_path=os.environ.get("GOVERNOR_TRANSITION_CLI", DEFAULT_TRANSITION_CLI))


def _step(tk, la, token, *, op, durable, sandbox, eligibility=ELIG):
    """One governed effect through the real enforce chain. Returns the terminal result dict."""
    act = MarkerActuator(sandbox, op, CONTENT)
    return enforce_chain(
        tk, la, token_handle=token, scope=SCOPE, target="write", effect_class="create_marker_v1",
        operation_hash=op, consumer=ACTOR, eligibility_reference=eligibility, capability_id=op,
        valid_until=1000, now=10, durable_path=durable, actuator=act)


def _snapshot_from(durable: Path):
    """Read the composed Stage 3c snapshot (and its hash) the kernel pinned in this step's chain."""
    for line in durable.read_text().splitlines():
        r = json.loads(line)
        if r.get("kind") == "composed_snapshot":
            return r["snapshot"], r["snapshot_hash"]
    return None, None


def _continuation_request(snapshot, tip, *, outcome, scope=SCOPE, next_step=NEXT_CLASS):
    return {
        "prior_snapshot": snapshot, "terminal_outcome": outcome, "prior_chain_tip": tip,
        "session_id": SESSION, "actor_id": ACTOR, "requested_next_step": next_step, "scope": scope,
        "remaining_capacity_ref": "token", "policy_version": POLICY_VERSION,
        "kernel_version": KERNEL_VERSION, "now": 20,
    }


def _presentation(grant):
    return {"session_id": grant["session_id"], "actor_id": grant["actor_id"],
            "chain_tip": grant["chain_tip"], "scope": grant["scope"],
            "next_step_class": grant["next_step_class"], "now": 20}


def _fresh(name):
    d = HERE / name
    d.mkdir(parents=True, exist_ok=True)
    for f in ("effects.jsonl", "continuation.jsonl"):
        (d / f).unlink(missing_ok=True)
    shutil.rmtree(d / "sandbox", ignore_errors=True)
    return d


def _write(d, result):
    (d / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    (d / "reconstruct.json").write_text(json.dumps({
        "effects": reconstruct(d / "effects.jsonl"),
        "continuation": reconstruct_continuation(d / "continuation.jsonl"),
    }, indent=2, sort_keys=True) + "\n")


def two_step_success():
    """The canonical trajectory: step 1 effect -> grant burned -> step 2 effect reaches the gate."""
    d = _fresh("two-step-success")
    effects, cont = d / "effects.jsonl", d / "continuation.jsonl"
    la, token = _la()
    tk = _tk(); tk.start()
    try:
        # Step 1 — initial delegated governed effect.
        s1 = _step(tk, la, token, op="op-step1", durable=effects, sandbox=d / "sandbox")
        assert s1["result"] == "consumed_effect_succeeded", s1
        snapshot, tip = _snapshot_from(effects)
        assert snapshot is not None and tip is not None

        # Bridge — earn the next step. Office issues a grant; we durably BURN it before step 2.
        decision = tk.decide_continuation(_continuation_request(snapshot, tip, outcome=s1["result"]), POLICY)
        assert decision["decision"] == "grant", decision
        grant = decision["grant"]
        pres = present_continuation(cont, grant, _presentation(grant), 20)
        assert pres["result"] == "admitted", pres

        # Step 2 — reaches the transition gate ONLY because the grant was burned (this script runs the
        # step-2 chain only on an admitted burn, exactly as the supervisor enforce branch does).
        s2 = _step(tk, la, token, op="op-step2", durable=effects, sandbox=d / "sandbox")
        assert s2["result"] == "consumed_effect_succeeded", s2
        finalize_continuation(cont, grant["grant_id"])

        # Replay must reconstruct both steps and the bridge.
        eff = reconstruct(effects)
        assert eff == {"op-step1:op-step1": "consumed_effect_succeeded",
                       "op-step2:op-step2": "consumed_effect_succeeded"}, eff
        assert reconstruct_continuation(cont) == {grant["grant_id"]: C_ADMITTED}
        _write(d, {"trajectory": "two_step_success", "step1": s1, "grant_id": grant["grant_id"],
                   "step2": s2, "claim": "one bounded step + exactly one earned next step, replay-legible"})
    finally:
        la.close()


def no_grant_refused():
    """Hostile: step 1 succeeds; the agent proposes step 2 OUTSIDE the slice (scope expansion). The
    continuation office refuses -> no grant -> step 2 never reaches the transition gate."""
    d = _fresh("no-grant-refused")
    effects, cont = d / "effects.jsonl", d / "continuation.jsonl"
    la, token = _la()
    tk = _tk(); tk.start()
    try:
        s1 = _step(tk, la, token, op="op-step1", durable=effects, sandbox=d / "sandbox")
        assert s1["result"] == "consumed_effect_succeeded", s1
        snapshot, tip = _snapshot_from(effects)

        # Step 2 proposed with a widened scope -> the office refuses; there is no grant to burn.
        decision = tk.decide_continuation(
            _continuation_request(snapshot, tip, outcome=s1["result"], scope="lab+prod"), POLICY)
        assert decision["decision"] == "refuse" and decision["kind"] == "scope_expansion", decision

        # The gate is closed: step 2's effect chain is NOT run. Nothing burned, no step-2 effect.
        assert reconstruct_continuation(cont) == {}
        eff = reconstruct(effects)
        assert eff == {"op-step1:op-step1": "consumed_effect_succeeded"}, eff
        assert "op-step2:op-step2" not in eff
        _write(d, {"trajectory": "no_grant_refused", "step1": s1, "continuation_refusal": decision,
                   "step2": "never_reached_transition_gate"})
    finally:
        la.close()


def burned_but_effect_refused():
    """The nasty one: grant burned, but step 2's effect gate refuses (eligibility recombination). The
    grant remains spent; re-presenting it is `already_consumed` — no retry-by-renarration."""
    d = _fresh("burned-but-effect-refused")
    effects, cont = d / "effects.jsonl", d / "continuation.jsonl"
    la, token = _la()
    tk = _tk(); tk.start()
    try:
        s1 = _step(tk, la, token, op="op-step1", durable=effects, sandbox=d / "sandbox")
        assert s1["result"] == "consumed_effect_succeeded", s1
        snapshot, tip = _snapshot_from(effects)

        decision = tk.decide_continuation(_continuation_request(snapshot, tip, outcome=s1["result"]), POLICY)
        assert decision["decision"] == "grant", decision
        grant = decision["grant"]
        pres = present_continuation(cont, grant, _presentation(grant), 20)
        assert pres["result"] == "admitted", pres
        finalize_continuation(cont, grant["grant_id"])  # the next step reached the gate...

        # ...but the downstream effect gate refuses (capability minted against the WRONG eligibility ->
        # binding recombination refuses before any consume/effect).
        s2 = _step(tk, la, token, op="op-step2", durable=effects, sandbox=d / "sandbox",
                   eligibility="sha256:WRONG")
        assert s2["result"] == "not_consumed_refused", s2

        # The grant is still spent — the agent used its attempt. Re-presenting it does not refund it.
        assert reconstruct_continuation(cont) == {grant["grant_id"]: C_ADMITTED}
        retry = present_continuation(cont, grant, _presentation(grant), 20)
        assert retry["result"] == "refused" and retry["refusal"] == "already_consumed", retry

        # The effect ledger shows step 2 refused (durable infra_refusal), never consumed.
        assert reconstruct(effects) == {"op-step1:op-step1": "consumed_effect_succeeded"}
        _write(d, {"trajectory": "burned_but_effect_refused", "step1": s1,
                   "grant_id": grant["grant_id"], "step2_effect": s2,
                   "retry_present": retry, "claim": "grant spent on the attempt; no retry-by-renarration"})
    finally:
        la.close()


def main():
    two_step_success()
    no_grant_refused()
    burned_but_effect_refused()
    print("continuation trajectory specimen generated under", HERE)


if __name__ == "__main__":
    main()
