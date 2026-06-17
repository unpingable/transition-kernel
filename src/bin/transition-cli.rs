//! transition-cli — the JSON boundary for the transition office.
//!
//! Reads one JSON object on stdin (a corpus case `{"input": {...}}` or a bare input object), runs the
//! legacy-corpus adapter, and emits the projected seven-field verdict plus the kernel's own decision on
//! stdout. Fail-closed (`la_cli`-style): any malformed input or unsupported scenario emits
//! `{"error": ..., "fail_closed": true}` and exits non-zero — never a verdict.

use std::io::Read;

use transition_kernel::legacy_corpus_adapter::{self, CorpusInput};
use transition_kernel::transition_core::TransitionDecision;

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(msg) => {
            // Fail-closed: structured error, never a decision.
            let err = serde_json::json!({ "error": msg, "fail_closed": true });
            println!("{}", err);
            1
        }
    });
}

fn run() -> Result<(), String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("stdin read failed: {e}"))?;

    let root: serde_json::Value =
        serde_json::from_str(&buf).map_err(|e| format!("invalid json: {e}"))?;

    // Live mode: a direct office-outputs probe (the supervisor hot path). Decides on real office
    // outputs via transition_core, non-operationally.
    if root.get("office_outputs").is_some() {
        let probe: transition_kernel::live::LiveProbe =
            serde_json::from_value(root).map_err(|e| format!("invalid office_outputs: {e}"))?;
        let out = transition_kernel::live::decide_live(probe)?;
        println!("{}", serde_json::to_string(&out).map_err(|e| e.to_string())?);
        return Ok(());
    }

    // GAP-3 memory-custody mode: may a remembered claim be relied upon as standing evidence?
    if root.get("memory_claim").is_some() {
        let claim: transition_kernel::memory_custody::wire::MemoryClaim =
            serde_json::from_value(root).map_err(|e| format!("invalid memory_claim: {e}"))?;
        let out = transition_kernel::memory_custody::wire::decide_memory(claim)?;
        println!("{}", serde_json::to_string(&out).map_err(|e| e.to_string())?);
        return Ok(());
    }

    // GAP-3b memory-gated transition: derive Standing through custody, then decide. The live
    // supervisor's Standing input flows through memory_custody — unpromoted memory cannot stamp standing.
    if root.get("memory_gated_transition").is_some() {
        let g: transition_kernel::memory_custody::wire::MemoryGatedTransition =
            serde_json::from_value(root).map_err(|e| format!("invalid memory_gated_transition: {e}"))?;
        let out = transition_kernel::memory_custody::wire::decide_memory_gated(g)?;
        println!("{}", serde_json::to_string(&out).map_err(|e| e.to_string())?);
        return Ok(());
    }

    // Stage 3a: drive the whole consequence chain to a FAKE receiver/actuator. No real effect.
    if root.get("execute_chain").is_some() {
        let bundle: transition_kernel::chain::ChainBundle =
            serde_json::from_value(root).map_err(|e| format!("invalid execute_chain: {e}"))?;
        let out = transition_kernel::chain::execute_chain(bundle)?;
        println!("{}", serde_json::to_string(&out).map_err(|e| e.to_string())?);
        return Ok(());
    }

    // Stage 3b1: validate up to receiver correspondence (NO burn) and emit LA consume params. LA owns
    // the authoritative burn; the kernel never invents the consumption_event_id.
    if root.get("correspondence_check").is_some() {
        // Reuse the execute_chain bundle shape, keyed differently.
        let mut v = root;
        let inner = v.get_mut("correspondence_check").map(|x| x.take())
            .ok_or("missing correspondence_check")?;
        let bundle: transition_kernel::chain::ChainBundle =
            serde_json::from_value(serde_json::json!({ "execute_chain": inner }))
                .map_err(|e| format!("invalid correspondence_check: {e}"))?;
        let out = transition_kernel::chain::check_correspondence(bundle)?;
        println!("{}", serde_json::to_string(&out).map_err(|e| e.to_string())?);
        return Ok(());
    }

    // Legacy mode: a corpus case ({"input": {...}}) or a bare scenario input object.
    let input_value = match root.get("input") {
        Some(v) => v.clone(),
        None => root,
    };
    let input: CorpusInput = serde_json::from_value(input_value)
        .map_err(|e| format!("invalid input block: {e}"))?;

    let (decision, verdict) = legacy_corpus_adapter::run(&input)
        .map_err(|e| format!("adapter refused: {e:?}"))?;

    let decision_tag = match &decision {
        TransitionDecision::Admit(_) => "admit",
        TransitionDecision::Refuse { .. } => "refuse",
        TransitionDecision::Escalate(_) => "escalate",
    };

    let out = serde_json::json!({
        "decision": decision_tag,
        "verdict": verdict,
    });
    println!("{}", serde_json::to_string(&out).map_err(|e| e.to_string())?);
    Ok(())
}
