# THEORY_COVERAGE

What of the governance theory this kernel currently exercises, and what it deliberately does not. This is
a coverage map, not a claim of completeness.

## Covered in A1 (decision surface)

- **Seam-precedence composition** of already-issued office outputs into `Admit | Refuse | Escalate`
  (`transition_core::decide`). Standing → standing-spendability → LA request/admission → LA consume.
- **Closed refusal taxonomy** (12 kinds) and **closed seam set**, reproduced byte-for-byte against the
  frozen contract and verified three ways (Rust ≡ frozen ≡ live Python).
- **Candidate ≠ authority** as a structural property (`operational: false` cannot be flipped from the
  decision stage; `AuthorizedTransition` is unconstructable).
- **Two distinct, non-coercible proofs** (read-verdict ⟂ write-standing) on the admission candidate
  (NoFreeStandingBridge, PARTIAL).
- **Exactly-once / linearity** at the projection level (`replay-budget` → `already_consumed`,
  `effect_count` stays 1) (ContractionHinge, PARTIAL).
- **The Wall-1 fence**: no simulated-origin chain is operational.

## Covered in GAP-3 (typed-memory custody)

- **`remembered → relied_upon` is unconstructable** without an explicit promotion receipt that survives
  custody (`memory_custody`): sealed `ReliedUpon`, single `promote` seam, closed refusal set.
- **Consumer-indexed adoption** (A's memory ≠ B's standing), **freshness/supersession at presentation**,
  **scope** and **coverage** checks. No third case: standing-eligible or advisory-only.
- **Feeds the non-operational kernel path**: `StandingEligible → StandingOutput::Verified`,
  `AdvisoryOnly → Required`. Still no `enforce`, no LA thaw, no effect capability.
- Frontier corpus (`vectors/frontier/gap3/`) kept separate from the legacy corpus.

## Covered in Stage 3a (consequence chain → fake receiver)

- **The full type chain** `candidate + LA SpendCapability + ExecutionRevalidation → AuthorizedTransition
  → receiver gate → fake actuator (HandlingReceipt)`, all `operational: false` (no real effect).
- **SpendCapability lives in LinearAccountant** (single-use, nonce, scope, target, effect-class, expiry,
  binds `eligibility_reference`); transition-kernel only consumes the wire form.
- **Anti-recombination binding** (capability eligibility == candidate standing_ref), **execution-clock
  revalidation** (stale/revoked-at-execution refuses), **receiver exact-correspondence** + **single-use
  burn**. 11 hostile tests; `execute_chain` CLI mode.

## Not covered (Stage 3b)

- The deliberate `operational:false → true` flip: real bounded effect + `EffectReceipt`, LA **consume
  thaw** (the real burn), live Standing re-verify at consume, and the `enforce` mode that bars the legacy
  route. Plus the 3b hostile set (process death between consume and effect replay-legible; actuator
  failure records spent/non-effect without inventing success; `enforce` fails closed on kernel
  unavailability and never degrades to `observe`).

## Not covered (named, deferred)

- **Live composition / consequence.** A1 uses scenario *fixtures* for office outputs; no live Standing
  verify, no LA consume, no receiver gate, no effect. (A2 adds live transport but still no consequence;
  Stage 3 adds the bound capability + receiver gate + EffectReceipt.)
- **Execution-time revalidation / the kernel's own clock** (TemporalCustody, UNREALIZED — see
  `specimens/temporal_custody/`).
- **Custody / delegation typing**, **`required_authority` typing**, **operator/ratification root**
  (AmendmentFragment) — named in `NON_CLAIMS.md`, unbuilt.
- **The frontier:** GAP-3 typed-memory custody (first), GAP-2 continuation transport. No vectors yet;
  they live in `vectors/frontier/`, never grandfathered into the legacy corpus.
- **D3 confabulation** (`dangling_receipt_reference` / `proposal_validator_seam`) — outside the 9-case
  contract; the adapter refuses it as out-of-scope.

## The one-way Lean discipline

Lean obligations are tracked in `docs/LEAN_OBLIGATIONS.md` with dispositions. A green build here is never
evidence a Lean theorem holds; promotion requires an executable correspondence here **and** promotion of
the theorem out of `UNRATIFIED-CANDIDATE`/`SCRATCH` on the Lean side.
