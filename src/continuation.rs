//! GAP-2 C1: the pure continuation office (workbench).
//!
//! Stage 3 governed **consequence** — may *this effect* happen? The continuation office governs
//! **continuation** — given the prior governed step and its terminal receipt chain, may this agent take
//! *one more step* as an active worker? This is the AG-on-AG hinge, and its load-bearing line is:
//!
//! > The previous effect being legitimate does **not** authorize the next step.
//!
//! A clean `Consumed + EffectSucceeded` terminal is *necessary* but never *sufficient*: the office still
//! binds the next step to the exact prior chain tip, the same session and actor, the same scope (no
//! widening), and a known next-step class, under current policy/kernel versions. A legacy loop would let
//! an agent narrate its way into another action; this office refuses unless the continuation is bound,
//! fresh, and within the slice.
//!
//! Output is [`ContinuationDecision`]: `Grant(ContinuationGrant) | Refuse | Escalate`. The
//! [`ContinuationGrant`] is **sealed** — there is no public constructor, no `Default`, no `From` — so it
//! is **not constructible from narrative state**: the only path to one is [`decide_continuation`] over a
//! verified terminal chain (which must carry a Stage 3c [`ComposedExecutionSnapshot`]). The grant is
//! single-use, session/actor/chain-tip/scope/next-step-class bound, expiring, and non-transferable; its
//! single-use burn is owned by [`ContinuationConsumer`] (the analog of the receiver gate).
//!
//! **Workbench only (C1).** No live supervisor wiring, no `enforce`, no real effect. C2 (`observe/hold`)
//! and C3 (`enforce`) come later, and only after reuse / transfer / scope-expansion are unconstructable
//! or refused here first.

use std::collections::HashSet;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::composed_snapshot::ComposedExecutionSnapshot;

/// The kernel version this office runs as; continuation refuses a prior step stamped with a different
/// kernel version (the world moved — re-admit fresh, never inherit across a kernel change).
pub const CURRENT_KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The terminal outcome of the prior governed step (the Stage 3 terminal vocabulary; wire-compatible with
/// `agent_gov`'s `transition_enforce` terminal tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalOutcome {
    ConsumedEffectSucceeded,
    ConsumedEffectFailed,
    ConsumedEffectConflict,
    ConsumedOutcomeUnknown,
    ConsumedEffectNotAttempted,
    NotConsumedRefused,
}

impl TerminalOutcome {
    /// Only a settled, verified success may be continued from. Every other terminal — failure, conflict,
    /// unknown, not-attempted, or refusal — is *not* a basis for another step. (`OutcomeUnknown` in
    /// particular must not continue: we do not know the world's state.)
    pub fn is_settled_success(self) -> bool {
        matches!(self, TerminalOutcome::ConsumedEffectSucceeded)
    }
}

/// The continuation request: the prior governed step's terminal chain plus the proposed next step.
#[derive(Debug, Clone, Deserialize)]
pub struct ContinuationRequest {
    /// The Stage 3c snapshot that governed the prior effect. `None` models a terminal receipt that does
    /// not carry a composed snapshot — which cannot ground a continuation.
    pub prior_snapshot: Option<ComposedExecutionSnapshot>,
    pub terminal_outcome: TerminalOutcome,
    /// The chain tip the next step claims to continue from — must equal the prior snapshot's hash.
    pub prior_chain_tip: String,
    pub session_id: String,
    pub actor_id: String,
    /// The class of the next step the agent proposes (admissibility is policy-decided).
    pub requested_next_step: String,
    /// The scope the next step requests — must equal the prior governed scope (C1: no widening).
    pub scope: String,
    /// Opaque LinearAccountant capacity reference; bound into the grant for a later (C3) capacity recheck.
    /// C1 makes no capacity decision — LA owns "spent".
    pub remaining_capacity_ref: String,
    /// The policy/kernel versions the prior step ran under (checked against current — stale refuses).
    pub policy_version: String,
    pub kernel_version: String,
    pub now: u64,
}

/// The current admission policy for continuation: which next-step classes are admissible, the current
/// policy version, and the grant lifetime. Passed in (the office is pure — no globals).
#[derive(Debug, Clone, Deserialize)]
pub struct ContinuationPolicy {
    pub current_policy_version: String,
    pub admissible_next_step_classes: Vec<String>,
    /// Grant lifetime added to `now` to set `expires_at`.
    pub grant_ttl: u64,
}

/// Why continuation was refused. Each is a way the proposed next step failed to cohere with the prior
/// governed step — never a way the prior step itself was illegitimate (that was Stage 3's job).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationRefuseKind {
    /// The terminal receipt carries no composed snapshot (or a lossy one) — nothing to continue *from*.
    TerminalReceiptMissingSnapshot,
    /// The claimed chain tip does not match the prior snapshot's hash.
    ChainTipMismatch,
    StalePolicyVersion,
    StaleKernelVersion,
    /// The prior step did not end in a settled success — legitimacy of a *failed/unknown* step does not
    /// license another step.
    PriorStepNotSucceeded,
    /// The next step requests a scope other than the prior governed scope (C1 requires exact scope).
    ScopeExpansion,
}

impl ContinuationRefuseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ContinuationRefuseKind::TerminalReceiptMissingSnapshot => "terminal_receipt_missing_snapshot",
            ContinuationRefuseKind::ChainTipMismatch => "chain_tip_mismatch",
            ContinuationRefuseKind::StalePolicyVersion => "stale_policy_version",
            ContinuationRefuseKind::StaleKernelVersion => "stale_kernel_version",
            ContinuationRefuseKind::PriorStepNotSucceeded => "prior_step_not_succeeded",
            ContinuationRefuseKind::ScopeExpansion => "scope_expansion",
        }
    }
}

/// An escalation carries the authority a continuation *would* require — a typed obligation, not a
/// default-open. An unknown next-step class escalates: the office will not silently admit nor silently
/// deny a class it has no policy for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationEscalation {
    pub required_authority: String,
    pub reasons: Vec<String>,
}

/// The single-use authority to take exactly one more governed step. **Sealed**: no public constructor, no
/// `Default`, no `From` — the only path to a `ContinuationGrant` is [`decide_continuation`]'s grant arm.
/// A narrated proposal cannot become a grant by name. Bound to the session, actor, chain tip, scope, and
/// next-step class it was issued for; expiring; non-transferable (the binds are re-checked at consume).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationGrant {
    /// Single-use identity, derived deterministically from the bound fields so a replayed continuation
    /// maps to the same grant in the consumer's burn ledger (cf. `consumption_event_id`).
    grant_id: String,
    session_id: String,
    actor_id: String,
    chain_tip: String,
    scope: String,
    next_step_class: String,
    capacity_ref: String,
    expires_at: u64,
    _seal: (),
}

impl ContinuationGrant {
    pub fn grant_id(&self) -> &str {
        &self.grant_id
    }
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }
    pub fn chain_tip(&self) -> &str {
        &self.chain_tip
    }
    pub fn scope(&self) -> &str {
        &self.scope
    }
    pub fn next_step_class(&self) -> &str {
        &self.next_step_class
    }
    pub fn capacity_ref(&self) -> &str {
        &self.capacity_ref
    }
    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }
}

/// The office's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuationDecision {
    Grant(ContinuationGrant),
    Refuse {
        kind: ContinuationRefuseKind,
        reasons: Vec<String>,
    },
    Escalate(ContinuationEscalation),
}

/// Decide whether the proposed next step may continue from the prior governed step. Pure: no globals, no
/// effect, no clock of its own beyond the `now` supplied in the request.
///
/// Order: evidence presence → chain-tip coherence → version freshness → prior-step success → scope →
/// next-step-class admissibility → grant. Earlier checks are more fundamental (you cannot evaluate a
/// continuation without the snapshot it continues from).
pub fn decide_continuation(
    req: &ContinuationRequest,
    policy: &ContinuationPolicy,
) -> ContinuationDecision {
    // 1. Evidence: the prior terminal chain must carry a (coherent) composed snapshot.
    let snapshot = match &req.prior_snapshot {
        None => {
            return refuse(
                ContinuationRefuseKind::TerminalReceiptMissingSnapshot,
                "terminal receipt carries no composed snapshot",
            );
        }
        // A snapshot that claims liveness but pins no execution clock is lossy — it cannot ground a
        // chain tip (the Stage 3c MissingRevalidationClock case, refused here too).
        Some(s) if s.revalidation_live && s.revalidated_at == 0 => {
            return refuse(
                ContinuationRefuseKind::TerminalReceiptMissingSnapshot,
                "composed snapshot present but lossy (no execution clock)",
            );
        }
        Some(s) => s,
    };

    // 2. Chain-tip binding: the claimed tip must be the prior snapshot's exact hash.
    if snapshot.snapshot_hash() != req.prior_chain_tip {
        return refuse(
            ContinuationRefuseKind::ChainTipMismatch,
            "prior_chain_tip does not match the prior snapshot hash",
        );
    }

    // 3. Freshness: the prior step's policy/kernel versions must be current (the world may have moved).
    if req.policy_version != policy.current_policy_version {
        return refuse(
            ContinuationRefuseKind::StalePolicyVersion,
            "prior step ran under a non-current policy version",
        );
    }
    if req.kernel_version != CURRENT_KERNEL_VERSION {
        return refuse(
            ContinuationRefuseKind::StaleKernelVersion,
            "prior step ran under a non-current kernel version",
        );
    }

    // 4. The prior step must have settled in success. Necessary, never sufficient.
    if !req.terminal_outcome.is_settled_success() {
        return refuse(
            ContinuationRefuseKind::PriorStepNotSucceeded,
            "prior step did not end in Consumed + EffectSucceeded",
        );
    }

    // 5. Scope: the next step must stay within the prior governed scope. C1 requires exact scope; any
    //    difference (notably widening) refuses. The snapshot's scope is the governed envelope.
    if req.scope != snapshot.scope {
        return refuse(
            ContinuationRefuseKind::ScopeExpansion,
            "next-step scope is not the prior governed scope (C1: no widening)",
        );
    }

    // 6. Next-step class: unknown classes escalate (never silently admit nor silently deny).
    if !policy
        .admissible_next_step_classes
        .iter()
        .any(|c| c == &req.requested_next_step)
    {
        return ContinuationDecision::Escalate(ContinuationEscalation {
            required_authority: format!("authority_for_next_step_class:{}", req.requested_next_step),
            reasons: vec!["requested next-step class is not in the admissible set".to_string()],
        });
    }

    // 7. Grant: bound, single-use, expiring. The grant_id is derived from the binds so a replay maps to
    //    the same single-use identity.
    let grant_id = derive_grant_id(req);
    ContinuationDecision::Grant(ContinuationGrant {
        grant_id,
        session_id: req.session_id.clone(),
        actor_id: req.actor_id.clone(),
        chain_tip: req.prior_chain_tip.clone(),
        scope: req.scope.clone(),
        next_step_class: req.requested_next_step.clone(),
        capacity_ref: req.remaining_capacity_ref.clone(),
        expires_at: req.now.saturating_add(policy.grant_ttl),
        _seal: (),
    })
}

fn refuse(kind: ContinuationRefuseKind, reason: &str) -> ContinuationDecision {
    ContinuationDecision::Refuse { kind, reasons: vec![reason.to_string()] }
}

/// Derive the single-use grant identity from the bound fields (canonical JCS + SHA-256). Deterministic so
/// the same continuation maps to the same identity in the burn ledger; bound so a grant for one
/// (session, actor, chain tip, class, scope) is not the same identity as any other.
fn derive_grant_id(req: &ContinuationRequest) -> String {
    let basis = serde_json::json!({
        "schema": "transition_kernel.continuation_grant.v1",
        "chain_tip": req.prior_chain_tip,
        "session_id": req.session_id,
        "actor_id": req.actor_id,
        "next_step_class": req.requested_next_step,
        "scope": req.scope,
    });
    let canonical = serde_jcs::to_vec(&basis).expect("grant basis is plain serializable data");
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    format!("cont:{:x}", hasher.finalize())
}

/// What an agent presents when it actually tries to take the granted next step. The consumer checks this
/// against the grant's binds; a transfer (different actor/session) or a different chain tip/scope/class
/// fails correspondence.
#[derive(Debug, Clone, Deserialize)]
pub struct NextStepPresentation {
    pub session_id: String,
    pub actor_id: String,
    pub chain_tip: String,
    pub scope: String,
    pub next_step_class: String,
    pub now: u64,
}

/// Why the consumer refused to admit a presented next step against a grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationRefusal {
    SessionMismatch,
    ActorMismatch,
    ChainTipMismatch,
    ScopeMismatch,
    NextStepClassMismatch,
    Expired,
    AlreadyConsumed,
}

impl ContinuationRefusal {
    pub fn as_str(self) -> &'static str {
        match self {
            ContinuationRefusal::SessionMismatch => "session_mismatch",
            ContinuationRefusal::ActorMismatch => "actor_mismatch",
            ContinuationRefusal::ChainTipMismatch => "chain_tip_mismatch",
            ContinuationRefusal::ScopeMismatch => "scope_mismatch",
            ContinuationRefusal::NextStepClassMismatch => "next_step_class_mismatch",
            ContinuationRefusal::Expired => "expired",
            ContinuationRefusal::AlreadyConsumed => "already_consumed",
        }
    }
}

/// The record of presenting a next step against a grant. `admitted` is true only on exact correspondence
/// of an unexpired, unburned grant (and, in C1, nothing actually executes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationReceipt {
    pub admitted: bool,
    pub grant_id: String,
    pub refusal: Option<ContinuationRefusal>,
}

/// The single-use enforcement boundary for continuation grants — the analog of the receiver gate. Owns
/// the burn ledger; a grant admits at most once even under parallel presentation.
#[derive(Debug, Default)]
pub struct ContinuationConsumer {
    burned: HashSet<String>,
}

impl ContinuationConsumer {
    pub fn new() -> Self {
        Self { burned: HashSet::new() }
    }

    /// Exact-correspondence check **without** burning. The presented next step must match every bind of
    /// the grant; an expired grant refuses. Does not consult the burn ledger.
    pub fn check_correspondence(
        &self,
        grant: &ContinuationGrant,
        p: &NextStepPresentation,
    ) -> Option<ContinuationRefusal> {
        if p.session_id != grant.session_id {
            return Some(ContinuationRefusal::SessionMismatch);
        }
        if p.actor_id != grant.actor_id {
            return Some(ContinuationRefusal::ActorMismatch);
        }
        if p.chain_tip != grant.chain_tip {
            return Some(ContinuationRefusal::ChainTipMismatch);
        }
        if p.scope != grant.scope {
            return Some(ContinuationRefusal::ScopeMismatch);
        }
        if p.next_step_class != grant.next_step_class {
            return Some(ContinuationRefusal::NextStepClassMismatch);
        }
        if p.now > grant.expires_at {
            return Some(ContinuationRefusal::Expired);
        }
        None
    }

    /// Present a next step against the grant and, on exact correspondence, **burn** the single-use grant.
    /// A second presentation of the same grant (e.g. a replay across parallel next steps) refuses
    /// `already_consumed` — the grant authorizes exactly one step.
    pub fn present(
        &mut self,
        grant: &ContinuationGrant,
        p: &NextStepPresentation,
    ) -> ContinuationReceipt {
        if let Some(r) = self.check_correspondence(grant, p) {
            return ContinuationReceipt {
                admitted: false,
                grant_id: grant.grant_id.clone(),
                refusal: Some(r),
            };
        }
        if self.burned.contains(&grant.grant_id) {
            return ContinuationReceipt {
                admitted: false,
                grant_id: grant.grant_id.clone(),
                refusal: Some(ContinuationRefusal::AlreadyConsumed),
            };
        }
        self.burned.insert(grant.grant_id.clone());
        ContinuationReceipt { admitted: true, grant_id: grant.grant_id.clone(), refusal: None }
    }

    pub fn is_burned(&self, grant_id: &str) -> bool {
        self.burned.contains(grant_id)
    }
}
