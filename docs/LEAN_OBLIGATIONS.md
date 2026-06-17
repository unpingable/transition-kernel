# Lean obligation ledger

The unratified Lean corpus (`~/git/lean`) is this kernel's **experimental specification backlog**. The
dependency is strictly one-way:

> `unratified Lean fragment → candidate obligation → Rust mechanism + hostile specimen → observed
> correspondence or failure → ratification.`

The forbidden inversion — `Lean theorem → Rust copies a theorem-shaped API → "validated"` — is ceremonial
self-affirmation with types and is not permitted. A green build here is **never** evidence that a Lean
theorem holds. (This fence is already doctrine on the Lean side: `lean/.../V2.0-EXIT-CRITERIA.md` states
agent_gov correspondence is "external operational evidence, NOT a verified reduction," and "Rust
verifying Lean" is out of scope of the ratification basis.)

Lean marks ratification with an executable `Custody-Class`: `PUBLIC-SHIPPED` = ratified;
`UNRATIFIED-CANDIDATE` + `SCRATCH` = the candidate feedstock below. All listed theorems are
theorem-bearing (no `sorry`); "unratified" means not-yet-promoted under composition pressure.

## Enforcement modes

- **(a) type exclusion** — make the prohibited conversion structurally unconstructable.
- **(b) runtime refusal obligation** — where static exclusion is impossible (e.g. an execution-time
  clock), require an explicit refusal.
- **(c) conformance specimen** — an executable hostile case showing the theorem matters under
  composition, custody, time, and effects.

## Disposition vocabulary

Per obligation: `UNREALIZED` (named, no mechanism yet) · `PARTIAL` (mechanism exists, incomplete) ·
`CORRESPONDS` (mechanism + passing hostile specimen demonstrates the correspondence). Promotion to
`CORRESPONDS` requires an executable correspondence, never assertion.

## The ledger

| Obligation | Lean (file:line) · class | Mode | What it constrains here | Rust artifact | Disposition |
|---|---|---|---|---|---|
| **TemporalCustody / stale-at-execution** | `Scratch/TemporalCustody.lean:123` · SCRATCH | (b)+(c) | freshness re-checked at the boundary, not inherited from citation; refuse stale-at-use | `memory_custody::promote`, `authority::ExecutionRevalidation` | **PARTIAL** — memory rely boundary: `promote` refuses `stale_or_superseded` (`frontier/gap3/04`). Transition **execution** boundary: `ExecutionRevalidation::revalidate` now refuses `StaleAtExecution`/`StandingNotLive` at the execution clock (Stage 3a; `stage3a_receiver`). Live-Standing wiring at consume is 3b. |
| **NoFreeStandingBridge** (+ NoFreeLift, NoFreeStandingReadout) | `Admissibility/NoFreeStandingBridge.lean:136` · UNRATIFIED-CAND | (a) | the admission candidate carries two distinct proofs (read-verdict ⟂ write-standing) with no coercion between them | `transition_core::{ReadVerdict, WriteStanding, AdmissionCandidate}` — no `From`, private seals | **PARTIAL** — both proofs are distinct, sealed, non-coercible; not yet exercised by a hostile bridge specimen. |
| **RetroactiveLegitimation / FigLeaf** | `Admissibility/RetroactiveLegitimation.lean:199` · UNRATIFIED-CAND | (a) | the authorizing witness must be typed over the *pre-state*; a post-state witness must not typecheck | — | **UNREALIZED** — Stage 3 (the amend/authorize path) does not exist yet. |
| **MultiConsumerAdoption / observer standing** | `Scratch/MultiConsumerAdoption.lean:106` · SCRATCH | (a) | adoption/standing token parameterized by consumer; A's adoption can't discharge B's obligation | `memory_custody::promote` + `frontier/gap3/03` + GAP-3b live splice | **CORRESPONDS (component)** — workbench: `promote` refuses `consumer_mismatch`; live: the supervisor derives Standing through custody (GAP-3b observe/hold). **Full operational correspondence reserved for `enforce` (Stage 3)** — until then the legacy cooked-context route is observable/holdable, not yet barred. |
| **AmendmentFragment / founding-does-not-bless** | `Admissibility/AmendmentFragment.lean:585` · UNRATIFIED-CAND | (a)+(c) | an amend-policy step requires a pre-state validation token; self-grant unconstructible. Formal anchor for the operator/ratification root | `NON_CLAIMS.md` (root non-claim) | **UNREALIZED** — named in NON_CLAIMS; no amendment path built. |
| **ContractionHinge** | `Admissibility/ContractionHinge.lean:269` · UNRATIFIED-CAND | (c)→(a) | duplication/reuse of an authority-bearing artifact (LA spend token, continuation, replayed receipt) is refused, not assumed harmless | `replay-budget` corpus + LA `SpendCapability` + 3b1 live consume | **CORRESPONDS (operational)** — 3b1: LA is the sole authoritative burn; a replayed `consumption_event_id` refuses via LA `AlreadyConsumed`, proven over real `la_cli` (`test_transition_enforce_3b1`). Capacity is actually spent at-most-once. |
| **StaleEvidenceMerge** | `Admissibility/StaleEvidenceMerge.lean:250` · UNRATIFIED-CAND | (b)+(c) | merge re-evaluates the inherited freshness horizon; refuse `merged_stale` even when each branch was locally fresh | — | **UNREALIZED** — no merge/continuation path yet (GAP-2). |
| **Standing-as-readout / no-free-readout** | `Scratch/NoFreeStandingReadout.lean:102` · SCRATCH | (a) | a relied-upon value is unconstructable except from a stipulated root/promotion; "readable/present" yields no standing | `memory_custody::{ReliedUpon, promote, assess}` + `frontier/gap3/02` + GAP-3b live splice | **CORRESPONDS (component)** — workbench: `ReliedUpon` sealed, unpromoted memory → `Required` (no standing); live: GAP-3b routes the supervisor's Standing through custody (unpromoted inherited memory → no candidate). **Full operational correspondence reserved for `enforce` (Stage 3).** |

## How to advance an obligation

1. Build the smallest hostile specimen under `specimens/<obligation>/`.
2. Record the observed result in `CORRESPONDENCE.md` (realized) or `NON_CORRESPONDENCE.md` (gap /
   missing distinction / falsification).
3. Update the disposition column here. Promotion to `CORRESPONDS` requires the specimen to pass.
4. Ratification (removing an obligation from this backlog) additionally requires the Lean side to promote
   the theorem out of `UNRATIFIED-CANDIDATE`/`SCRATCH` — survival under both formal proof and hostile
   operational composition. A green build here is necessary, never sufficient.
