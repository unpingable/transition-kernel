//! Stage 3a: drive the whole consequence chain from one JSON bundle, terminating at the fake actuator.
//!
//! This is the boundary 3b will call once consequence turns on; in 3a the actuator is a no-op and every
//! refusal is proven before any (fake) act. Non-operational throughout.

use serde::Deserialize;

use crate::authority::{AuthorizedTransition, ExecutionRevalidation, LaCapability, OperationContext};
use crate::live::WireOfficeOutputs;
use crate::receiver_gate::{FakeActuator, ReceiverGate, RequestedOperation};
use crate::transition_core::{decide, TransitionDecision};

#[derive(Debug, Deserialize)]
pub struct ChainBundle {
    pub execute_chain: Chain,
}

#[derive(Debug, Deserialize)]
pub struct Chain {
    pub office_outputs: WireOfficeOutputs,
    pub capability: LaCapability,
    pub revalidation: WireRevalidation,
    pub operation: OperationContext,
    pub requested: RequestedOperation,
}

#[derive(Debug, Deserialize)]
pub struct WireRevalidation {
    pub standing_ref: String,
    pub live: bool,
    pub valid_until: u64,
    pub now: u64,
}

/// Run candidate → finalize → receiver gate → fake actuator. Each stage may refuse; the JSON names where.
pub fn execute_chain(bundle: ChainBundle) -> Result<serde_json::Value, String> {
    let c = bundle.execute_chain;

    // 1. The decision (candidate or refusal).
    let office = c.office_outputs.into_office()?;
    let candidate = match decide(&office) {
        TransitionDecision::Admit(cand) => cand,
        TransitionDecision::Refuse { kind, seam, .. } => {
            return Ok(serde_json::json!({
                "stage": "decision", "result": "refused",
                "refusal_kind": kind.as_str(), "refusing_seam": seam.as_str(),
                "operational": false,
            }));
        }
        TransitionDecision::Escalate(_) => {
            return Ok(serde_json::json!({
                "stage": "decision", "result": "escalate", "operational": false,
            }));
        }
    };

    // 2. Execution revalidation (stale/revoked-at-execution refuses here).
    let reval = match ExecutionRevalidation::revalidate(
        &c.revalidation.standing_ref, c.revalidation.live, c.revalidation.valid_until,
        c.revalidation.now,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Ok(serde_json::json!({
                "stage": "revalidation", "result": "refused", "reason": format!("{e:?}"),
                "operational": false,
            }));
        }
    };

    // 3. Final binding (anti-recombination refuses here).
    let authorized =
        match AuthorizedTransition::finalize(candidate, c.capability, reval, c.operation) {
            Ok(a) => a,
            Err(e) => {
                return Ok(serde_json::json!({
                    "stage": "binding", "result": "refused", "reason": format!("{e:?}"),
                    "operational": false,
                }));
            }
        };

    // 4. Receiver gate + fake actuator.
    let mut gate = ReceiverGate::new();
    let mut actuator = FakeActuator::new();
    let receipt = gate.handle(&authorized, &c.requested);
    let acted = receipt.acted;
    actuator.act(receipt.clone());

    Ok(serde_json::json!({
        "stage": "receiver",
        "result": if acted { "accepted" } else { "refused" },
        "handling_receipt": receipt,
        // Stage 3a: the act is fake — accepted means the receiver WOULD admit; nothing moved.
        "fake_actuator": true,
        "operational": false,
    }))
}

/// 3b1: validate the chain up to (and including) the receiver correspondence check — but DO NOT burn.
/// The authoritative burn is LinearAccountant's consume, which the orchestrator performs next. On
/// success, emit the exact parameters LA needs (the orchestrator never invents them): the
/// `consumption_event_id` is derived from the operation + capability so a replay maps to the same event
/// (LA's `already_consumed`), and the `eligibility_reference` is carried verbatim end-to-end.
pub fn check_correspondence(bundle: ChainBundle) -> Result<serde_json::Value, String> {
    let c = bundle.execute_chain;

    let office = c.office_outputs.into_office()?;
    let candidate = match decide(&office) {
        TransitionDecision::Admit(cand) => cand,
        TransitionDecision::Refuse { kind, seam, .. } => {
            return Ok(serde_json::json!({
                "result": "refused", "stage": "decision",
                "refusal_kind": kind.as_str(), "refusing_seam": seam.as_str(),
            }));
        }
        TransitionDecision::Escalate(_) => {
            return Ok(serde_json::json!({ "result": "escalate", "stage": "decision" }));
        }
    };

    let reval = match ExecutionRevalidation::revalidate(
        &c.revalidation.standing_ref, c.revalidation.live, c.revalidation.valid_until,
        c.revalidation.now,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Ok(serde_json::json!({
                "result": "refused", "stage": "revalidation", "reason": format!("{e:?}"),
            }));
        }
    };

    let cap_nonce = c.capability.capability_id.clone();
    let eligibility_reference = c.capability.eligibility_reference.clone();
    let scope = c.capability.scope.clone();
    let authorized =
        match AuthorizedTransition::finalize(candidate, c.capability, reval, c.operation) {
            Ok(a) => a,
            Err(e) => {
                return Ok(serde_json::json!({
                    "result": "refused", "stage": "binding", "reason": format!("{e:?}"),
                }));
            }
        };

    let gate = ReceiverGate::new();
    if let Some(r) = gate.check_correspondence(&authorized, &c.requested) {
        return Ok(serde_json::json!({
            "result": "refused", "stage": "receiver", "reason": r.as_str(),
        }));
    }

    // Correspondence holds. Hand the orchestrator the exact LA consume parameters; LA owns the burn.
    let consumption_event_id = format!(
        "{}:{}", authorized.operation().operation_hash, cap_nonce
    );
    Ok(serde_json::json!({
        "result": "correspondence_ok",
        "consumption_event_id": consumption_event_id,
        "eligibility_reference": eligibility_reference,
        "scope": scope,
        "capability_nonce": cap_nonce,
    }))
}
