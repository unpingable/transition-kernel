# NON_CLAIMS

What `transition_kernel` is **not**, stated so the office is never read as more than it is.

- **It does not own the governance root.** The constitutional tier does not start here. An
  operator/ratification root installs kernel versions, the policy bundle, and delegations; this office
  operates *under* delegated authority and cannot self-found. (Formal anchor: Lean
  `AmendmentFragment` / `founding_does_not_bless`.)
- **It does not establish standing.** Standing is fetched and verified from the `standing` office; this
  office consumes a *reference* to an already-issued Standing receipt, never mints one.
- **It does not mint or spend capacity.** Capacity is LinearAccountant's. This office binds a reference
  to an LA-issued capability; it never decrements stock or burns a token itself.
- **It does not execute effects.** The receiver gate + actuator do. This office emits at most an
  admission *candidate*; it holds no effect surface.
- **It is not Wicket, and does not redeem Wicket's verdict.** A Wicket `Authorized` outcome is
  *advisory input*; this office **re-decides**. It never treats a Wicket receipt as the grant.
- **It does not validate Lean.** The dependency is one-way: unratified Lean fragments inform candidate
  obligations; a green build here is never evidence that a Lean theorem holds.
- **"Belief is free; consequence is paid"** — and inheritance is not free either. This office governs
  the *conversion* of a presented claim into consequence, never internal cognition, prose, or
  scratch state. The honest claim is "generative capability without generative authority."
- **The decision stage is non-operational by construction.** It returns
  `TransitionDecision::Admit(AdmissionCandidate)`; an `AuthorizedTransition` (the only object that can
  cross into execution) is *unconstructable* until LA capability is bound and execution-time
  revalidation passes. `operational: false` is a structural property, not a flag.
- **"Unconstructable" is leaf-relative — internal lineage is unforgeable by construction; leaf
  authenticity is vouched for upstream.** The type graph guarantees that a candidate-for-X cannot be
  paired with a capability-minted-for-Y (anti-recombination) and that an `AuthorizedTransition` cannot be
  built without both proofs, a bound capability, and a passed revalidation. It does **not** guarantee the
  *truth* of the leaves: `LaCapability` is an open `pub`/`Deserialize` wire mirror (LinearAccountant is
  the sole real minter); `ExecutionRevalidation::revalidate` proves the check *ran and passed with these
  inputs*, never that `live`/freshness are cosmically accurate; actuator idempotency is the actuator's
  contract, not the kernel's. The trust perimeter is LA + the bundle-assembling supervisor, by design.
  This is the same cap on generalization as the Lean side's per-bridge `BridgeValid`: the load-bearing
  obligation lives outside the verified object, and we say so rather than hide it.
