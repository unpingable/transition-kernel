# NEXT

```
next_action: GAP-2 complete (C1→C2→C3) — await operator for next frontier
```

The summit `stage3b2-first-effect` is sealed; its follow-on **Stage 3c — composed receipt snapshot
coherence** pins the exact admission snapshot that governed the effect (see `CORRESPONDENCE.md` → Stage 3c).

**GAP-2 C1 — the pure continuation office — is built and verified (workbench).** `src/continuation.rs`:
`decide_continuation(ContinuationRequest, ContinuationPolicy) → Grant | Refuse | Escalate`. The
`ContinuationGrant` is sealed (no public constructor / `Default` / `From`) — **not constructible from
narrative state** — single-use, session/actor/chain-tip/scope/next-step-class bound, expiring, and
non-transferable; `ContinuationConsumer` owns the single-use burn (the receiver-gate analog). The hinge
holds: a clean `Consumed + EffectSucceeded` is necessary but **not sufficient**. 20 hostile tests
(`tests/gap2_c1_continuation.rs`), all 12 named vectors.

**GAP-2 C2 — the live observe/hold splice — is built and verified.** transition-cli gained a
`continuation` mode (office only — no consumer, so no burn is reachable). In `agent_gov`,
`ContinuationProbe` (disabled/observe/hold) sits at the next-step boundary in the supervisor *before* the
per-effect transition gate: `observe` records the decision and lets the legacy loop continue (a non-grant
is a loud divergence — where the old loop self-authorizes continuation), `hold` records and stops, and
`disabled` is byte-identical baseline. **No grant burn, no LA consume, no effect** merely because C2
observed. 10 acceptance tests (`agent_gov/tests/test_runtime_continuation_probe_c2.py`).

**GAP-2 C3 — continuation enforce — is built and verified.** A grant must be presented and durably
**burned** before the next step reaches the transition gate; no grant → no next step (the legacy route is
structurally unreachable from the enforce branch). The burn is durable, not in-memory (the LA lesson):
`agent_gov` `continuation_enforce.py` owns the ledger — `present_continuation` (correspondence + durable
reuse check + burn), `finalize_continuation`, `reconstruct_continuation` (`unused` |
`spent_outcome_unknown` | `admitted_to_transition`). Burn semantics are sharp: a grant authorizes one
next-step **attempt**, not one successful effect — it stays spent even if the downstream effect gate
refuses, and a crash after the burn replays as `continuation_spent_outcome_unknown` (not retryable). 12
hostile/durability tests (`agent_gov/tests/test_continuation_enforce_c3.py`); full transition+continuation
suite green (55). Coordinated state: agent_gov `main` @ `80f7f47`. Nothing pushed.

> **Claim earned (GAP-2):** a supervised agent can take a further governed step only by presenting a
> single-use, receipt-bound continuation grant. The loop no longer renews itself by narrative momentum —
> the agent must show a receipt to keep being an agent. This is real AG-on-AG; not yet full
> self-governance.

**Await operator for the next frontier.** Named, not yet built:
- The two-layer **issue-path** custody gradient — `ContinuationGrantWire → VerifiedContinuationGrant →
  AuthorizedContinuation` (mirroring `LaCapability → (VerifiedLaCapability) → AuthorizedTransition`) —
  matters once an *external* issuer is in the loop, or the leaf-authenticity seam reopens wearing a new
  hat (see `NON_CLAIMS.md` / README "Trust perimeter"). C1–C3 make the grant unforgeable and
  reuse/transfer/scope-expansion/expiry refused, but the issuer is still trusted.
- The composed continuation chain has no Lean obligation yet — same proof→world order Stage 3c followed:
  the deployed enforce gate is now a forcing consumer that a future theorem could justify.

Frozen claim (effect layer): *at-most-once authority consumption and replay-legible execution of one
idempotent bounded effect through the live AG supervisor.*
