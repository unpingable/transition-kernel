//! Stage 3c: the composed receipt pins the admission snapshot that governed the effect.
//!
//! The chain already proves "capacity was consumed and an effect happened." These tests prove the
//! stronger composed-coherence property: a verifier can reconstruct **the exact composed admission
//! snapshot under which the effect was allowed**, and any receipt that describes a *different* admission
//! state than the one that governed the effect is rejected.
//!
//! The centerpiece is `pinning_snapshot_distinguishes_execution_snapshots`: the operational dual of the
//! Lean `lossy_receipt_cannot_pin_snapshot` model — two transitions over the *same lineage* but a
//! *different execution snapshot* must not collapse to the same receipt.

use transition_kernel::authority::{AuthorizedTransition, ExecutionRevalidation, LaCapability, OperationContext};
use transition_kernel::composed_snapshot::{
    candidate_basis_hash, ComposedExecutionSnapshot, SnapshotIncoherence, SNAPSHOT_SCHEMA,
};
use transition_kernel::transition_core::{
    decide, AdmissionCandidate, AdmissionOutput, ConsumeOutput, OfficeOutputs, SpendabilityOutput,
    StandingOutput, TransitionDecision,
};

const ELIG: &str = "sha256:standing-xyz";
const NONCE: &str = "nonce-7";
const OP: &str = "op-hash-abc";
const CONSUMER: &str = "ag:main";
const SCOPE: &str = "lab";
const TARGET: &str = "demo";
const EFFECT: &str = "create_marker_v1";

fn candidate_with(standing: &str, scope: &str, target: &str) -> AdmissionCandidate {
    let office = OfficeOutputs {
        standing: StandingOutput::Verified { receipt_ref: standing.into() },
        spendability: SpendabilityOutput::NotApplicable,
        la_admission: AdmissionOutput::Admitted,
        la_consume: ConsumeOutput::Consumed,
        scope: scope.into(),
        target: target.into(),
    };
    match decide(&office) {
        TransitionDecision::Admit(c) => c,
        other => panic!("expected admit, got {other:?}"),
    }
}

fn candidate() -> AdmissionCandidate {
    candidate_with(ELIG, SCOPE, TARGET)
}

fn capability() -> LaCapability {
    LaCapability {
        capability_id: NONCE.into(),
        scope: SCOPE.into(),
        target: TARGET.into(),
        effect_class: EFFECT.into(),
        eligibility_reference: ELIG.into(),
        expires_at: 1000,
        single_use: true,
    }
}

fn op_ctx() -> OperationContext {
    OperationContext { operation_hash: OP.into(), consumer: CONSUMER.into() }
}

/// An authorization whose revalidation was checked at `now` against horizon `valid_until`.
fn authorized_at(now: u64, valid_until: u64) -> AuthorizedTransition {
    let reval = ExecutionRevalidation::revalidate(ELIG, true, valid_until, now)
        .expect("standing live and fresh");
    AuthorizedTransition::finalize(candidate(), capability(), reval, op_ctx())
        .expect("clean lineage should finalize")
}

fn cev() -> String {
    format!("{OP}:{NONCE}")
}

fn snapshot() -> ComposedExecutionSnapshot {
    ComposedExecutionSnapshot::from_authorized(&authorized_at(10, 1000), cev())
}

// --- the snapshot describes what it governed -------------------------------- //

#[test]
fn snapshot_captures_the_verified_admission_state() {
    let s = snapshot();
    assert_eq!(s.schema, SNAPSHOT_SCHEMA);
    assert_eq!(s.operation_hash, OP);
    assert_eq!(s.eligibility_reference, ELIG);
    assert_eq!(s.capability_nonce, NONCE);
    assert_eq!(s.consumption_event_id, cev());
    assert_eq!(s.consumer, CONSUMER);
    assert_eq!(s.scope, SCOPE);
    assert_eq!(s.target, TARGET);
    assert_eq!(s.effect_class, EFFECT);
    // The execution-clock facts that govern admission are pinned, not discarded.
    assert!(s.revalidation_live);
    assert_eq!(s.revalidated_at, 10);
    assert_eq!(s.revalidation_valid_until, 1000);
    // The candidate basis is bound (not the empty/default hash).
    assert_eq!(s.admission_candidate_hash, candidate_basis_hash(&candidate()));
    assert!(s.snapshot_hash().starts_with("sha256:"));
}

#[test]
fn clean_snapshot_verifies_against_its_consume() {
    let s = snapshot();
    let h = s.snapshot_hash();
    assert_eq!(s.verify_consume_binding(&h, ELIG, NONCE, &cev()), Ok(()));
}

// --- replay reconstructs the identical hash --------------------------------- //

#[test]
fn snapshot_hash_is_deterministic_across_rebuild_and_roundtrip() {
    let a = snapshot().snapshot_hash();
    let b = snapshot().snapshot_hash();
    assert_eq!(a, b, "rebuilding the same snapshot yields the same hash");

    // A durable record round-trips through JSON and still reconstructs the identical hash (the replay
    // path: read the snapshot back from the store, recompute).
    let json = serde_json::to_string(&snapshot()).unwrap();
    let restored: ComposedExecutionSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.snapshot_hash(), a, "replay reconstructs the identical snapshot hash");
}

// --- the lossy-receipt theorem, made concrete ------------------------------- //

#[test]
fn pinning_snapshot_distinguishes_execution_snapshots() {
    // Same lineage (same candidate basis, eligibility, nonce, operation, consumption event) but a
    // DIFFERENT execution snapshot: revalidated at a different clock against a different horizon. A lossy
    // receipt (eligibility-only) could not tell these apart; the pinning snapshot must.
    let early = ComposedExecutionSnapshot::from_authorized(&authorized_at(10, 1000), cev());
    let later = ComposedExecutionSnapshot::from_authorized(&authorized_at(20, 2000), cev());

    assert_eq!(early.eligibility_reference, later.eligibility_reference);
    assert_eq!(early.capability_nonce, later.capability_nonce);
    assert_eq!(early.admission_candidate_hash, later.admission_candidate_hash);
    assert_eq!(early.consumption_event_id, later.consumption_event_id);

    assert_ne!(
        early.snapshot_hash(),
        later.snapshot_hash(),
        "two execution snapshots over one lineage must not collapse to the same receipt hash",
    );
}

#[test]
fn each_governing_fact_changes_the_hash() {
    let base = snapshot();
    let h = base.snapshot_hash();

    let mut elig = base.clone();
    elig.eligibility_reference = "sha256:other-standing".into();
    assert_ne!(elig.snapshot_hash(), h);

    let mut nonce = base.clone();
    nonce.capability_nonce = "nonce-9".into();
    assert_ne!(nonce.snapshot_hash(), h);

    let mut when = base.clone();
    when.revalidated_at = 11;
    assert_ne!(when.snapshot_hash(), h);

    let mut horizon = base.clone();
    horizon.revalidation_valid_until = 999;
    assert_ne!(horizon.snapshot_hash(), h);

    let mut basis = base.clone();
    basis.admission_candidate_hash = "sha256:different-basis".into();
    assert_ne!(basis.snapshot_hash(), h);
}

// --- a receipt that describes a different admission state is rejected -------- //

#[test]
fn tampered_hash_is_rejected() {
    let s = snapshot();
    assert_eq!(
        s.verify_consume_binding("sha256:not-the-real-hash", ELIG, NONCE, &cev()),
        Err(SnapshotIncoherence::HashMismatch),
    );
}

#[test]
fn consume_referencing_a_different_eligibility_is_rejected() {
    let s = snapshot();
    let h = s.snapshot_hash();
    assert_eq!(
        s.verify_consume_binding(&h, "sha256:WRONG", NONCE, &cev()),
        Err(SnapshotIncoherence::EligibilityReferenceMismatch),
    );
}

#[test]
fn consume_referencing_a_different_nonce_is_rejected() {
    let s = snapshot();
    let h = s.snapshot_hash();
    assert_eq!(
        s.verify_consume_binding(&h, ELIG, "nonce-other", &cev()),
        Err(SnapshotIncoherence::CapabilityNonceMismatch),
    );
}

#[test]
fn effect_outcome_referencing_a_different_consume_is_rejected() {
    let s = snapshot();
    let h = s.snapshot_hash();
    assert_eq!(
        s.verify_consume_binding(&h, ELIG, NONCE, "op-other:nonce-7"),
        Err(SnapshotIncoherence::ConsumptionEventMismatch),
    );
}

#[test]
fn a_live_snapshot_without_an_execution_clock_is_rejected() {
    // The lossy case directly: claims liveness but pins no clock. from_authorized never builds this; a
    // hand-rolled durable record could, and the verifier must refuse it before trusting the hash.
    let mut lossy = snapshot();
    lossy.revalidated_at = 0;
    let h = lossy.snapshot_hash();
    assert_eq!(
        lossy.verify_consume_binding(&h, ELIG, NONCE, &cev()),
        Err(SnapshotIncoherence::MissingRevalidationClock),
    );
}

// --- the admission basis hash is sensitive to what was relied upon ---------- //

#[test]
fn candidate_basis_hash_tracks_the_relied_upon_references() {
    let base = candidate_basis_hash(&candidate());
    assert_ne!(base, candidate_basis_hash(&candidate_with("sha256:other", SCOPE, TARGET)));
    assert_ne!(base, candidate_basis_hash(&candidate_with(ELIG, "other-scope", TARGET)));
    assert_ne!(base, candidate_basis_hash(&candidate_with(ELIG, SCOPE, "other-target")));
    // Stable for identical basis.
    assert_eq!(base, candidate_basis_hash(&candidate()));
}
