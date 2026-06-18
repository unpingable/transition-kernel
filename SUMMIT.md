# Summit: `stage3b2-first-effect`

The first defensible vertical claim of the governance stack, sealed across three coordinated repositories.

## Claim

> At-most-once authority consumption and replay-legible execution of one idempotent bounded effect
> through the live AG supervisor.

Non-claims: not generic exactly-once external effects; not a general actuator framework.

## The vertical

```
memory custody -> standing/admission -> bounded capacity -> execution-time revalidation
  -> receiver correspondence -> authoritative burn (LinearAccountant) -> one real bounded effect
  -> replay-legible outcome
```

## Coordinated repo states (tag `stage3b2-first-effect` on each)

| Repo | Role | Commit |
|---|---|---|
| transition-kernel | governed-transition office | `fc082388c28567c51bdd6e1d2dd7cc3946e084cf` |
| linearaccountant | bounded capacity + authoritative burn | `a439b8adec46a4055f13ea8b91456fa7ca114f5b` |
| agent_gov | first reflexive workload (live supervisor) | `2c6fb675237a67fbd940526b2b231c8da9c52dc4` |

Verification at the summit: transition-kernel `cargo test` green; linearaccountant `cargo test` green;
agent_gov 411 runtime tests green (1 skipped). Canonical receipt bundle: `specimens/receipt-bundle/`.

## Follow-on: `stage3c-composed-snapshot` (composed receipt snapshot coherence)

A completeness pass on the shipped Stage 3 chain — not a new surface. The composed durable receipt now
pins the exact admission snapshot that governed the effect: `composed_snapshot::ComposedExecutionSnapshot`
(verified `eligibility_reference` + execution-clock revalidation facts + candidate basis + capability
nonce → one canonical `snapshot_hash`), recorded durably before the burn, referenced by the consume
receipt, and refused on any incoherence by `reconstruct_composed`. Closes the gap named in
`NON_CORRESPONDENCE.md`; realized in `CORRESPONDENCE.md` → *Stage 3c*.

> **Invariant earned:** a verifier reconstructs not just "capacity was consumed and an effect happened,"
> but the exact composed admission snapshot under which that effect was allowed.

Coordinated repo states (local; tag + final SHAs recorded by the operator at push time):

| Repo | Role | Commit |
|---|---|---|
| transition-kernel | composed snapshot object + canonical hash + emission | `e270b27` (kernel); docs/specimens follow in the sealing commit |
| agent_gov | durable snapshot record + coherence reconstruction | `44dc2e0` |
| linearaccountant | unchanged (`a439b8adec46a4055f13ea8b91456fa7ca114f5b`) | — |

Verification: transition-kernel `cargo test` green (11 new Stage 3c hostile tests); agent_gov transition
suite green (9 new Stage 3c tests, 22 total). Lean: `Scratch/ExecutionRevalidation.lean` is upstream
design evidence (proof→world fence intact), not a stated theorem over the public API.

## Next (named, not ratified)

With Stage 3c sealed, **GAP-2 — continuation** is the right next frontier: the kernel deciding whether the
agent earns another governed step (not merely whether one effect may happen). That advances the thesis. A
registry of safely constitutional file operations does not. (No Lean theorem until the GAP-2 operational
gate exists — same proof→world order Stage 3c followed.)
