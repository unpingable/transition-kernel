# NEXT

```
next_action: GAP-2 C3 (continuation enforce: present/burn) — await operator
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

**Next: GAP-2 C3 — continuation enforce.** This is the actual "earns next breath" point: present the grant
to `ContinuationConsumer`, burn it single-use, and allow the next step only on admit. Mirrors Stage 3
(`observe/hold` → `enforce`). The clerk's renewal stamp starts being spent — once, and only when the
renewal actually controls the next step.

A still-open design choice for C2/C3 (named, not yet built): whether to harden the *issue* path against a
caller-fabricated request with the two-layer custody gradient

```
ContinuationGrantWire        # open representation from the issuing office
VerifiedContinuationGrant    # sealed result of issuer verification, consumer-bound
AuthorizedContinuation       # sealed composition with the prior chain tip
```

mirroring `LaCapability → (VerifiedLaCapability) → AuthorizedTransition`. C1 already makes the *grant*
unforgeable and reuse/transfer/scope-expansion refused; the gradient matters once an external issuer is in
the loop (C2/C3), or the leaf-authenticity seam reopens wearing a new hat (see `NON_CLAIMS.md` / README
"Trust perimeter").

Frozen claim: *at-most-once authority consumption and replay-legible execution of one idempotent bounded
effect through the live AG supervisor.*
