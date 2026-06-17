# Correspondence log

Where a Lean obligation has an observed, executable correspondence in this kernel. Entries here are
*realized* mechanisms; promotion of an obligation to `CORRESPONDS` in `LEAN_OBLIGATIONS.md` requires a
passing hostile specimen.

## NoFreeStandingBridge — PARTIAL (A1)

The admission candidate carries two separately-constructed proofs with no coercion between them:
`transition_core::ReadVerdict` (read-side: standing verified + admission ok) and
`transition_core::WriteStanding` (write-side: no seam refused the mutation). Both have private `_seal`
fields and are constructed only inside `decide`'s admit arm; there is no `From<ReadVerdict> for
WriteStanding` (or the reverse) and no `Default`. `AdmissionCandidate::new` requires *both* by value.

Mirrors ratified `Execution.lean::AuthorizedStep`, which carries `stepAllowed` ∧ `authorityAuthorized` as
distinct proofs. Not yet `CORRESPONDS`: no hostile specimen yet attempts to forge a candidate from a
single proof or bridge read→write standing. (Next: `specimens/no_free_bridge/`.)

## ContractionHinge — PARTIAL (A1)

The `replay-budget` corpus case reproduces the exactly-once refusal: a replayed `consumption_event_id`
yields `already_consumed` at `la_seam`, `effect_count` stays 1, `consumed` flips to false. The authority
artifact (the spend) is not laundered into a second occurrence. Not yet `CORRESPONDS`: the move-only /
single-use *capability type* is Stage 3; A1 only reproduces the refusal at the projection level.

## Stage 3b2 — the first real bounded effect (through the live supervisor)

The fake actuator is replaced by one deliberately boring idempotent effect — exclusive `create_new` of a
fixed-content marker beneath a sandbox root (`create_marker_v1`: no shell, no overwrite, no general
write). Ordering: `AuthorizedTransition → correspondence → durable LA consume → durable
EffectAttemptReceipt → create_new(marker) → durable terminal outcome`.

Terminal vocabulary (`EffectSucceeded` requires post-effect verification — the path exists and the
content hash matches; a returned write success is not testimony): `NotConsumed+Refused`,
`Consumed+EffectSucceeded`, `Consumed+EffectFailed`, `Consumed+OutcomeUnknown` (+`Conflict` for wrong
content — never success).

All five crash points are replay-legible (`test_transition_enforce_3b2.py`): before consume → not spent;
after consume / after attempt → `OutcomeUnknown`; after effect → reconcile from the marker (exact →
recovered `EffectSucceeded`; absent → non-effect terminal; wrong content → `Conflict`, never success);
after success → ordinary replay refusal. Reconciliation classifies the past; it never repeats the effect
or mints authority.

**Live supervisor `enforce`** (`test_runtime_transition_probe_enforce.py`): the full chain must produce
an `AuthorizedTransition` (a Rust `Admit` alone is insufficient); any refusal / failed revalidation /
failed correspondence / unavailable kernel-or-LA denies. The legacy lab_gate route is **structurally
unreachable** from the enforce branch (it returns; never falls through). `disabled`/`observe`/`hold`/
`enforce` remain distinct.

> **Claim earned: at-most-once authority consumption and replay-legible execution of one idempotent
> bounded effect through the live AG supervisor.** Not generic exactly-once external effects; not yet a
> general actuator framework. One dull marker file has constitutional rights before `rm -rf` does.

## Stage 3b1 — live accounting consequence (LA owns the burn)

The real chain now spends capacity (`operational: true`), actuator still fake. Orchestrated in
`agent_gov/src/governor/runtime/transition_enforce.py` over real `la_cli` + `transition-cli`:
`issue_capability → correspondence_check (no burn) → LA consume → durable ConsumeReceipt → fake actuator`.

Proven (5 tests, `agent_gov/tests/test_transition_enforce_3b1.py`):
- **LA is the sole authoritative burn** — the receiver only checks correspondence (no burn); a replayed
  `consumption_event_id` refuses via LA `AlreadyConsumed`, not a second notion of spent.
- **Fail-closed, never observe** — kernel/LA unavailable → refusal, no consume, nothing durable.
- **Exact `eligibility_reference` survives end-to-end** (mint → bundle → correspondence echo → consume).
- **Consume receipt durable BEFORE any effect attempt**; **process death after consume reconstructs
  `spent_outcome_unknown`** (never replay permission), and a replay still refuses.

Claim fence: **at-most-once authority consumption with replay-legible effect outcome** — not exactly-once
external effect. The deliberate first real bounded effect (+ the terminal `Consumed×Effect{Succeeded|
Failed|Unknown}` / `NotConsumed×Refused` vocabulary) is Stage 3b2.

## Stage 3a — the consequence chain to a fake receiver

The full type chain is realized without a real effect:
`candidate (decide) + LA SpendCapability + ExecutionRevalidation + OperationContext → AuthorizedTransition
→ ReceiverGate → FakeActuator (HandlingReceipt)`.

- **SpendCapability lives in LinearAccountant** (`la.issue_capability`, `la_cli issue_capability`):
  single-use, nonce-bearing, scoped, target/effect-class bound, expiring; binds the granted token's
  opaque `eligibility_reference` verbatim. transition-kernel holds only a wire mirror (`LaCapability`).
- **Anti-recombination at the binding** (`AuthorizedTransition::finalize`): the capability's
  `eligibility_reference` must equal the candidate's `standing_ref` (never recomputed), with scope/target
  match and same-Standing revalidation. A capability minted against a different Standing cannot finalize
  (`EligibilityMismatch`).
- **ExecutionRevalidation** rechecks Standing liveness + freshness at the *execution* clock
  (`StaleAtExecution` / `StandingNotLive`).
- **Receiver gate** validates exact correspondence (operation hash, consumer, scope, target, effect
  class, nonce, expiry, single-use state) and burns the capability; a valid key for effect A cannot be
  replayed for effect B, and a burned capability refuses `already_burned`. The actuator is fake (no real
  effect). 11 hostile tests (`tests/stage3a_receiver.rs`) + the `execute_chain` CLI mode pass.

Everything is `operational: false`: accepted means the receiver *would* admit; nothing moved. The
deliberate flip to a real bounded effect (+ `enforce`, + LA consume thaw) is Stage 3b.

## GAP-3 memory custody — CORRESPONDS (MultiConsumerAdoption, NoFreeStandingReadout)

`memory_custody` makes `remembered → relied_upon` structurally unconstructable. `ReliedUpon` is sealed
(private `_seal`, no `From<MemoryArtifact>`, no `Default`); the sole constructor is `promote`, which
refuses unless an explicit `PromotionReceipt` raises the artifact to `MayRely` AND every custody check
passes: consumer indexing (presenter == intended *and* authorized consumer), freshness/supersession at
*presentation* time, scope, and digest/subject coverage. Closed refusal set: `no_promotion_to_rely`,
`promotion_does_not_cover`, `consumer_mismatch`, `stale_or_superseded`, `scope_mismatch`.

`assess` has no third case: a presented claim is `Relied` (→ `StandingEligible`) or `Advisory`
(→ `AdvisoryOnly`, which maps to `StandingOutput::Required` — no standing). The 8 hostile unit tests
(`tests/frontier_gap3.rs`) + 4 frontier vectors (`vectors/frontier/gap3/`) pass; the consumer-mismatch
and no-free-readout obligations are promoted to `CORRESPONDS` in the ledger. TemporalCustody is `PARTIAL`
(freshness realized at the memory rely boundary; the transition execution clock is still Stage 3).

This is upstream of Stage 3 by design: the inputs to a future `AdmissionCandidate → SpendCapability →
effect` chain cannot be laundered through memory, so when consequence turns on it turns on clean inputs.

## Structural: candidate ≠ authority (A1)

`AuthorizedTransition` (the only execution-crossing object) is unconstructable from the A1 decision
stage: its only constructor (`authority::AuthorizedTransition::bind`) requires a `SpendCapability` and an
`ExecutionRevalidation`, neither of which A1 can produce. `operational: false` is therefore a structural
property — observed: `cargo build` reports `bind` as dead code precisely because the authority-minting
path does not yet exist. This is the type-level form of "the courthouse issues no loading-dock keys at
its opening ceremony."
