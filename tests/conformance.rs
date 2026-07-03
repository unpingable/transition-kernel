//! A1 conformance: the Rust kernel must reproduce every frozen `agent_governor.corpus.v1` verdict
//! byte-for-byte on the seven contract fields.
//!
//! Divergence discipline (per plan): a mismatch must **(1)** emit a classified divergence receipt **and
//! (2)** fail the run. Only an operator-approved divergence manifest (`vectors/legacy/
//! divergence_manifest.json`) converts a specific mismatch into an accepted tightening/contract
//! revision. Absent that, "classified divergence" is a failure, not an articulate green test.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use transition_kernel::legacy_corpus_adapter::{self, CorpusInput};
use transition_kernel::LegacyVerdict;

#[derive(Debug, Deserialize)]
struct CorpusCase {
    case: String,
    input: CorpusInput,
    expected_verdict: LegacyVerdict,
}

/// An operator-approved divergence: by case name. Presence here converts a mismatch from a failure into
/// an accepted, reviewed change. Empty/absent in A1 (all nine must match).
#[derive(Debug, Deserialize, Default)]
struct DivergenceManifest {
    #[serde(default)]
    accepted_cases: Vec<String>,
}

fn legacy_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vectors/legacy")
}

fn load_manifest() -> DivergenceManifest {
    let path = legacy_dir().join("divergence_manifest.json");
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).expect("divergence_manifest.json is malformed"),
        Err(_) => DivergenceManifest::default(),
    }
}

/// A classified divergence receipt — emitted on any mismatch, accepted or not.
fn emit_divergence_receipt(case: &str, expected: &LegacyVerdict, actual: &LegacyVerdict) {
    // Classification is left `unclassified` until an operator triages it into the manifest with one of:
    // rust_bug | python_accidental | underspecified_contract | deliberate_tightening.
    let receipt = serde_json::json!({
        "divergence_receipt": {
            "case": case,
            "classification": "unclassified",
            "expected": expected,
            "actual": actual,
        }
    });
    eprintln!("{}", serde_json::to_string_pretty(&receipt).unwrap());
}

#[test]
fn legacy_corpus_byte_for_byte() {
    let manifest = load_manifest();
    let accepted: HashSet<&str> = manifest.accepted_cases.iter().map(String::as_str).collect();

    let mut cases = 0;
    let mut unaccepted_divergences = 0;

    for entry in std::fs::read_dir(legacy_dir()).expect("vectors/legacy missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("divergence_manifest.json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let case: CorpusCase = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{}: malformed corpus case: {e}", path.display()));
        cases += 1;

        let (_decision, actual) = legacy_corpus_adapter::run(&case.input)
            .unwrap_or_else(|e| panic!("{}: adapter refused: {e:?}", case.case));

        if actual != case.expected_verdict {
            emit_divergence_receipt(&case.case, &case.expected_verdict, &actual);
            if accepted.contains(case.case.as_str()) {
                eprintln!("  (accepted by operator divergence manifest)");
            } else {
                unaccepted_divergences += 1;
            }
        }
    }

    assert_eq!(cases, 13, "expected all 13 frozen corpus cases");
    assert_eq!(
        unaccepted_divergences, 0,
        "{unaccepted_divergences} unaccepted divergence(s) — see receipts above"
    );
}
