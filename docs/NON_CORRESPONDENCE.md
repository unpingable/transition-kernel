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
