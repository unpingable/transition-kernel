# NEXT

```
next_action: GAP-2 C2 (live supervisor observe/hold) — await operator
```

The summit `stage3b2-first-effect` is sealed; its follow-on **Stage 3c — composed receipt snapshot
coherence** pins the exact admission snapshot that governed the effect (see `CORRESPONDENCE.md` → Stage 3c).

**GAP-2 C1 — the pure continuation office — is built and verified (workbench).** `src/continuation.rs`:
`decide_continuation(ContinuationRequest, ContinuationPolicy) → Grant | Refuse | Escalate`. The
`ContinuationGrant` is sealed (no public constructor / `Default` / `From`) — **not constructible from
narrative state** — single-use, session/actor/chain-tip/scope/next-step-class bound, expiring, and
non-transferable; `ContinuationConsumer` owns the single-use burn (the receiver-gate analog). The hinge
holds: a clean `Consumed + EffectSucceeded` is necessary but **not sufficient** — a refused/unknown
terminal, a stale version, a scope expansion, a wrong chain tip, or a reused grant all refuse, and an
unknown next-step class escalates. 20 hostile tests (`tests/gap2_c1_continuation.rs`), all 12 named
vectors. No live wiring, no enforce.

**Next: GAP-2 C2** — wire the continuation office into the live supervisor at `observe`/`hold` only (still
no enforce), then **C3** — `enforce`. The order mirrors Stage 3 (`observe/hold` → `enforce`): only after
the live splice is observable/holdable does continuation actually gate a step.

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
