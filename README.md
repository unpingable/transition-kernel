# transition-kernel

> **Status (2026-07-26): executable admissibility research specimen.** This
> repository is a sealed research office in the *classic* (agent_gov)
> governance lineage. It is **not** a prior version of Docket and is not
> deprecated: Docket's governed work runtime was built greenfield, compared
> against this repo afterward under an isolation-gated task, and the recorded
> verdict was "complementary organs, not rival implementations of one organ —
> nothing imported" (`docket` repo, `docs/governed-runtime/old-rust-comparison.md`).
> The live governance constellation's canonical offices are AG ng (authority),
> Docket (execution), NQ (testimony/claims/reliance), and Nightshift
> (proposal/orchestration); the agent_gov constellation framing below, and the
> live 3b consequence chain orchestrated from agent_gov, are frozen historical
> claims sealed at `stage3b2-first-effect`. Frontier work stays
> named-not-ratified per `NEXT.md`.

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
cargo test                 # conformance (13 frozen cases, byte-for-byte) + quarantined Lean specimen
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

### Trust perimeter (read this before reading "unconstructable" as more than it is)

`AuthorizedTransition` makes **internal lineage and recombination constraints unforgeable by
construction** — all fields module-private, the sole producer is `finalize`, and there is no public
constructor or struct-literal path even from the test crate. But "unconstructable" is *leaf-relative*.
**Authenticity of leaf artifacts — Standing assessments, `LaCapability`, revalidation facts (`live` /
freshness), and actuator idempotency — remains the responsibility of their issuing offices and the
trusted composition boundary** (LinearAccountant as the sole real capability minter; the supervisor that
assembles the bundle from real office outputs). `LaCapability` is a deliberately open `pub` + `Deserialize`
wire mirror ("we receive it; we never mint one"); the anti-recombination check (`capability
.eligibility_reference == candidate.standing_ref`) blocks *recombination*, not *fabrication* by whoever
already sits inside the trusted boundary. The `pub` / `pub(crate)` / private gradient **is** the
separation-of-offices map: a field's visibility tells you which office is responsible for vouching for it.

This is the same shape on three surfaces — Lean proves relative to a per-bridge `BridgeValid` it can't
discharge; composition proves relative to authentic Standing/LA leaves; effect replay-safety holds
relative to a per-actuator idempotency the kernel can't enforce. Each office mechanically enforces its own
clause and makes every external assumption visible at a typed boundary. That is more honest than
pretending the type graph proves facts about the world. Future hardening (named, not built): a sealed
`VerifiedLaCapability` (issuer-verified) distinct from the open `LaCapability` wire form — warranted only
if the bundle ever crosses an attacker-controlled transport or the model can influence raw serialized
fields.

## Status (summit: `stage3b2-first-effect`)

Each authority was turned on at a reviewed boundary, the legacy route progressively fenced and now
structurally unreachable under `enforce`:

- **A1** — decision-surface conformance: the frozen corpus byte-for-byte (13 cases today; nine at the
  original summit), verified Rust ≡ frozen ≡ live Python.
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
