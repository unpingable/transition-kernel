//! Stage 3c: the composed execution snapshot — the receipt that pins the admission state it governed.
//!
//! The Stage 3 chain is the **first consumer that composes multiple offices' re-admissions into one
//! effect** (`ExecutionRevalidation` over standing → receiver correspondence → LA consume → effect). Each
//! office re-checks and receipts its *own* decision coherently, but until now the *composed* durable
//! receipt (`transition_enforce.py`: `consume_receipt` → `effect_attempt` → `effect_outcome`) recorded
//! only the LA decision and effect outcome — the `ExecutionRevalidation` facts that governed admission
//! were checked and then discarded. A verifier could confirm "capacity was consumed and an effect
//! happened" but not "the effect happened under exactly *this* composed admission snapshot."
//!
//! [`ComposedExecutionSnapshot`] closes that gap by porting `standing`'s gold-standard pattern (a receipt
//! built from the exact evaluated snapshot) to the composed seam: it captures the verified opaque
//! `eligibility_reference`, the execution-clock revalidation facts (`revalidated_at` / `valid_until` /
//! `live`), the capability nonce, and the bound operation, and reduces them to one canonical
//! [`ComposedExecutionSnapshot::snapshot_hash`] (RFC 8785 JCS + SHA-256). The orchestrator records this
//! snapshot durably *before* the consume, and the consume receipt references the snapshot hash — so a
//! replay reconstructs the identical hash and a verifier can prove same-snapshot coherence end to end.
//!
//! Fence (`NON_CORRESPONDENCE.md`): this is the operational gate. No Lean theorem is *stated* here — the
//! Lean `lossy_receipt_cannot_pin_snapshot` scratch model is upstream evidence that the carry
//! (`revalidated_at` + `valid_until`, not just the eligibility reference) is load-bearing; this deployed
//! gate is the forcing consumer that a downstream theorem would later justify.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::authority::AuthorizedTransition;
use crate::transition_core::AdmissionCandidate;

/// Schema tag for the snapshot's canonical form. Bump only on a wire-format change; it is part of the
/// hashed body, so a tag change is a hash change (old receipts do not silently validate against new code).
pub const SNAPSHOT_SCHEMA: &str = "transition_kernel.composed_snapshot.v1";

/// Schema tag for the admission-candidate basis hash (the relied-upon opaque references).
const CANDIDATE_BASIS_SCHEMA: &str = "transition_kernel.admission_candidate_basis.v1";

/// The composed admission snapshot under which a single bounded effect was allowed.
///
/// Every field is a fact the kernel **verified** on the path to constructing the [`AuthorizedTransition`]
/// (never re-derived from content; Standing digests are non-reproducible). The struct carries no hash
/// field — the hash is computed *over* these fields by [`Self::snapshot_hash`], so it cannot disagree
/// with what it commits to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedExecutionSnapshot {
    /// Canonical-form schema tag (see [`SNAPSHOT_SCHEMA`]). Part of the hashed body.
    pub schema: String,
    /// The transition-kernel version that produced the snapshot.
    pub kernel_version: String,

    /// The exact operation the authorization is for.
    pub operation_hash: String,
    /// Hash of the admitted candidate's relied-upon basis (standing_ref ⋄ scope ⋄ target). Binds the
    /// snapshot to the *admission* decision without recomputing any opaque Standing digest.
    pub admission_candidate_hash: String,
    /// The verified opaque Standing receipt reference the candidate relied on and the capability was
    /// minted against (anti-recombination: equal by construction at `finalize`).
    pub eligibility_reference: String,

    /// Was Standing live at the execution clock? Always `true` for a constructed snapshot — an
    /// [`crate::authority::ExecutionRevalidation`] cannot exist otherwise — recorded explicitly so the
    /// receipt *states* the fact it relied on rather than leaving it implicit.
    pub revalidation_live: bool,
    /// The freshness horizon the re-admission was checked against.
    pub revalidation_valid_until: u64,
    /// The execution clock at which the re-admission was checked.
    pub revalidated_at: u64,

    /// The single-use capability nonce burned for this effect.
    pub capability_nonce: String,
    /// The LA consumption event id this snapshot governs (derived from operation + capability).
    pub consumption_event_id: String,

    pub consumer: String,
    pub scope: String,
    pub target: String,
    pub effect_class: String,
}

/// Why a snapshot failed to cohere with the durable records that claim it. Each is a way the composed
/// receipt would describe a *different* admission state than the one that governed the effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotIncoherence {
    /// The recomputed canonical hash does not equal the claimed `snapshot_hash`.
    HashMismatch,
    /// The consume record references a different eligibility reference than the snapshot pins.
    EligibilityReferenceMismatch,
    /// The consume record references a different capability nonce than the snapshot pins.
    CapabilityNonceMismatch,
    /// The consume record references a different consumption event than the snapshot pins.
    ConsumptionEventMismatch,
    /// The snapshot claims liveness but carries no execution clock — a lossy receipt that cannot
    /// distinguish two different execution snapshots over the same lineage (the case the Lean model
    /// `lossy_receipt_cannot_pin_snapshot` rules out).
    MissingRevalidationClock,
}

impl SnapshotIncoherence {
    pub fn as_str(self) -> &'static str {
        match self {
            SnapshotIncoherence::HashMismatch => "snapshot_hash_mismatch",
            SnapshotIncoherence::EligibilityReferenceMismatch => "eligibility_reference_mismatch",
            SnapshotIncoherence::CapabilityNonceMismatch => "capability_nonce_mismatch",
            SnapshotIncoherence::ConsumptionEventMismatch => "consumption_event_mismatch",
            SnapshotIncoherence::MissingRevalidationClock => "missing_revalidation_clock",
        }
    }
}

impl ComposedExecutionSnapshot {
    /// Build the snapshot from the verified terminal authorization. Every field comes from the
    /// [`AuthorizedTransition`] (whose constructor already enforced the anti-recombination binding) plus
    /// the LA `consumption_event_id` the correspondence stage derived; nothing is re-derived from content.
    pub fn from_authorized(
        authorized: &AuthorizedTransition,
        consumption_event_id: String,
    ) -> Self {
        let cap = authorized.capability();
        let op = authorized.operation();
        let reval = authorized.revalidation();
        Self {
            schema: SNAPSHOT_SCHEMA.to_string(),
            kernel_version: env!("CARGO_PKG_VERSION").to_string(),
            operation_hash: op.operation_hash.clone(),
            admission_candidate_hash: candidate_basis_hash(authorized.candidate()),
            eligibility_reference: cap.eligibility_reference.clone(),
            // A constructed ExecutionRevalidation witnesses liveness at `revalidated_at`.
            revalidation_live: true,
            revalidation_valid_until: reval.valid_until(),
            revalidated_at: reval.revalidated_at(),
            capability_nonce: cap.capability_id.clone(),
            consumption_event_id,
            consumer: op.consumer.clone(),
            scope: cap.scope.clone(),
            target: cap.target.clone(),
            effect_class: cap.effect_class.clone(),
        }
    }

    /// The canonical content hash of the snapshot: RFC 8785 (JCS) canonicalization → SHA-256, prefixed
    /// `sha256:`. Deterministic and field-order-independent, so a replay over the same durable records
    /// reconstructs the identical hash.
    pub fn snapshot_hash(&self) -> String {
        let canonical = serde_jcs::to_vec(self).expect("snapshot is plain serializable data");
        let mut hasher = Sha256::new();
        hasher.update(&canonical);
        format!("sha256:{:x}", hasher.finalize())
    }

    /// Verify that a durable consume record coheres with this snapshot: the claimed hash recomputes, the
    /// snapshot carries its execution clock, and the consume's references match what the snapshot pins.
    /// This is the Rust statement of the chain invariant the orchestrator's reconstruction mirrors.
    pub fn verify_consume_binding(
        &self,
        claimed_hash: &str,
        consume_eligibility_reference: &str,
        consume_capability_nonce: &str,
        consume_consumption_event_id: &str,
    ) -> Result<(), SnapshotIncoherence> {
        // A live snapshot with no execution clock is lossy: it cannot distinguish two execution snapshots
        // over the same lineage. (from_authorized never produces this; a hand-rolled receipt could.)
        if self.revalidation_live && self.revalidated_at == 0 {
            return Err(SnapshotIncoherence::MissingRevalidationClock);
        }
        if self.snapshot_hash() != claimed_hash {
            return Err(SnapshotIncoherence::HashMismatch);
        }
        if self.eligibility_reference != consume_eligibility_reference {
            return Err(SnapshotIncoherence::EligibilityReferenceMismatch);
        }
        if self.capability_nonce != consume_capability_nonce {
            return Err(SnapshotIncoherence::CapabilityNonceMismatch);
        }
        if self.consumption_event_id != consume_consumption_event_id {
            return Err(SnapshotIncoherence::ConsumptionEventMismatch);
        }
        Ok(())
    }
}

/// Hash the admitted candidate's relied-upon basis — the opaque references it admitted on, never
/// recomputed from content. Binds a snapshot to the admission decision without reproducing any Standing
/// digest. Canonicalized (JCS) so it is stable across field order.
pub fn candidate_basis_hash(candidate: &AdmissionCandidate) -> String {
    let basis = serde_json::json!({
        "schema": CANDIDATE_BASIS_SCHEMA,
        "standing_ref": candidate.standing_ref,
        "scope": candidate.scope,
        "target": candidate.target,
    });
    let canonical = serde_jcs::to_vec(&basis).expect("basis is plain serializable data");
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    format!("sha256:{:x}", hasher.finalize())
}
