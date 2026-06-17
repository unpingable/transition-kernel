//! Quarantined specimen: TemporalCustody / stale-at-execution.
//!
//! See `specimens/temporal_custody/README.md`. Disposition: UNREALIZED.
//!
//! This is fenced out of A1's *claim acceptance* (the legacy conformance corpus is the A1 gate), but it
//! is **not** permitted to rot: it compiles, runs deterministically, and its assertions pin the current
//! observed behavior so that any drift fails CI until the README + `docs/LEAN_OBLIGATIONS.md` disposition
//! are updated to match.

use transition_kernel::transition_core::{
    decide, AdmissionOutput, ConsumeOutput, OfficeOutputs, RefusingSeam, SpendabilityOutput,
    StandingOutput, TransitionDecision,
};

/// The disposition this specimen asserts. If you change kernel behavior such that the assertions below
/// no longer hold, you must also update this constant, the README, and the ledger row — in the same
/// change. That coupling is the anti-rot mechanism.
const DISPOSITION: &str = "UNREALIZED";

fn outputs(spendability: SpendabilityOutput) -> OfficeOutputs {
    OfficeOutputs {
        standing: StandingOutput::Verified {
            receipt_ref: "d".repeat(64),
        },
        spendability,
        la_admission: AdmissionOutput::Admitted,
        la_consume: ConsumeOutput::Consumed,
        scope: "fs_write".to_string(),
        target: "demo".to_string(),
    }
}

/// REALIZED slice: when the gate reports the citation-time observation lapsed before spend (`Unbounded`),
/// the kernel refuses at the standing-spendability seam, before any capacity is reserved.
#[test]
fn stale_at_execution_gate_verdict_is_honored() {
    let block = serde_json::json!({ "lapse_coverage": "exceeded_horizon" });
    let decision = decide(&outputs(SpendabilityOutput::Unbounded { block }));
    match decision {
        TransitionDecision::Refuse { seam, .. } => {
            assert_eq!(seam, RefusingSeam::StandingSpendabilitySeam);
        }
        other => panic!("expected refusal at standing_spendability_seam, got {other:?}"),
    }
}

/// UNREALIZED slice: the kernel owns no execution clock. `decide` takes no execution-time input, so it
/// cannot itself distinguish "fresh at citation, stale at execution" — it trusts the gate's output. We
/// pin that fact: a `Bounded` gate output ADMITS regardless of any (absent) execution timing. When Stage
/// 3 gives the kernel its own execution recheck, THIS assertion is expected to change, which will fail
/// the specimen and force a disposition update.
#[test]
fn kernel_has_no_independent_execution_recheck_yet() {
    assert_eq!(DISPOSITION, "UNREALIZED", "disposition drift: update README + ledger together");
    let decision = decide(&outputs(SpendabilityOutput::Bounded));
    assert!(
        matches!(decision, TransitionDecision::Admit(_)),
        "A1 admits a Bounded gate output with no execution-time recheck of its own; if this changed, \
         the TemporalCustody obligation has advanced and the disposition must be updated"
    );
}
