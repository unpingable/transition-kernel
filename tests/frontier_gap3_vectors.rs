//! GAP-3 frontier vectors driven through the wire boundary (the `memory_claim` CLI contract).
//!
//! These vectors live in `vectors/frontier/`, kept SEPARATE from the legacy conformance corpus — they
//! govern newly specified memory-custody semantics and are never grandfathered into `vectors/legacy/`.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use transition_kernel::memory_custody::wire::{decide_memory, MemoryClaim};

#[derive(Debug, Deserialize)]
struct Expected {
    verdict: String,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    standing_output: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VectorMeta {
    case: String,
    expected: Expected,
}

fn frontier_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vectors/frontier/gap3")
}

#[test]
fn gap3_frontier_vectors() {
    let mut count = 0;
    for entry in std::fs::read_dir(frontier_dir()).expect("vectors/frontier/gap3 missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).unwrap();
        // MemoryClaim ignores the `expected`/`schema` keys; VectorMeta ignores `memory_claim`.
        let claim: MemoryClaim = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{}: bad memory_claim: {e}", path.display()));
        let meta: VectorMeta = serde_json::from_str(&raw).unwrap();
        count += 1;

        let out = decide_memory(claim).unwrap_or_else(|e| panic!("{}: {e}", meta.case));
        assert_eq!(
            out["verdict"], meta.expected.verdict,
            "{}: verdict mismatch", meta.case
        );
        if let Some(reason) = &meta.expected.reason {
            assert_eq!(out["reason"], *reason, "{}: reason mismatch", meta.case);
        }
        if let Some(so) = &meta.expected.standing_output {
            assert_eq!(out["standing_output"], *so, "{}: standing_output mismatch", meta.case);
        }
    }
    assert_eq!(count, 4, "expected 4 GAP-3 frontier vectors");
}
