//! Live probe input: a direct "cooked context" of already-issued office outputs.
//!
//! The legacy adapter expands a *scenario selector* into office fixtures; the live path instead carries
//! the actual office outputs of a real proposed operation, so `transition_core::decide` runs on real
//! inputs. The live projection emits only the office's own clause (decision + refusal kind/seam) and is
//! **non-operational by construction**: this path mints a candidate, never an authority, so `operational`
//! is always `false`.

use serde::Deserialize;

use crate::transition_core::{
    decide, AdmissionOutput, ConsumeOutput, OfficeOutputs, SpendabilityOutput, StandingOutput,
    TransitionDecision,
};

#[derive(Debug, Deserialize)]
pub struct LiveProbe {
    pub office_outputs: WireOfficeOutputs,
}

#[derive(Debug, Deserialize)]
pub struct WireOfficeOutputs {
    /// "verified" | "required" | "expired"
    pub standing: String,
    #[serde(default)]
    pub standing_receipt_ref: Option<String>,
    /// "not_applicable" | "bounded" | "unbounded"
    #[serde(default = "default_not_applicable")]
    pub spendability: String,
    /// "admitted" | "denied" | "not_reached"
    #[serde(default = "default_admitted")]
    pub la_admission: String,
    /// "consumed" | "already_consumed" | "not_reached"
    #[serde(default = "default_consumed")]
    pub la_consume: String,
    pub scope: String,
    pub target: String,
}

fn default_not_applicable() -> String {
    "not_applicable".to_string()
}
fn default_admitted() -> String {
    "admitted".to_string()
}
fn default_consumed() -> String {
    "consumed".to_string()
}

impl WireOfficeOutputs {
    pub fn into_office(self) -> Result<OfficeOutputs, String> {
        let standing = match self.standing.as_str() {
            "verified" => StandingOutput::Verified {
                receipt_ref: self
                    .standing_receipt_ref
                    .ok_or("standing=verified requires standing_receipt_ref")?,
            },
            "required" => StandingOutput::Required,
            "expired" => StandingOutput::Expired,
            other => return Err(format!("unknown standing: {other:?}")),
        };
        let spendability = match self.spendability.as_str() {
            "not_applicable" => SpendabilityOutput::NotApplicable,
            "bounded" => SpendabilityOutput::Bounded,
            "unbounded" => SpendabilityOutput::Unbounded {
                block: serde_json::Value::Null,
            },
            other => return Err(format!("unknown spendability: {other:?}")),
        };
        let la_admission = match self.la_admission.as_str() {
            "admitted" => AdmissionOutput::Admitted,
            "denied" => AdmissionOutput::Denied,
            "not_reached" => AdmissionOutput::NotReached,
            other => return Err(format!("unknown la_admission: {other:?}")),
        };
        let la_consume = match self.la_consume.as_str() {
            "consumed" => ConsumeOutput::Consumed,
            "already_consumed" => ConsumeOutput::AlreadyConsumed,
            "not_reached" => ConsumeOutput::NotReached,
            other => return Err(format!("unknown la_consume: {other:?}")),
        };
        Ok(OfficeOutputs {
            standing,
            spendability,
            la_admission,
            la_consume,
            scope: self.scope,
            target: self.target,
        })
    }
}

/// Decide on a live probe and return the wire JSON: the office's own clause, non-operational.
pub fn decide_live(probe: LiveProbe) -> Result<serde_json::Value, String> {
    let office = probe.office_outputs.into_office()?;
    let decision = decide(&office);
    let (tag, refusal_kind, refusing_seam) = match &decision {
        TransitionDecision::Admit(_) => ("admit", serde_json::Value::Null, serde_json::Value::Null),
        TransitionDecision::Refuse { kind, seam, .. } => (
            "refuse",
            serde_json::Value::String(kind.as_str().to_string()),
            serde_json::Value::String(seam.as_str().to_string()),
        ),
        TransitionDecision::Escalate(_) => {
            ("escalate", serde_json::Value::Null, serde_json::Value::Null)
        }
    };
    Ok(serde_json::json!({
        "decision": tag,
        "verdict": {
            "refusal_kind": refusal_kind,
            "refusing_seam": refusing_seam,
            // Non-operational by construction: this path mints a candidate, never an authority.
            "operational": false,
        }
    }))
}
