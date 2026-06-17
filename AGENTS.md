# AGENTS.md — working notes for agents in this repo

`transition-kernel` is the governed-transition office: `cooked context → Admit(candidate) | Refuse |
Escalate`. It is the **sole minter of "admitted"** and the home of the anti-recombination invariant. Read
`README.md` for the office map, `NON_CLAIMS.md` for what this is *not*, and `CONTRACT.md` for the frozen
decision contract.

## Operating rules (these are load-bearing, not style)

1. **Structural guarantees over flags/names.** A stage must not be *typed* or *named* with more authority
   than it holds. The decision stage yields a non-authority `AdmissionCandidate`; `AuthorizedTransition`
   is unconstructable until LA binds a capability and execution-time revalidation passes. `operational`
   is structural, never a boolean someone flips.
2. **Adapter ≠ office.** `legacy_corpus_adapter` may reproduce the historical seven-field contract;
   `transition_core` answers only its own clause and recomputes no peer's semantics.
3. **LA owns the burn.** The receiver gate validates correspondence and may detect obvious replay, but
   `linearaccountant` is the sole authoritative "spent" ledger (`consumption_event_id` → AlreadyConsumed).
4. **Two corpora stay separate.** `vectors/legacy/` (settled Python semantics, byte-for-byte) vs
   `vectors/frontier/` (GAP-2/GAP-3 — may reject what Python permits). Never grandfather frontier into
   legacy.
5. **One-way Lean.** `unratified Lean → Rust forcing case → ratification`. A green build is never evidence
   a theorem holds. Keep `docs/LEAN_OBLIGATIONS.md` honest (component vs full-operational).
6. **Verification by exit code.** Judge `cargo test` / `pytest` by the bare exit code, never by eyeballing
   piped output.

## Verify

```bash
cargo test                          # decision conformance + frontier + Stage-3 chain
python3 scripts/differential.py     # Rust ≡ frozen corpus ≡ live Python (if agent_gov importable)
```

The live consequence chain (3b1/3b2) is orchestrated in `agent_gov`
(`src/governor/runtime/transition_enforce.py`) over the real `la_cli` + `transition-cli`.

## Claim (frozen at `stage3b2-first-effect`)

> At-most-once authority consumption and replay-legible execution of one idempotent bounded effect through
> the live AG supervisor.

Not generic exactly-once external effects; not a general actuator framework.
