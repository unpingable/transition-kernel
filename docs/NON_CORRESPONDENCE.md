# Non-correspondence log

Where the implementation reveals a gap, a missing distinction, an accidental Python semantics, or a
falsification of a proposed correspondence. This is the honest other half of `CORRESPONDENCE.md` — the
place that keeps "byte-for-byte compatibility" from quietly becoming "we preserved the laundering seam."

## Candidate Python-accidental-semantics: `gap_accounted` reports a `refusing_seam` on a non-refusal

**Observed (A1).** Corpus case `05-gap-accounted` freezes `outcome: gap_accounted`,
`refusal_kind: admission_gap_accounted`, **`refusing_seam: wicket_seam`** — yet nothing refuses: the
chain fully consumes (`effect_count: 1`, `consumed: true`, proposal packet present). In `drill_runner.py`
this is a pure scenario-label assignment (`_classify_chain_outcome`), and the wicket office actually
returns plain `authorized` (admit-any).

**Why it's filed here.** A `refusing_seam` naming a seam that did not refuse is presentation, not
decision semantics. The honest `transition_core::decide` returns `Admit` for this office configuration;
the seam/label is applied in `legacy_corpus_adapter::project` solely to reproduce the frozen contract.
We reproduce it (A1 requires byte-for-byte) **and** flag it: this is a candidate for the
frontier/adversarial corpus, where the kernel may intentionally emit a non-refusal shape
(`outcome: gap_accounted`, `refusing_seam: null`) and classify the difference as
`python_accidental` — *not* grandfathered into the legacy corpus.

**Status.** Reproduced in legacy projection; not yet raised to an operator divergence manifest. No action
in A1 beyond this record.

## Gap: TemporalCustody has no execution-clock mechanism (UNREALIZED)

The kernel does not yet re-check freshness at an execution clock. In A1 the standing-spendability gate's
verdict arrives as an *upstream office output* (`SpendabilityOutput`) that the kernel trusts; the kernel
holds no clock of its own and performs no independent execution-time recheck. The obligation
(`LEAN_OBLIGATIONS.md` → TemporalCustody, mode (b)) bites only once the kernel owns the execution
boundary (Stage 3). See `specimens/temporal_custody/`.

## Gap: the read↔write proof distinction is real but untested by a hostile bridge

`ReadVerdict` and `WriteStanding` are distinct and non-coercible by construction, but no specimen yet
*attempts* the forbidden bridge (forging a candidate from one proof, or coercing read→write standing). Until
`specimens/no_free_bridge/` exists, NoFreeStandingBridge stays `PARTIAL`, not `CORRESPONDS`.

## Completeness obligation: the composed receipt does not pin the admission snapshot (Stage 3b)

**Named, not built. No code change filed with this entry.**

**Provenance.** Cross-constellation execution-time re-admission audit, 2026-06-18 (nightshift / standing /
linearaccountant / agent_gov / Lean). The audit's seam-2 question — "is the receipt a projection of the
*same evaluated snapshot* that governed the effect?" — is answered cleanly **somewhere**: `standing`'s
grant receipts are the gold standard (receipt built from the exact read snapshot, written in one atomic
transaction whose state-update is CAS-conditional on the unchanged head digest; the receipt *structurally
cannot* describe a different state than governed the effect). LA's consume receipt has the within-ledger
version (the receipt **is** the `Event`, recorded atomically in the decision branch).

**The gap.** transition-kernel's Stage 3 chain is the **first consumer that composes multiple offices'
re-admissions into one effect** (`ExecutionRevalidation` over standing → receiver correspondence → LA
consume → effect). Each office re-checks its own invalidating facts at its own clock and receipts its own
decision coherently — but the **composed** receipt chain
(`transition_enforce.py`: `consume_receipt` → `effect_attempt` → `effect_outcome`) **does not record the
admission/revalidation snapshot** (`live` / `valid_until` / the exact `eligibility_reference`) that
governed admission. So a verifier cannot confirm "the composed receipt describes the same standing-state
that governed this effect." The `ExecutionRevalidation` facts are *checked* and then *discarded*; only the
LA decision and the effect outcome are durable. (This composed-coherence gap is distinct from the
per-office coherence each kernel already has, and from the leaf-authenticity perimeter in `NON_CLAIMS.md`
— here even the facts the kernel *did* evaluate are not pinned into the receipt it emits.)

**Smallest forcing case (the fix, when ratified).** Record the admission snapshot into the
`consume_receipt`: the verified `eligibility_reference`, the revalidation `(live, valid_until, now)`, and
the capability nonce — i.e. **port `standing`'s already-built atomic-snapshot-receipt pattern to the
composed seam**. This is an *implemented-instance-to-port* + a completeness fix on a shipped slice
(Stage 3b shipped the composed chain without a snapshot-coherent composed receipt), **not** a new result.
Owner: this repo.

**Do not formalize in Lean yet (proof→world fence).** No compiled Lean theorem covers decision↔receipt
same-snapshot coherence; `Execution.lean` evaluates authority and applies the effect on one shared
`state` and emits no receipt, so the property is currently *unstatable* there, not merely unproven. A Lean
kernel needs a forcing consumer, not an elegant gap: the honest order is **make this composed receipt
snapshot-coherent here first (operational), and let that deployed gate become the forcing consumer** that
would later justify the theorem. (Sibling findings, owned elsewhere: AG's `receipt_kernel` has no
named "receipt verdict == decision verdict, same snapshot" invariant on the `receipt_bridge` event path —
`agent_gov/specs/gaps/`; `standing`'s `StandingDecision` carries no `body_digest`/`jti` — known MVP gap,
owned by `standing`.)
