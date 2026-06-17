//! GAP-3: typed-memory custody — "belief is free; inheritance is not."
//!
//! The gap spec calls this the most dangerous seam: the path
//! `remembered → restated → treated-as-witnessed → relied-upon` launders interpretation into inherited
//! fact while every individual step looks respectable. This module makes that laundering
//! **structurally unconstructable**: a remembered artifact cannot become standing-eligible evidence
//! except through an explicit [`PromotionReceipt`] that survives custody checks (consumer, freshness,
//! scope, coverage). There is no `From<MemoryArtifact> for ReliedUpon`, no `Default`, no coercion.
//!
//! This is upstream of the transition office: it governs whether a remembered basis may even be
//! *presented* as standing. It is non-operational — it mints no authority and no effect; it only decides
//! whether memory may be relied upon, or is advisory-only. There is deliberately **no third case**.
//!
//! Lean anchors: `MultiConsumerAdoption` (consumer indexing), `NoFreeStandingReadout` (standing only
//! from a stipulated root/promotion — no free readout), `TemporalCustody`/`RetroactiveLegitimation`
//! (freshness/supersession at presentation).

use crate::transition_core::StandingOutput;

/// The use-level lattice. Strictly ordered: a higher level subsumes the rights below it, but moving
/// *up* requires an explicit promotion — never a cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UseLevel {
    MayRemember = 0,
    MayQuote = 1,
    MayPropose = 2,
    MayRely = 3,
}

impl UseLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            UseLevel::MayRemember => "may_remember",
            UseLevel::MayQuote => "may_quote",
            UseLevel::MayPropose => "may_propose",
            UseLevel::MayRely => "may_rely",
        }
    }
}

/// What *kind* of remembered thing this is (the epistemic class never auto-promotes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicClass {
    Observation,
    Derivation,
    Narrative,
    Inherited,
}

/// Custody metadata: who recorded what, for whom, in what scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Custody {
    pub origin: String,
    pub recorder: String,
    pub subject: String,
    pub scope: String,
    /// The single consumer this artifact was recorded *for*. Adoption is consumer-indexed
    /// (MultiConsumerAdoption): A's memory is not B's standing.
    pub intended_consumer: String,
    pub recorded_at: u64,
    /// If this artifact supersedes a prior one, its digest.
    pub supersedes: Option<String>,
}

/// Freshness / supersession of a remembered artifact, evaluated at *presentation* time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Freshness {
    pub valid_until: u64,
    pub superseded: bool,
}

/// A remembered artifact. By itself it is **advisory at most** — `granted_level` records the level it
/// was recorded as permitting (default `MayRemember`); rising to `MayRely` requires promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryArtifact {
    pub custody: Custody,
    pub epistemic_class: EpistemicClass,
    pub granted_level: UseLevel,
    pub freshness: Freshness,
    pub content_digest: String,
}

/// An explicit authorization to move a *specific* artifact up to a target level, for a *specific*
/// consumer, with its own validity. Issued by an authority (operator/root), never self-minted by the
/// rememberer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionReceipt {
    pub artifact_digest: String,
    pub subject: String,
    pub to_level: UseLevel,
    pub authorized_consumer: String,
    pub issued_by: String,
    pub valid_until: u64,
}

/// The act of presenting a remembered claim to be relied upon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    pub consumer: String,
    pub now: u64,
    pub scope: String,
}

/// A claim that has been promoted to rely-eligible. **Sealed**: constructable only by [`promote`].
/// There is no public constructor, no `Default`, and no conversion from [`MemoryArtifact`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReliedUpon {
    basis_digest: String,
    subject: String,
    consumer: String,
    _seal: (),
}

impl ReliedUpon {
    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
    pub fn consumer(&self) -> &str {
        &self.consumer
    }
}

/// The closed refusal taxonomy for memory custody.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRefusal {
    /// The promotion receipt does not raise this artifact to `MayRely`.
    NoPromotionToRely,
    /// The receipt does not cover this artifact (digest/subject mismatch).
    PromotionDoesNotCover,
    /// The presenting consumer is not the intended/authorized consumer.
    ConsumerMismatch,
    /// The artifact is superseded, or freshness lapsed at presentation (or the receipt expired).
    StaleOrSuperseded,
    /// The presentation scope does not match the artifact's custody scope.
    ScopeMismatch,
}

impl MemoryRefusal {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryRefusal::NoPromotionToRely => "no_promotion_to_rely",
            MemoryRefusal::PromotionDoesNotCover => "promotion_does_not_cover",
            MemoryRefusal::ConsumerMismatch => "consumer_mismatch",
            MemoryRefusal::StaleOrSuperseded => "stale_or_superseded",
            MemoryRefusal::ScopeMismatch => "scope_mismatch",
        }
    }
}

/// The ONLY path from a remembered artifact to a rely-eligible claim. Every custody check must pass; any
/// failure is a typed refusal. This function is the single seam where `remembered → relied_upon` is
/// permitted, and it is permitted only under an explicit promotion that survives custody.
pub fn promote(
    artifact: &MemoryArtifact,
    receipt: &PromotionReceipt,
    presentation: &Presentation,
) -> Result<ReliedUpon, MemoryRefusal> {
    // 1. The receipt must actually promote to rely.
    if receipt.to_level != UseLevel::MayRely {
        return Err(MemoryRefusal::NoPromotionToRely);
    }
    // 2. The receipt must cover *this* artifact (content + subject).
    if receipt.artifact_digest != artifact.content_digest
        || receipt.subject != artifact.custody.subject
    {
        return Err(MemoryRefusal::PromotionDoesNotCover);
    }
    // 3. Consumer indexing: the presenter must be both the artifact's intended consumer AND the
    //    receipt's authorized consumer. A's adoption is not B's standing.
    if artifact.custody.intended_consumer != presentation.consumer
        || receipt.authorized_consumer != presentation.consumer
    {
        return Err(MemoryRefusal::ConsumerMismatch);
    }
    // 4. Freshness / supersession at presentation time (not at citation time).
    if artifact.freshness.superseded
        || presentation.now > artifact.freshness.valid_until
        || presentation.now > receipt.valid_until
    {
        return Err(MemoryRefusal::StaleOrSuperseded);
    }
    // 5. Scope must match.
    if artifact.custody.scope != presentation.scope {
        return Err(MemoryRefusal::ScopeMismatch);
    }
    Ok(ReliedUpon {
        basis_digest: artifact.content_digest.clone(),
        subject: artifact.custody.subject.clone(),
        consumer: presentation.consumer.clone(),
        _seal: (),
    })
}

/// A claim presented to the kernel. `Relied` can ONLY hold a [`ReliedUpon`] (which only [`promote`] can
/// make); `Advisory` holds the raw artifact. There is no third variant — a remembered thing is either
/// promoted-and-relied or advisory-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentedClaim {
    Relied(ReliedUpon),
    Advisory(MemoryArtifact),
}

/// The memory office's clause: may this presented claim be treated as standing evidence?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryVerdict {
    /// Promoted + custody-clean: a typed claim eligible to back a standing reference.
    StandingEligible {
        basis_digest: String,
        subject: String,
        consumer: String,
    },
    /// Not relied: advisory-only, carrying no authority. (The `reason` names why it is not standing.)
    AdvisoryOnly { reason: &'static str },
}

/// Assess a presented claim. Non-operational: this mints no authority and confers no effect; it only
/// classifies a remembered claim as standing-eligible or advisory-only.
pub fn assess(presented: &PresentedClaim) -> MemoryVerdict {
    match presented {
        PresentedClaim::Relied(r) => MemoryVerdict::StandingEligible {
            basis_digest: r.basis_digest.clone(),
            subject: r.subject.clone(),
            consumer: r.consumer.clone(),
        },
        PresentedClaim::Advisory(_) => MemoryVerdict::AdvisoryOnly {
            reason: "memory presented without rely-promotion is advisory-only",
        },
    }
}

/// Bridge to the transition office: a standing-eligible memory verdict may back a verified standing
/// reference; advisory-only memory yields no standing (the basis was never relied-upon-eligible).
pub fn to_standing_output(verdict: &MemoryVerdict) -> StandingOutput {
    match verdict {
        MemoryVerdict::StandingEligible { basis_digest, .. } => StandingOutput::Verified {
            receipt_ref: basis_digest.clone(),
        },
        MemoryVerdict::AdvisoryOnly { .. } => StandingOutput::Required,
    }
}

/// The JSON wire boundary for the memory-custody office (the `memory_claim` CLI mode).
pub mod wire {
    use super::*;
    use crate::transition_core::{
        decide, AdmissionOutput, ConsumeOutput, OfficeOutputs, SpendabilityOutput,
        TransitionDecision,
    };
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct MemoryClaim {
        pub memory_claim: Claim,
    }

    #[derive(Debug, Deserialize)]
    pub struct Claim {
        pub artifact: WireArtifact,
        #[serde(default)]
        pub promotion_receipt: Option<WireReceipt>,
        pub presentation: WirePresentation,
    }

    #[derive(Debug, Deserialize)]
    pub struct WireArtifact {
        pub origin: String,
        pub recorder: String,
        pub subject: String,
        pub scope: String,
        pub intended_consumer: String,
        pub recorded_at: u64,
        #[serde(default)]
        pub supersedes: Option<String>,
        pub epistemic_class: String,
        #[serde(default = "default_may_remember")]
        pub granted_level: String,
        pub valid_until: u64,
        #[serde(default)]
        pub superseded: bool,
        pub content_digest: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct WireReceipt {
        pub artifact_digest: String,
        pub subject: String,
        pub to_level: String,
        pub authorized_consumer: String,
        pub issued_by: String,
        pub valid_until: u64,
    }

    #[derive(Debug, Deserialize)]
    pub struct WirePresentation {
        pub consumer: String,
        pub now: u64,
        pub scope: String,
    }

    fn default_may_remember() -> String {
        "may_remember".to_string()
    }

    fn parse_level(s: &str) -> Result<UseLevel, String> {
        Ok(match s {
            "may_remember" => UseLevel::MayRemember,
            "may_quote" => UseLevel::MayQuote,
            "may_propose" => UseLevel::MayPropose,
            "may_rely" => UseLevel::MayRely,
            other => return Err(format!("unknown use level: {other:?}")),
        })
    }

    fn parse_class(s: &str) -> Result<EpistemicClass, String> {
        Ok(match s {
            "observation" => EpistemicClass::Observation,
            "derivation" => EpistemicClass::Derivation,
            "narrative" => EpistemicClass::Narrative,
            "inherited" => EpistemicClass::Inherited,
            other => return Err(format!("unknown epistemic class: {other:?}")),
        })
    }

    /// The resolved outcome of a memory claim (shared by the standalone and gated modes).
    enum Resolution {
        Eligible { digest: String, subject: String, consumer: String },
        Advisory { reason: &'static str },
        Refused { reason: MemoryRefusal },
    }

    fn resolve(c: Claim) -> Result<Resolution, String> {
        let artifact = MemoryArtifact {
            custody: Custody {
                origin: c.artifact.origin,
                recorder: c.artifact.recorder,
                subject: c.artifact.subject,
                scope: c.artifact.scope,
                intended_consumer: c.artifact.intended_consumer,
                recorded_at: c.artifact.recorded_at,
                supersedes: c.artifact.supersedes,
            },
            epistemic_class: parse_class(&c.artifact.epistemic_class)?,
            granted_level: parse_level(&c.artifact.granted_level)?,
            freshness: Freshness {
                valid_until: c.artifact.valid_until,
                superseded: c.artifact.superseded,
            },
            content_digest: c.artifact.content_digest,
        };
        let presentation = Presentation {
            consumer: c.presentation.consumer,
            now: c.presentation.now,
            scope: c.presentation.scope,
        };

        let presented = match c.promotion_receipt {
            None => PresentedClaim::Advisory(artifact),
            Some(r) => {
                let receipt = PromotionReceipt {
                    artifact_digest: r.artifact_digest,
                    subject: r.subject,
                    to_level: parse_level(&r.to_level)?,
                    authorized_consumer: r.authorized_consumer,
                    issued_by: r.issued_by,
                    valid_until: r.valid_until,
                };
                match promote(&artifact, &receipt, &presentation) {
                    Ok(relied) => PresentedClaim::Relied(relied),
                    // A presented-with-receipt-but-custody-failed is a refusal, distinct from the
                    // advisory case of never having been promoted.
                    Err(reason) => return Ok(Resolution::Refused { reason }),
                }
            }
        };

        Ok(match assess(&presented) {
            MemoryVerdict::StandingEligible { basis_digest, subject, consumer } => {
                Resolution::Eligible { digest: basis_digest, subject, consumer }
            }
            MemoryVerdict::AdvisoryOnly { reason } => Resolution::Advisory { reason },
        })
    }

    /// Decide a memory claim and return the wire JSON: `standing_eligible` | `advisory_only` |
    /// `refused(reason)`. Non-operational by construction.
    pub fn decide_memory(claim: MemoryClaim) -> Result<serde_json::Value, String> {
        Ok(match resolve(claim.memory_claim)? {
            Resolution::Eligible { digest, subject, consumer } => serde_json::json!({
                "verdict": "standing_eligible",
                "basis_digest": digest,
                "subject": subject,
                "consumer": consumer,
                "standing_output": "verified",
                "operational": false,
            }),
            Resolution::Advisory { reason } => serde_json::json!({
                "verdict": "advisory_only",
                "reason": reason,
                "standing_output": "required",
                "operational": false,
            }),
            Resolution::Refused { reason } => serde_json::json!({
                "verdict": "refused",
                "reason": reason.as_str(),
                "standing_output": "required",
                "operational": false,
            }),
        })
    }

    // -- GAP-3b: memory-gated transition (memory custody -> standing -> transition) ------------- //

    #[derive(Debug, Deserialize)]
    pub struct MemoryGatedTransition {
        pub memory_gated_transition: GatedClaim,
    }

    #[derive(Debug, Deserialize)]
    pub struct GatedClaim {
        pub memory_claim: Claim,
        pub operation: WireOperation,
    }

    #[derive(Debug, Deserialize)]
    pub struct WireOperation {
        #[serde(default = "default_not_applicable")]
        pub spendability: String,
        #[serde(default = "default_admitted")]
        pub la_admission: String,
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

    fn parse_spendability(s: &str) -> Result<SpendabilityOutput, String> {
        Ok(match s {
            "not_applicable" => SpendabilityOutput::NotApplicable,
            "bounded" => SpendabilityOutput::Bounded,
            "unbounded" => SpendabilityOutput::Unbounded { block: serde_json::Value::Null },
            other => return Err(format!("unknown spendability: {other:?}")),
        })
    }
    fn parse_admission(s: &str) -> Result<AdmissionOutput, String> {
        Ok(match s {
            "admitted" => AdmissionOutput::Admitted,
            "denied" => AdmissionOutput::Denied,
            "not_reached" => AdmissionOutput::NotReached,
            other => return Err(format!("unknown la_admission: {other:?}")),
        })
    }
    fn parse_consume(s: &str) -> Result<ConsumeOutput, String> {
        Ok(match s {
            "consumed" => ConsumeOutput::Consumed,
            "already_consumed" => ConsumeOutput::AlreadyConsumed,
            "not_reached" => ConsumeOutput::NotReached,
            other => return Err(format!("unknown la_consume: {other:?}")),
        })
    }

    /// The combined office: derive Standing from memory custody, then run the transition decision. The
    /// live supervisor's Standing input flows THROUGH custody — a remembered claim without a `MayRely`
    /// promotion yields `StandingOutput::Required`, so the transition refuses for want of standing.
    /// Non-operational by construction (mints a candidate, never an authority).
    pub fn decide_memory_gated(g: MemoryGatedTransition) -> Result<serde_json::Value, String> {
        let claim = g.memory_gated_transition;
        let resolution = resolve(claim.memory_claim)?;
        let (standing, memory_json) = match resolution {
            Resolution::Eligible { digest, subject, consumer } => (
                StandingOutput::Verified { receipt_ref: digest.clone() },
                serde_json::json!({
                    "verdict": "standing_eligible", "basis_digest": digest,
                    "subject": subject, "consumer": consumer,
                }),
            ),
            Resolution::Advisory { reason } => (
                StandingOutput::Required,
                serde_json::json!({ "verdict": "advisory_only", "reason": reason }),
            ),
            Resolution::Refused { reason } => (
                StandingOutput::Required,
                serde_json::json!({ "verdict": "refused", "reason": reason.as_str() }),
            ),
        };

        let op = claim.operation;
        let office = OfficeOutputs {
            standing,
            spendability: parse_spendability(&op.spendability)?,
            la_admission: parse_admission(&op.la_admission)?,
            la_consume: parse_consume(&op.la_consume)?,
            scope: op.scope,
            target: op.target,
        };
        let decision = decide(&office);
        let (tag, refusal_kind, refusing_seam) = match &decision {
            TransitionDecision::Admit(_) => {
                ("admit", serde_json::Value::Null, serde_json::Value::Null)
            }
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
            "verdict": { "refusal_kind": refusal_kind, "refusing_seam": refusing_seam, "operational": false },
            "memory": memory_json,
        }))
    }
}
