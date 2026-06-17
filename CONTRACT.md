# CONTRACT — the transition office's decision surface

This is the contract `transition_kernel` fills. The authority over it is the frozen corpus
(`vectors/legacy/*.json`, schema `agent_governor.corpus.v1`), exported from `agent_gov`. The Rust kernel
must reproduce these verdicts byte-for-byte; divergence emits a classified receipt **and fails the run**
unless an operator divergence manifest accepts it.

## Input

A corpus case carries an `input` block (a scenario *selector*, not raw office artifacts):

```json
{ "scenario": "all-green", "origin_mode": "drill",
  "override_origin_mode": null, "confabulate_citation": null }
```

- `scenario` — one of the 8 `SUPPORTED_SCENARIOS` (alias `already-consumed → replay-budget`).
- `origin_mode` — the declared simulated origin (`drill`, ...).
- `override_origin_mode` — case 07 only: rebuild the finding under this origin (`synthetic`).
- `confabulate_citation` — D3 only (`standing`/`evidence`); **out of scope** for the 9-case contract.

The adapter expands a scenario into typed office-output fixtures (`OfficeOutputs`); `transition_core`
composes those into a decision. The office outputs are *inputs*; the composition is the kernel's clause.

## Output — the seven frozen fields

| Field | Type | Source |
|---|---|---|
| `outcome` | `consumed \| refused \| gap_accounted` | decision + gap presentation |
| `refusal_kind` | one of the closed 12, or `null` | decision |
| `refusing_seam` | `standing_seam \| standing_spendability_seam \| la_seam \| wicket_seam \| proposal_validator_seam`, or `null` | decision (+ gap label) |
| `effect_count` | int (linearity) | mechanics |
| `consumed` | bool | mechanics |
| `operational` | bool | Wall-1 fence: `origin_mode == "observed"` (never true in corpus) |
| `proposal_packet_present` | bool | `(outcome ∈ {consumed, gap_accounted}) ∧ consumed` |

Receipt IDs are **not** part of the contract (content-addressed, environment-shaped). Standing receipt
digests are non-reproducible (random UUID + wall-clock in the hashed body) and are treated as opaque
pointers, never recomputed from content.

## Closed vocabularies

- **outcome** (3): `consumed`, `refused`, `gap_accounted`.
- **refusal_kind** (12, = `linear_accountant_client.CLOSED_REFUSAL_KINDS`): `standing_required`,
  `standing_expired`, `admission_denied`, `admission_gap_accounted`, `capacity_refused`,
  `already_consumed`, `dangling_receipt_reference`, `token_expired`, `token_revoked`, `unknown_token`,
  `scope_mismatch`, `standing_before_spendability_not_bounded`.
- **refusing_seam** (5): `standing_seam`, `standing_spendability_seam`, `la_seam`, `wicket_seam`
  (gap-accounted only — a non-refusal presentation label; see `docs/NON_CORRESPONDENCE.md`),
  `proposal_validator_seam` (D3 only).

## Invariants

1. **Wall-1 fence.** Every corpus input is a simulated origin (`drill`/`synthetic`), so `operational` is
   `false` in all nine — even the cases that mechanically consume. Running this corpus cannot mint a
   spendable/operational receipt.
2. **Candidate ≠ authority.** The decision stage yields `TransitionDecision::Admit(AdmissionCandidate)`;
   `AuthorizedTransition` is unconstructable here (`NON_CLAIMS.md`). `operational: false` is structural.
3. **Seam precedence.** Standing (inside wicket) → standing-spendability gate → LA request/admission →
   LA consume → admit. First refusal short-circuits.
