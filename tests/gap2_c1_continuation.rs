//! GAP-2 C1: the pure continuation office (workbench).
//!
//! Stage 3 governed consequence (may *this effect* happen?). Continuation governs whether the agent earns
//! *one more step*. The hinge: **the previous effect being legitimate does not authorize the next step.**
//! A clean `Consumed + EffectSucceeded` is necessary but never sufficient — the next step must be bound to
//! the prior chain tip, the same session/actor/scope, and a known class, under current versions.
//!
//! No live supervisor, no enforce, no real effect: these exercise the office and its single-use consumer
//! directly.

use transition_kernel::composed_snapshot::ComposedExecutionSnapshot;
use transition_kernel::continuation::{
    decide_continuation, ContinuationConsumer, ContinuationDecision, ContinuationPolicy,
    ContinuationRefusal, ContinuationRefuseKind, ContinuationRequest, NextStepPresentation,
    TerminalOutcome, CURRENT_KERNEL_VERSION,
};

const SESSION: &str = "sess-1";
const ACTOR: &str = "ag:main";
const SCOPE: &str = "lab";
const NEXT_CLASS: &str = "create_marker_v1";
const POLICY_VERSION: &str = "policy.v1";

/// A coherent prior composed snapshot (the Stage 3c receipt) governed at scope `lab`.
fn prior_snapshot() -> ComposedExecutionSnapshot {
    ComposedExecutionSnapshot {
        schema: "transition_kernel.composed_snapshot.v1".into(),
        kernel_version: CURRENT_KERNEL_VERSION.into(),
        operation_hash: "op-1".into(),
        admission_candidate_hash: "sha256:basis".into(),
        eligibility_reference: "sha256:standing-xyz".into(),
        revalidation_live: true,
        revalidation_valid_until: 1000,
        revalidated_at: 10,
        capability_nonce: "nonce-7".into(),
        consumption_event_id: "op-1:nonce-7".into(),
        consumer: ACTOR.into(),
        scope: SCOPE.into(),
        target: "write".into(),
        effect_class: NEXT_CLASS.into(),
    }
}

fn policy() -> ContinuationPolicy {
    ContinuationPolicy {
        current_policy_version: POLICY_VERSION.into(),
        admissible_next_step_classes: vec![NEXT_CLASS.into(), "read_v1".into()],
        grant_ttl: 100,
    }
}

/// A clean request: succeeded prior step, matching chain tip, current versions, in-scope, known class.
fn request() -> ContinuationRequest {
    let snap = prior_snapshot();
    let tip = snap.snapshot_hash();
    ContinuationRequest {
        prior_snapshot: Some(snap),
        terminal_outcome: TerminalOutcome::ConsumedEffectSucceeded,
        prior_chain_tip: tip,
        session_id: SESSION.into(),
        actor_id: ACTOR.into(),
        requested_next_step: NEXT_CLASS.into(),
        scope: SCOPE.into(),
        remaining_capacity_ref: "token:t0".into(),
        policy_version: POLICY_VERSION.into(),
        kernel_version: CURRENT_KERNEL_VERSION.into(),
        now: 20,
    }
}

fn grant_of(req: &ContinuationRequest) -> transition_kernel::continuation::ContinuationGrant {
    match decide_continuation(req, &policy()) {
        ContinuationDecision::Grant(g) => g,
        other => panic!("expected grant, got {other:?}"),
    }
}

fn presentation() -> NextStepPresentation {
    NextStepPresentation {
        session_id: SESSION.into(),
        actor_id: ACTOR.into(),
        chain_tip: prior_snapshot().snapshot_hash(),
        scope: SCOPE.into(),
        next_step_class: NEXT_CLASS.into(),
        now: 20,
    }
}

fn assert_refuse(d: ContinuationDecision, kind: ContinuationRefuseKind) {
    match d {
        ContinuationDecision::Refuse { kind: k, .. } => assert_eq!(k, kind),
        other => panic!("expected Refuse({kind:?}), got {other:?}"),
    }
}

// --- the happy path: a bound, single-use grant ------------------------------ //

#[test]
fn clean_continuation_after_consumed_effect_succeeded_grants() {
    let grant = grant_of(&request());
    assert_eq!(grant.session_id(), SESSION);
    assert_eq!(grant.actor_id(), ACTOR);
    assert_eq!(grant.chain_tip(), prior_snapshot().snapshot_hash());
    assert_eq!(grant.scope(), SCOPE);
    assert_eq!(grant.next_step_class(), NEXT_CLASS);
    assert_eq!(grant.expires_at(), 120); // now=20 + ttl=100

    // The grant admits exactly one matching next step.
    let mut consumer = ContinuationConsumer::new();
    let receipt = consumer.present(&grant, &presentation());
    assert!(receipt.admitted);
    assert!(consumer.is_burned(grant.grant_id()));
}

// --- the semantic line: a legitimate prior step is necessary, not sufficient - //

#[test]
fn refuse_after_consumed_outcome_unknown() {
    let mut req = request();
    req.terminal_outcome = TerminalOutcome::ConsumedOutcomeUnknown;
    assert_refuse(decide_continuation(&req, &policy()), ContinuationRefuseKind::PriorStepNotSucceeded);
}

#[test]
fn refuse_after_not_consumed_refused() {
    let mut req = request();
    req.terminal_outcome = TerminalOutcome::NotConsumedRefused;
    assert_refuse(decide_continuation(&req, &policy()), ContinuationRefuseKind::PriorStepNotSucceeded);
}

#[test]
fn refuse_after_consumed_effect_failed_or_conflict() {
    for outcome in [TerminalOutcome::ConsumedEffectFailed, TerminalOutcome::ConsumedEffectConflict,
                    TerminalOutcome::ConsumedEffectNotAttempted] {
        let mut req = request();
        req.terminal_outcome = outcome;
        assert_refuse(decide_continuation(&req, &policy()),
                      ContinuationRefuseKind::PriorStepNotSucceeded);
    }
}

// --- evidence + coherence at issue ------------------------------------------ //

#[test]
fn terminal_receipt_missing_snapshot_refuses() {
    let mut req = request();
    req.prior_snapshot = None;
    assert_refuse(decide_continuation(&req, &policy()),
                  ContinuationRefuseKind::TerminalReceiptMissingSnapshot);
}

#[test]
fn lossy_snapshot_without_execution_clock_refuses() {
    let mut req = request();
    if let Some(s) = req.prior_snapshot.as_mut() {
        s.revalidated_at = 0; // claims liveness but pins no clock
    }
    // The chain tip no longer matches either, but the lossy guard fires first.
    assert_refuse(decide_continuation(&req, &policy()),
                  ContinuationRefuseKind::TerminalReceiptMissingSnapshot);
}

#[test]
fn wrong_chain_tip_refuses_at_issue() {
    let mut req = request();
    req.prior_chain_tip = "sha256:some-other-tip".into();
    assert_refuse(decide_continuation(&req, &policy()), ContinuationRefuseKind::ChainTipMismatch);
}

// --- freshness -------------------------------------------------------------- //

#[test]
fn stale_policy_version_refuses() {
    let mut req = request();
    req.policy_version = "policy.v0".into();
    assert_refuse(decide_continuation(&req, &policy()), ContinuationRefuseKind::StalePolicyVersion);
}

#[test]
fn stale_kernel_version_refuses() {
    let mut req = request();
    req.kernel_version = "0.0.0-ancient".into();
    assert_refuse(decide_continuation(&req, &policy()), ContinuationRefuseKind::StaleKernelVersion);
}

// --- scope: no widening ----------------------------------------------------- //

#[test]
fn requested_scope_expansion_refuses() {
    let mut req = request();
    req.scope = "lab+prod".into(); // broader than the prior governed scope
    assert_refuse(decide_continuation(&req, &policy()), ContinuationRefuseKind::ScopeExpansion);
}

// --- unknown next-step class escalates -------------------------------------- //

#[test]
fn unknown_next_step_class_escalates() {
    let mut req = request();
    req.requested_next_step = "delete_everything_v1".into();
    match decide_continuation(&req, &policy()) {
        ContinuationDecision::Escalate(e) => {
            assert!(e.required_authority.contains("delete_everything_v1"));
        }
        other => panic!("expected Escalate, got {other:?}"),
    }
}

// --- the consumer: non-transferable, single-use, expiring ------------------- //

#[test]
fn wrong_session_at_present_refuses() {
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    let mut p = presentation();
    p.session_id = "sess-other".into();
    let r = consumer.present(&grant, &p);
    assert!(!r.admitted);
    assert_eq!(r.refusal, Some(ContinuationRefusal::SessionMismatch));
    assert!(!consumer.is_burned(grant.grant_id()), "a refused presentation burns nothing");
}

#[test]
fn wrong_actor_at_present_refuses_non_transferable() {
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    let mut p = presentation();
    p.actor_id = "ag:other".into();
    assert_eq!(consumer.present(&grant, &p).refusal, Some(ContinuationRefusal::ActorMismatch));
}

#[test]
fn wrong_chain_tip_at_present_refuses() {
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    let mut p = presentation();
    p.chain_tip = "sha256:other-tip".into();
    assert_eq!(consumer.present(&grant, &p).refusal, Some(ContinuationRefusal::ChainTipMismatch));
}

#[test]
fn scope_mismatch_at_present_refuses() {
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    let mut p = presentation();
    p.scope = "prod".into();
    assert_eq!(consumer.present(&grant, &p).refusal, Some(ContinuationRefusal::ScopeMismatch));
}

#[test]
fn wrong_next_step_class_at_present_refuses() {
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    let mut p = presentation();
    p.next_step_class = "read_v1".into();
    assert_eq!(consumer.present(&grant, &p).refusal, Some(ContinuationRefusal::NextStepClassMismatch));
}

#[test]
fn expired_grant_at_present_refuses() {
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    let mut p = presentation();
    p.now = 121; // past expires_at = 120
    assert_eq!(consumer.present(&grant, &p).refusal, Some(ContinuationRefusal::Expired));
}

#[test]
fn reused_grant_refuses_already_consumed() {
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    assert!(consumer.present(&grant, &presentation()).admitted);
    let second = consumer.present(&grant, &presentation());
    assert!(!second.admitted);
    assert_eq!(second.refusal, Some(ContinuationRefusal::AlreadyConsumed));
}

#[test]
fn grant_replay_across_parallel_next_steps_admits_once() {
    // Two next steps race to present the SAME grant against the same single-use ledger. Exactly one wins.
    let grant = grant_of(&request());
    let mut consumer = ContinuationConsumer::new();
    let outcomes: Vec<bool> =
        (0..2).map(|_| consumer.present(&grant, &presentation()).admitted).collect();
    assert_eq!(outcomes.iter().filter(|&&a| a).count(), 1, "a grant authorizes exactly one step");
}

// --- a fresh continuation is a DIFFERENT grant identity --------------------- //

#[test]
fn distinct_continuations_have_distinct_grant_ids() {
    let g1 = grant_of(&request());
    // A different next-step class (still admissible) is a different continuation → different identity.
    let mut req2 = request();
    req2.requested_next_step = "read_v1".into();
    let g2 = grant_of(&req2);
    assert_ne!(g1.grant_id(), g2.grant_id());

    // And the burn of one does not burn the other.
    let mut consumer = ContinuationConsumer::new();
    assert!(consumer.present(&g1, &presentation()).admitted);
    assert!(!consumer.is_burned(g2.grant_id()));
}
