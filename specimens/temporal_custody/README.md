# Specimen: TemporalCustody / stale-at-execution

**Lean:** `Scratch/TemporalCustody.lean:123`
(`citation_validity_does_not_imply_execution_admissibility`) · Custody-Class `SCRATCH`
**Enforcement mode:** (b) runtime refusal obligation + (c) conformance specimen
**Disposition:** `UNREALIZED`

## The obligation

> A receipt valid when *cited* need not be admissible when *executed* later. Freshness must be re-checked
> at the **execution** boundary, not inherited from citation time.

## Quarantine status (read before judging this specimen)

This specimen is **fenced**:
- It is **not** part of the nine-case legacy conformance corpus.
- It is **not** required for A1 completion, and A1 does **not** claim to discharge this obligation.
- It is allowed to expose missing machinery.
- **But it is not permitted to rot:** it must compile, run deterministically, carry an explicit
  disposition, and **fail CI if its observed behavior changes without a matching update to this README
  and `docs/LEAN_OBLIGATIONS.md`.** The fence keeps it out of the green-bar *claim*, not out of the
  build.

## What is realized vs unrealized in A1

- **Realized (the gate's verdict is honored):** when the upstream standing-spendability gate reports
  `Unbounded` (the citation-time observation lapsed before spend), `transition_core::decide` refuses at
  `standing_spendability_seam` before any capacity is reserved (`effect_count = 0`). The hero specimen
  (corpus case 08) and its twin (09) traverse identical machinery and diverge only on the gap.
- **Unrealized (why the disposition is `UNREALIZED`):** the kernel does **not** own an execution clock.
  The staleness is computed by the *gate* (an upstream office) and arrives as a `SpendabilityOutput`
  fixture the kernel trusts. `transition_core::decide` takes no execution-time input and performs no
  independent re-check at spend time. TemporalCustody-as-a-kernel-obligation bites only once the kernel
  owns the execution boundary (Stage 3: `AuthorizedTransition` + `ExecutionRevalidation`). Until then the
  obligation is carried, visible, and unmet — by design, not by oversight.

## Promotion path

When Stage 3 gives the kernel an execution clock and an `ExecutionRevalidation` proof, extend
`specimen.rs` to drive a candidate that was fresh at citation and stale at the *kernel's own* execution
recheck, assert the kernel refuses on its own authority, move the disposition to `PARTIAL`/`CORRESPONDS`,
and record it in `docs/CORRESPONDENCE.md`.
