# Provenance

This project is human-directed and AI-assisted. Final design authority,
acceptance criteria, and editorial control rest with the human author. AI
contributions were material and are categorized below by function.

## Human authorship

The author defined the project direction, requirements, and design intent — in
particular the staged discipline by which each new authority was turned on only
at a reviewed boundary (A1 → A2/A2b → GAP-3/3b → Stage 3a → 3b1 → 3b2), and the
structural-guarantee rule that a stage's types must not grant more authority than
it holds. AI systems contributed proposals, drafts, implementation, and critique
under author supervision; they did not independently determine project goals or
deployment decisions.

## AI assistance

AI assistants drafted the Rust kernel (`transition_core`, `authority`,
`receiver_gate`, `memory_custody`, `chain`, `live`), the legacy-corpus adapter,
the conformance/frontier/Stage-3 tests, and the cross-repo orchestration in
`agent_gov` and `linearaccountant`. All correspondences to the Lean substrate are
tracked one-way in `docs/LEAN_OBLIGATIONS.md`: a green build here is never evidence
that a Lean theorem holds.

## Relationship to the constellation

`transition-kernel` is the governed-transition office in a multi-repo governance
stack. It composes with `standing`, `linearaccountant` (the burn authority),
`wicket` (advisory), and `wlp` (receiver doctrine), and is exercised by
`agent_gov` as the first reflexive workload. See `README.md` for the office map.
