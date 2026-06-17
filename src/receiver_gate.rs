//! The receiver gate (the loading-dock door) and a deliberately pathetic fake actuator.
//!
//! The actuator does NOT decide authorization. The receiver gate validates the **exact** correspondence
//! between an [`AuthorizedTransition`] and the requested operation — operation hash, consumer, scope,
//! target, effect class, capability nonce, expiry, single-use state, and the candidate/capability/
//! standing lineage — and burns the single-use capability. Only on accept does the fake actuator emit a
//! `HandlingReceipt` with `acted: true`. In Stage 3a there is no real effect; this proves accept/refuse
//! and burn behavior before any furniture moves.
//!
//! WLP doctrine realized: "no receiver mutates state on an action-bearing claim unless the claim is
//! attributable and carries the witness required by the receiver's admission policy"; a valid capability
//! for effect A cannot be replayed for effect B (effect-class × target × scope × nonce co-validation).

use std::collections::HashSet;

use crate::authority::AuthorizedTransition;

/// The exact operation a caller asks the receiver to admit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RequestedOperation {
    pub operation_hash: String,
    pub consumer: String,
    pub scope: String,
    pub target: String,
    pub effect_class: String,
    pub capability_nonce: String,
    pub now: u64,
}

/// Why the receiver refused. Each corresponds to a correspondence the requested operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverRefusal {
    OperationMismatch,
    ConsumerMismatch,
    ScopeMismatch,
    TargetMismatch,
    EffectClassMismatch,
    NonceMismatch,
    Expired,
    AlreadyBurned,
}

impl ReceiverRefusal {
    pub fn as_str(self) -> &'static str {
        match self {
            ReceiverRefusal::OperationMismatch => "operation_mismatch",
            ReceiverRefusal::ConsumerMismatch => "consumer_mismatch",
            ReceiverRefusal::ScopeMismatch => "scope_mismatch",
            ReceiverRefusal::TargetMismatch => "target_mismatch",
            ReceiverRefusal::EffectClassMismatch => "effect_class_mismatch",
            ReceiverRefusal::NonceMismatch => "nonce_mismatch",
            ReceiverRefusal::Expired => "expired",
            ReceiverRefusal::AlreadyBurned => "already_burned",
        }
    }
}

/// The terminal record of a handling attempt — the analog of WLP's `HandlingReceipt`. `acted` is true
/// only on accept (and, in Stage 3a, the act is a fake/no-op).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HandlingReceipt {
    pub acted: bool,
    pub capability_nonce: String,
    pub operation_hash: String,
    pub refusal: Option<String>,
}

/// The receiver-side enforcement boundary. Owns the single-use burn set; the consequence-bearer is
/// structurally behind it.
#[derive(Debug, Default)]
pub struct ReceiverGate {
    burned: HashSet<String>,
}

impl ReceiverGate {
    pub fn new() -> Self {
        Self { burned: HashSet::new() }
    }

    /// Validate exact correspondence and, on accept, burn the capability. Returns the handling receipt.
    pub fn handle(
        &mut self,
        authorized: &AuthorizedTransition,
        requested: &RequestedOperation,
    ) -> HandlingReceipt {
        let cap = authorized.capability();
        let op = authorized.operation();

        let refusal = self.check(authorized, requested, cap, op);
        match refusal {
            Some(r) => HandlingReceipt {
                acted: false,
                capability_nonce: requested.capability_nonce.clone(),
                operation_hash: requested.operation_hash.clone(),
                refusal: Some(r.as_str().to_string()),
            },
            None => {
                // Accept: burn the single-use capability, then (fake) act.
                self.burned.insert(cap.capability_id.clone());
                HandlingReceipt {
                    acted: true,
                    capability_nonce: cap.capability_id.clone(),
                    operation_hash: op.operation_hash.clone(),
                    refusal: None,
                }
            }
        }
    }

    /// Exact-correspondence check **without** consulting the burn set. This is the live-path check:
    /// the receiver validates the requested effect matches the authorization, but it does NOT own the
    /// authoritative notion of "spent" — that is LinearAccountant's consume ledger (3b1). Use this when
    /// LA performs the burn, so the receiver never creates a second competing notion of spent.
    pub fn check_correspondence(
        &self,
        authorized: &AuthorizedTransition,
        req: &RequestedOperation,
    ) -> Option<ReceiverRefusal> {
        let cap = authorized.capability();
        let op = authorized.operation();
        // Operation + consumer: the requested effect must be the exact one authorized, presented by the
        // authorized consumer.
        if req.operation_hash != op.operation_hash {
            return Some(ReceiverRefusal::OperationMismatch);
        }
        if req.consumer != op.consumer {
            return Some(ReceiverRefusal::ConsumerMismatch);
        }
        // Capability correspondence: a valid key for effect A cannot be replayed for effect B.
        if req.scope != cap.scope {
            return Some(ReceiverRefusal::ScopeMismatch);
        }
        if req.target != cap.target {
            return Some(ReceiverRefusal::TargetMismatch);
        }
        if req.effect_class != cap.effect_class {
            return Some(ReceiverRefusal::EffectClassMismatch);
        }
        if req.capability_nonce != cap.capability_id {
            return Some(ReceiverRefusal::NonceMismatch);
        }
        if req.now > cap.expires_at {
            return Some(ReceiverRefusal::Expired);
        }
        None
    }

    fn check(
        &self,
        authorized: &AuthorizedTransition,
        req: &RequestedOperation,
        cap: &crate::authority::LaCapability,
        _op: &crate::authority::OperationContext,
    ) -> Option<ReceiverRefusal> {
        if let Some(r) = self.check_correspondence(authorized, req) {
            return Some(r);
        }
        // 3a-only local single-use detection. In the live 3b1 path LA owns this via `already_consumed`;
        // here it is a fast advisory replay detector for the fake-actuator path.
        if self.burned.contains(&cap.capability_id) {
            return Some(ReceiverRefusal::AlreadyBurned);
        }
        None
    }

    pub fn is_burned(&self, capability_id: &str) -> bool {
        self.burned.contains(capability_id)
    }
}

/// The deliberately pathetic actuator: it records handling receipts and performs no real effect. Its
/// only job is to prove the receiver gate's accept/refuse/burn behavior end to end.
#[derive(Debug, Default)]
pub struct FakeActuator {
    pub receipts: Vec<HandlingReceipt>,
}

impl FakeActuator {
    pub fn new() -> Self {
        Self { receipts: Vec::new() }
    }

    /// "Perform" the effect: in Stage 3a this only records the receipt. No filesystem, no tool, nothing.
    pub fn act(&mut self, receipt: HandlingReceipt) -> &HandlingReceipt {
        self.receipts.push(receipt);
        self.receipts.last().unwrap()
    }
}
