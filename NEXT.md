# NEXT

```
next_action: nothing queued — await operator
candidate_next: Branch B (issue-path custody gradient) — open only when an external issuer crosses the perimeter
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

**GAP-2 C4 — the two-step AG-on-AG trajectory specimen — is built and verified.** The C3 mechanism walked
as a trajectory, not more mechanism: `specimens/continuation-trajectory/` runs step-1 effect → grant
issued → durable burn → step-2 effect (gated by the burn) over the real binaries + `agent_gov`, with three
bundles (`two-step-success`, `no-grant-refused`, `burned-but-effect-refused`). Running `generate.py`
asserts every invariant, so it is a walk, not prose.

> **Claim earned (GAP-2):** a supervised agent executed a bounded governed step, earned exactly one
> further governed step through a single-use, receipt-bound continuation grant, and the full two-step
> trajectory is replay-legible. The loop no longer renews itself by narrative momentum — the agent must
> show a receipt to keep being an agent. Real AG-on-AG; not yet full self-governance.

**Branch A — Lean theorem feedstock — done (narrowly).** `lean/LeanProofs/Scratch/NoFreeContinuation.lean`
(hand-managed lean repo; authored + type-checks clean, axiom-clean to `propext`; left uncommitted there).
Models exactly the C4 spine, schematic over scope/class/version: (1) prior success does not imply
continuation authority (`EffectSucceeded ⊬ MayContinue`); (2) authority is chain-tip bound; (3,5)
single-use / no contraction / a burned grant cannot be replayed; (4) a grant authorizes an *attempt*, not
a success (burn persists when the downstream effect refuses). Seed: *a successful governed act does not
authorize the agent's next breath.* Ledger row: `LEAN_OBLIGATIONS.md` → NoFreeContinuation (forcing
consumer = the deployed C3 enforce gate + C4 trajectory; not ratified — proof→world fence).

**Await operator. Remaining named branch:**

- **Branch B — issue-path custody gradient.** `WireContinuationGrant → VerifiedContinuationGrant →
  AuthorizedContinuation` (mirroring `LaCapability → (VerifiedLaCapability) → AuthorizedTransition`).
  **Open only when an artifact crosses from external wire into trusted continuation/effect authority** —
  an out-of-office issuer, or a model/tool that can influence raw serialized grant fields. Today the
  issuer is the same trusted sealed office; building this first would polish the perimeter before there is
  an attacker-shaped path crossing it.

Frozen claim (effect layer): *at-most-once authority consumption and replay-legible execution of one
idempotent bounded effect through the live AG supervisor.*
