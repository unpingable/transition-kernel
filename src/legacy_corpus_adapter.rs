//! Legacy corpus adapter: reproduces the historical `agent_governor.corpus.v1` contract.
//!
//! This layer — **not** the kernel core — is permitted to reproduce the full seven-field historical
//! projection, including fields owned by *other* offices (`consumed`, `effect_count`, `operational`) and
//! the one genuine presentation quirk (`gap_accounted` carries a `refusing_seam` even though nothing
//! refuses). It expands a scenario selector into typed office-output fixtures, runs the honest
//! [`crate::transition_core::decide`], then projects the legacy verdict. Keeping this here is what stops
//! `transition_core` from becoming a compressed reimplementation of the whole Python spine.
//!
//! Source of truth: `agent_gov/src/governor/drill_runner.py` + `tests/test_corpus_contract.py` (traced).

use crate::transition_core::{
    decide, AdmissionOutput, ConsumeOutput, OfficeOutputs, RefusalKind, RefusingSeam, SpendabilityOutput,
    StandingOutput, TransitionDecision,
};
use crate::LegacyVerdict;

/// The 8 canonical scenarios (`drill_runner.py:257`). The 9th corpus case reuses `all-green` with an
/// origin override.
pub const SUPPORTED_SCENARIOS: [&str; 12] = [
    "no-standing",
    "standing-expired",
    "wicket-denied",
    "wicket-gap-accounted",
    "replay-budget",
    "all-green",
    "temporal-lapse",
    "temporal-lapse-twin",
    // B5 LA-token quartet (2026-07-03): verified standing + admitted LA, but the
    // consume returns each token-state ConsumptionDecision.
    "scope-mismatch",
    "token-revoked",
    "token-expired",
    "unknown-token",
];

/// The all-green standing digest fixture (`drill_runner.py` `ALL_GREEN_STANDING_DIGEST`).
const ALL_GREEN_STANDING_DIGEST: &str =
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

/// A corpus case `input` block.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CorpusInput {
    pub scenario: String,
    #[serde(default)]
    pub origin_mode: Option<String>,
    /// Case 07: rebuild the finding under this origin (the only synthetic path).
    #[serde(default)]
    pub override_origin_mode: Option<String>,
    #[serde(default)]
    pub confabulate_citation: Option<String>,
}

/// Mechanics the projection needs but the seam-precedence decision doesn't carry.
#[derive(Debug, Clone)]
struct Mechanics {
    effect_count: u32,
    /// Chain mechanically consumed (false for replay even though `effect_count == 1`).
    consumed: bool,
    /// The `gap_accounted` presentation label (a non-refusal that nonetheless reports `wicket_seam`).
    gap_accounted: bool,
}

#[derive(Debug)]
pub enum AdapterError {
    UnsupportedScenario(String),
    /// D3 confabulation is outside the 9-case contract.
    ConfabulationOutOfScope,
}

fn resolve_alias(scenario: &str) -> &str {
    match scenario {
        "already-consumed" => "replay-budget",
        other => other,
    }
}

/// Expand a scenario into office-output fixtures + projection mechanics.
fn expand(scenario: &str) -> (OfficeOutputs, Mechanics) {
    let verified = || StandingOutput::Verified {
        receipt_ref: ALL_GREEN_STANDING_DIGEST.to_string(),
    };
    let scope = "fs_write".to_string();
    let target = "demo".to_string();

    let (standing, spendability, la_admission, la_consume, mech) = match scenario {
        "all-green" => (
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Admitted,
            ConsumeOutput::Consumed,
            Mechanics { effect_count: 1, consumed: true, gap_accounted: false },
        ),
        "no-standing" => (
            StandingOutput::Required, SpendabilityOutput::NotApplicable,
            AdmissionOutput::NotReached, ConsumeOutput::NotReached,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        "standing-expired" => (
            StandingOutput::Expired, SpendabilityOutput::NotApplicable,
            AdmissionOutput::NotReached, ConsumeOutput::NotReached,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        "wicket-denied" => (
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Denied,
            ConsumeOutput::NotReached,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        "wicket-gap-accounted" => (
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Admitted,
            ConsumeOutput::Consumed,
            Mechanics { effect_count: 1, consumed: true, gap_accounted: true },
        ),
        "replay-budget" => (
            // The chain consumes once (effect 1), then a replayed consume returns AlreadyConsumed.
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Admitted,
            ConsumeOutput::AlreadyConsumed,
            Mechanics { effect_count: 1, consumed: false, gap_accounted: false },
        ),
        "temporal-lapse" => (
            verified(), SpendabilityOutput::Unbounded { block: temporal_lapse_block() },
            AdmissionOutput::NotReached, ConsumeOutput::NotReached,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        "temporal-lapse-twin" => (
            verified(), SpendabilityOutput::Bounded, AdmissionOutput::Admitted,
            ConsumeOutput::Consumed,
            Mechanics { effect_count: 1, consumed: true, gap_accounted: false },
        ),
        // B5 quartet: standing verified, LA admits, consume refuses on the token
        // state — no effect spent (effect 0, not consumed).
        "scope-mismatch" => (
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Admitted,
            ConsumeOutput::ScopeMismatch,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        "token-revoked" => (
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Admitted,
            ConsumeOutput::Revoked,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        "token-expired" => (
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Admitted,
            ConsumeOutput::Expired,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        "unknown-token" => (
            verified(), SpendabilityOutput::NotApplicable, AdmissionOutput::Admitted,
            ConsumeOutput::UnknownToken,
            Mechanics { effect_count: 0, consumed: false, gap_accounted: false },
        ),
        _ => unreachable!("caller validates scenario before expand"),
    };

    (OfficeOutputs { standing, spendability, la_admission, la_consume, scope, target }, mech)
}

/// The two-clock block for the hero specimen (case 08), matching `expected_receipt_block`.
fn temporal_lapse_block() -> serde_json::Value {
    serde_json::json!({
        "gap_basis": {
            "kind": "monotonic",
            "source": "process_monotonic",
            "epoch": "boot:demo-single-host",
            "start_ns": 40_000_000_000u64,
            "end_ns": 51_000_000_000u64
        },
        "gap_ns": 11_000_000_000u64,
        "bound_ns": 10_000_000_000u64,
        "overage_ns": 1_000_000_000u64,
        "lapse_coverage": "exceeded_horizon",
        "wall": {
            "observed_at": "2026-06-09T00:00:40Z",
            "uncertainty_ms": serde_json::Value::Null,
            "source": "system_clock_unsynced",
            "role": "display_only"
        }
    })
}

/// Run a corpus input through the office fixtures + honest decision, and project the legacy verdict.
/// Returns both the kernel's decision (its own clause) and the projected legacy contract.
pub fn run(input: &CorpusInput) -> Result<(TransitionDecision, LegacyVerdict), AdapterError> {
    if input.confabulate_citation.is_some() {
        return Err(AdapterError::ConfabulationOutOfScope);
    }
    let scenario = resolve_alias(&input.scenario);
    if !SUPPORTED_SCENARIOS.contains(&scenario) {
        return Err(AdapterError::UnsupportedScenario(scenario.to_string()));
    }

    let (office, mech) = expand(scenario);
    let decision = decide(&office);

    // The effective origin is the override if present (case 07), else the declared origin.
    let effective_origin = input
        .override_origin_mode
        .as_deref()
        .or(input.origin_mode.as_deref())
        .unwrap_or("drill");
    // Wall-1 fence: only an `observed` origin is operational. Never true in the corpus.
    let operational = effective_origin == "observed";

    let verdict = project(&decision, &mech, operational);
    Ok((decision, verdict))
}

/// Project the seven-field legacy verdict from the honest decision + mechanics.
fn project(decision: &TransitionDecision, mech: &Mechanics, operational: bool) -> LegacyVerdict {
    // The `gap_accounted` presentation label: a non-refusal that nonetheless reports a refusing seam.
    // This is the one place historical presentation diverges from "a seam only names a refusal"; it is
    // owned here, in the adapter, not in the core. (Candidate divergence — see docs/NON_CORRESPONDENCE.)
    if mech.gap_accounted {
        return LegacyVerdict {
            outcome: "gap_accounted".to_string(),
            refusal_kind: Some(RefusalKind::AdmissionGapAccounted.as_str().to_string()),
            refusing_seam: Some(RefusingSeam::WicketSeam.as_str().to_string()),
            effect_count: mech.effect_count,
            consumed: mech.consumed,
            operational,
            proposal_packet_present: mech.consumed,
        };
    }

    match decision {
        TransitionDecision::Refuse { kind, seam, .. } => LegacyVerdict {
            outcome: "refused".to_string(),
            refusal_kind: Some(kind.as_str().to_string()),
            refusing_seam: Some(seam.as_str().to_string()),
            effect_count: mech.effect_count,
            consumed: mech.consumed,
            operational,
            proposal_packet_present: false,
        },
        TransitionDecision::Admit(_) => {
            let proposal_packet_present = mech.consumed;
            LegacyVerdict {
                outcome: "consumed".to_string(),
                refusal_kind: None,
                refusing_seam: None,
                effect_count: mech.effect_count,
                consumed: mech.consumed,
                operational,
                proposal_packet_present,
            }
        }
        TransitionDecision::Escalate(_) => LegacyVerdict {
            // No corpus case escalates; kept total for honesty.
            outcome: "escalated".to_string(),
            refusal_kind: None,
            refusing_seam: None,
            effect_count: mech.effect_count,
            consumed: false,
            operational,
            proposal_packet_present: false,
        },
    }
}
