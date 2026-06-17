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

## Next (named, not ratified)

GAP-2 — continuation: the kernel deciding whether the agent earns another governed step. That advances
the thesis. A registry of safely constitutional file operations does not.
