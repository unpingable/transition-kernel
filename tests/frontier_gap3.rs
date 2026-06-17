//! GAP-3 frontier/adversarial tests: typed-memory custody.
//!
//! These are the FRONTIER corpus, kept conceptually separate from the legacy conformance corpus: they
//! test newly specified semantics and intentionally refuse things the legacy Python path never governed
//! ("belief is free; inheritance is not"). They are NEVER grandfathered into `vectors/legacy/`.
//!
//! The load-bearing property: `remembered → relied_upon` is unconstructable except via `promote`, which
//! must survive every custody check. Each hostile vector names the closed refusal it must trigger.

use transition_kernel::memory_custody::{
    assess, promote, to_standing_output, Custody, EpistemicClass, Freshness, MemoryArtifact,
    MemoryRefusal, MemoryVerdict, PresentedClaim, PromotionReceipt, Presentation, UseLevel,
};
use transition_kernel::transition_core::StandingOutput;

const DIGEST: &str = "sha256:summary-abc";

fn artifact() -> MemoryArtifact {
    MemoryArtifact {
        custody: Custody {
            origin: "session-A".into(),
            recorder: "ag:outer".into(),
            subject: "disk_pressure".into(),
            scope: "ops/cleanup".into(),
            intended_consumer: "ag:main".into(),
            recorded_at: 100,
            supersedes: None,
        },
        epistemic_class: EpistemicClass::Inherited,
        granted_level: UseLevel::MayRemember,
        freshness: Freshness { valid_until: 1000, superseded: false },
        content_digest: DIGEST.into(),
    }
}

fn rely_receipt() -> PromotionReceipt {
    PromotionReceipt {
        artifact_digest: DIGEST.into(),
        subject: "disk_pressure".into(),
        to_level: UseLevel::MayRely,
        authorized_consumer: "ag:main".into(),
        issued_by: "operator-root".into(),
        valid_until: 1000,
    }
}

fn presenting(consumer: &str, now: u64, scope: &str) -> Presentation {
    Presentation { consumer: consumer.into(), now, scope: scope.into() }
}

// --- the decisive structural property -------------------------------------- //

#[test]
fn remembered_without_promotion_is_advisory_only_and_yields_no_standing() {
    // A bare remembered artifact can ONLY be presented as Advisory — there is no constructor that turns
    // it into a Relied claim. This is the "tiny bureaucratic coup" made impossible by construction.
    let presented = PresentedClaim::Advisory(artifact());
    match assess(&presented) {
        MemoryVerdict::AdvisoryOnly { .. } => {}
        other => panic!("expected advisory-only, got {other:?}"),
    }
    // And it confers no standing.
    assert_eq!(
        to_standing_output(&assess(&presented)),
        StandingOutput::Required,
        "advisory memory must not back a verified standing reference"
    );
}

// --- hostile vectors, each pinned to its closed refusal --------------------- //

#[test]
fn inherited_summary_relied_without_rely_promotion_refuses() {
    // A promotion that only authorizes quoting must not yield rely.
    let mut r = rely_receipt();
    r.to_level = UseLevel::MayQuote;
    let err = promote(&artifact(), &r, &presenting("ag:main", 200, "ops/cleanup")).unwrap_err();
    assert_eq!(err, MemoryRefusal::NoPromotionToRely);
}

#[test]
fn consumer_mismatch_refuses() {
    // Recorded for ag:main; a different consumer presents it. A's adoption is not B's standing.
    let err = promote(&artifact(), &rely_receipt(), &presenting("ag:other", 200, "ops/cleanup"))
        .unwrap_err();
    assert_eq!(err, MemoryRefusal::ConsumerMismatch);
}

#[test]
fn cross_session_transfer_into_wrong_scope_refuses() {
    // Inherited across a session boundary into a different scope.
    let err = promote(&artifact(), &rely_receipt(), &presenting("ag:main", 200, "ops/other"))
        .unwrap_err();
    assert_eq!(err, MemoryRefusal::ScopeMismatch);
}

#[test]
fn stale_at_presentation_refuses() {
    // Fresh at citation (recorded_at=100) but presented past the horizon (valid_until=1000).
    let err = promote(&artifact(), &rely_receipt(), &presenting("ag:main", 1001, "ops/cleanup"))
        .unwrap_err();
    assert_eq!(err, MemoryRefusal::StaleOrSuperseded);
}

#[test]
fn superseded_artifact_refuses() {
    let mut a = artifact();
    a.freshness.superseded = true;
    let err = promote(&a, &rely_receipt(), &presenting("ag:main", 200, "ops/cleanup")).unwrap_err();
    assert_eq!(err, MemoryRefusal::StaleOrSuperseded);
}

#[test]
fn promotion_for_a_different_artifact_does_not_cover() {
    let mut r = rely_receipt();
    r.artifact_digest = "sha256:some-other-thing".into();
    let err = promote(&artifact(), &r, &presenting("ag:main", 200, "ops/cleanup")).unwrap_err();
    assert_eq!(err, MemoryRefusal::PromotionDoesNotCover);
}

// --- the one legitimate path ------------------------------------------------ //

#[test]
fn legitimate_promotion_yields_standing_eligible() {
    let relied = promote(&artifact(), &rely_receipt(), &presenting("ag:main", 200, "ops/cleanup"))
        .expect("clean custody should promote");
    let verdict = assess(&PresentedClaim::Relied(relied));
    match &verdict {
        MemoryVerdict::StandingEligible { basis_digest, subject, consumer } => {
            assert_eq!(basis_digest, DIGEST);
            assert_eq!(subject, "disk_pressure");
            assert_eq!(consumer, "ag:main");
        }
        other => panic!("expected standing-eligible, got {other:?}"),
    }
    // Only now may it back a verified standing reference, and only by the exact promoted digest.
    assert_eq!(
        to_standing_output(&verdict),
        StandingOutput::Verified { receipt_ref: DIGEST.into() },
    );
}
