//! Stage 3a: the consequence chain, terminating at a fake receiver. No real effect.
//!
//! candidate (decide) + LA capability + execution revalidation + operation context
//!   → AuthorizedTransition (final binding, anti-recombination enforced)
//!   → receiver gate (exact correspondence + single-use burn)
//!   → fake actuator (records a HandlingReceipt; performs nothing)
//!
//! Every hostile case the user named must refuse BEFORE the (fake) act.

use transition_kernel::authority::{
    AuthorizedTransition, BindError, ExecutionRevalidation, LaCapability, OperationContext, RevalError,
};
use transition_kernel::receiver_gate::{FakeActuator, ReceiverGate, RequestedOperation};
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
const EFFECT: &str = "fs_write";

/// An admitted candidate whose standing reference is the exact eligibility the capability will bind.
fn candidate() -> AdmissionCandidate {
    let office = OfficeOutputs {
        standing: StandingOutput::Verified { receipt_ref: ELIG.into() },
        spendability: SpendabilityOutput::NotApplicable,
        la_admission: AdmissionOutput::Admitted,
        la_consume: ConsumeOutput::Consumed,
        scope: SCOPE.into(),
        target: TARGET.into(),
    };
    match decide(&office) {
        TransitionDecision::Admit(c) => c,
        other => panic!("expected admit, got {other:?}"),
    }
}

fn capability() -> LaCapability {
    LaCapability {
        capability_id: NONCE.into(),
        scope: SCOPE.into(),
        target: TARGET.into(),
        effect_class: EFFECT.into(),
        eligibility_reference: ELIG.into(),
        expires_at: 100,
        single_use: true,
    }
}

fn reval(now: u64) -> ExecutionRevalidation {
    ExecutionRevalidation::revalidate(ELIG, true, 100, now).expect("standing live and fresh")
}

fn op_ctx() -> OperationContext {
    OperationContext { operation_hash: OP.into(), consumer: CONSUMER.into() }
}

fn authorized() -> AuthorizedTransition {
    AuthorizedTransition::finalize(candidate(), capability(), reval(10), op_ctx())
        .expect("clean lineage should finalize")
}

fn requested() -> RequestedOperation {
    RequestedOperation {
        operation_hash: OP.into(),
        consumer: CONSUMER.into(),
        scope: SCOPE.into(),
        target: TARGET.into(),
        effect_class: EFFECT.into(),
        capability_nonce: NONCE.into(),
        now: 10,
    }
}

// --- the happy path -------------------------------------------------------- //

#[test]
fn clean_chain_accepts_acts_and_burns() {
    let at = authorized();
    let mut gate = ReceiverGate::new();
    let mut actuator = FakeActuator::new();

    let receipt = gate.handle(&at, &requested());
    assert!(receipt.acted, "clean correspondence should accept");
    actuator.act(receipt);
    assert!(gate.is_burned(NONCE), "the single-use capability is burned");
    assert_eq!(actuator.receipts.len(), 1);
    assert!(actuator.receipts[0].acted);
}

// --- recombination / lineage refused at the BINDING, before a receiver exists --- //

#[test]
fn capability_for_a_different_standing_cannot_finalize() {
    // A capability minted against a different eligibility reference: the anti-recombination check bars
    // it. There is no AuthorizedTransition, hence no receiver, hence no effect.
    let mut cap = capability();
    cap.eligibility_reference = "sha256:some-other-standing".into();
    let err = AuthorizedTransition::finalize(candidate(), cap, reval(10), op_ctx()).unwrap_err();
    assert_eq!(err, BindError::EligibilityMismatch);
}

#[test]
fn capability_for_a_different_target_cannot_finalize() {
    let mut cap = capability();
    cap.target = "other".into();
    let err = AuthorizedTransition::finalize(candidate(), cap, reval(10), op_ctx()).unwrap_err();
    assert_eq!(err, BindError::TargetMismatch);
}

#[test]
fn non_single_use_capability_cannot_finalize() {
    let mut cap = capability();
    cap.single_use = false;
    let err = AuthorizedTransition::finalize(candidate(), cap, reval(10), op_ctx()).unwrap_err();
    assert_eq!(err, BindError::CapabilityNotSingleUse);
}

// --- stale-at-execution: the revalidation itself refuses --------------------- //

#[test]
fn stale_at_execution_blocks_revalidation() {
    // Admitted earlier, but at execution time the standing horizon (valid_until=100) has passed.
    assert_eq!(
        ExecutionRevalidation::revalidate(ELIG, true, 100, 101).unwrap_err(),
        RevalError::StaleAtExecution,
    );
}

#[test]
fn revoked_standing_blocks_revalidation() {
    assert_eq!(
        ExecutionRevalidation::revalidate(ELIG, false, 100, 10).unwrap_err(),
        RevalError::StandingNotLive,
    );
}

// --- receiver-gate correspondence refusals (before the fake act) ------------- //

#[test]
fn capability_replay_is_refused_after_burn() {
    let at = authorized();
    let mut gate = ReceiverGate::new();
    assert!(gate.handle(&at, &requested()).acted);
    let second = gate.handle(&at, &requested());
    assert!(!second.acted);
    assert_eq!(second.refusal.as_deref(), Some("already_burned"));
}

#[test]
fn wrong_target_is_refused() {
    let at = authorized();
    let mut gate = ReceiverGate::new();
    let mut req = requested();
    req.target = "other".into();
    let r = gate.handle(&at, &req);
    assert!(!r.acted);
    assert_eq!(r.refusal.as_deref(), Some("target_mismatch"));
    assert!(!gate.is_burned(NONCE), "a refused op burns nothing");
}

#[test]
fn wrong_effect_class_is_refused() {
    let at = authorized();
    let mut gate = ReceiverGate::new();
    let mut req = requested();
    req.effect_class = "net_write".into();
    assert_eq!(gate.handle(&at, &req).refusal.as_deref(), Some("effect_class_mismatch"));
}

#[test]
fn expired_capability_is_refused() {
    let at = authorized();
    let mut gate = ReceiverGate::new();
    let mut req = requested();
    req.now = 101; // past expires_at=100
    assert_eq!(gate.handle(&at, &req).refusal.as_deref(), Some("expired"));
}

#[test]
fn check_correspondence_does_not_burn() {
    // The live-path check (3b1): correspondence holds but the receiver does NOT own "spent" — repeated
    // checks both pass, because LA owns the authoritative burn.
    let at = authorized();
    let gate = ReceiverGate::new();
    assert!(gate.check_correspondence(&at, &requested()).is_none());
    assert!(gate.check_correspondence(&at, &requested()).is_none());
    assert!(!gate.is_burned(NONCE), "correspondence check must not burn");

    let mut wrong = requested();
    wrong.effect_class = "net".into();
    assert_eq!(
        gate.check_correspondence(&at, &wrong),
        Some(transition_kernel::receiver_gate::ReceiverRefusal::EffectClassMismatch),
    );
}

#[test]
fn wrong_consumer_and_operation_are_refused() {
    let at = authorized();
    let mut gate = ReceiverGate::new();

    let mut wrong_consumer = requested();
    wrong_consumer.consumer = "ag:other".into();
    assert_eq!(gate.handle(&at, &wrong_consumer).refusal.as_deref(), Some("consumer_mismatch"));

    let mut wrong_op = requested();
    wrong_op.operation_hash = "op-hash-different".into();
    assert_eq!(gate.handle(&at, &wrong_op).refusal.as_deref(), Some("operation_mismatch"));

    let mut wrong_nonce = requested();
    wrong_nonce.capability_nonce = "nonce-9".into();
    assert_eq!(gate.handle(&at, &wrong_nonce).refusal.as_deref(), Some("nonce_mismatch"));
}
