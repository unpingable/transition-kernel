# transition-kernel

The missing **governed-transition office** in the agent_gov governance constellation: given a proposed
transition and its cited basis, decide `Admit(AdmissionCandidate) | Refuse(kind, reasons) |
Escalate(required_authority)`. It is the **sole minter of "admitted"** and the sole home of the
anti-recombination invariant. It does **not** establish standing, mint/spend capacity, execute effects,
ratify policy, or own the governance root (see `NON_CLAIMS.md`).

> *Kubernetes for admissibility:* narrow controllers, typed transitions, receipts as the durable
> substrate. The distinction that must survive — Kubernetes reconciles **toward** desired state; this
> stack **refuses** desired state that lacks standing, authority, custody, or spend capacity. **No
> controller owns the whole sentence from observation to effect.**

## 30-second specimen

```bash
cargo test                 # conformance (9 frozen cases, byte-for-byte) + quarantined Lean specimen
python3 scripts/differential.py   # Rust ≡ frozen contract ≡ live Python (if agent_gov importable)
```

The contract is `vectors/legacy/*.json` (`agent_governor.corpus.v1`), exported from `agent_gov`. The Rust
kernel is a *frozen-contract fill*: reproduce the seven verdict fields byte-for-byte, or a divergence
gets a classified receipt **and a failed run** (never an articulate green test) — see `CONTRACT.md`.

## Structure

- `src/transition_core.rs` — the kernel's clause: seam-precedence composition of typed office outputs
  into a decision. Recomputes no peer (Standing/Wicket/LA) semantics.
- `src/legacy_corpus_adapter.rs` — expands a scenario into office-output fixtures and projects the
  historical seven-field contract (the layer allowed to reproduce peer-owned fields + presentation
  quirks).
- `src/authority.rs` — `AuthorizedTransition`, **unconstructable** until Stage 3 binds an LA capability
  and passes execution-time revalidation. `operational: false` is structural, not a flag.
- `src/bin/transition-cli.rs` — fail-closed JSON boundary.
- `docs/LEAN_OBLIGATIONS.md` — the candidate-obligation ledger (one-way: Lean → Rust forcing case →
  ratification; a green build is never evidence a Lean theorem holds).
- `specimens/temporal_custody/` — the first quarantined Lean specimen (disposition `UNREALIZED`).

## Claim (frozen at `stage3b2-first-effect`)

> **At-most-once authority consumption and replay-legible execution of one idempotent bounded effect
> through the live AG supervisor.**

**Non-claims:** not generic exactly-once external effects; not a general actuator framework; the kernel
does not establish standing, mint/spend capacity (that is LinearAccountant), ratify policy, or own the
governance root. See `NON_CLAIMS.md`.

## Status (summit: `stage3b2-first-effect`)

Each authority was turned on at a reviewed boundary, the legacy route progressively fenced and now
structurally unreachable under `enforce`:

- **A1** — decision-surface conformance: nine corpus cases byte-for-byte, verified Rust ≡ frozen ≡ live
  Python.
- **A2 / A2b** — live, non-operational traversal through the real `agent_gov` supervisor (transport +
  orchestration, no consequence).
- **GAP-3 / GAP-3b** — typed-memory custody: `remembered → relied_upon` unconstructable without explicit
  promotion; wired into the live supervisor so Standing flows through custody.
- **Stage 3a** — full consequence chain to a fake receiver (capability binding, execution revalidation,
  receiver correspondence, single-use; no real effect).
- **Stage 3b1** — live LA consume: capacity actually spent, LA the sole authoritative burn, fail-closed,
  replay-legible.
- **Stage 3b2** — one real bounded effect (`create_marker_v1`) through the live supervisor `enforce`
  mode, with the full terminal vocabulary and crash-point reconciliation.

The end-to-end receipt bundle (success · refusal · crash/reconciliation · replay refusal) is the
canonical specimen under `specimens/receipt-bundle/`. The next serious frontier is **GAP-2** (continuation
— the kernel deciding whether the agent earns another governed step); a registry of file operations is
not. Named, not ratified.
