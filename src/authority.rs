//! The authority-bearing terminal object and the final binding that constructs it.
//!
//! `AuthorizedTransition` is the only object that may cross into execution (receiver gate → actuator).
//! Its sole constructor, [`AuthorizedTransition::finalize`], requires an [`AdmissionCandidate`], an
//! LA-issued [`LaCapability`], a passed [`ExecutionRevalidation`], and an [`OperationContext`] — and it
//! enforces the **anti-recombination binding**: the capability must have been minted against the exact
//! opaque Standing reference the candidate relied on, with matching scope and target. There is no
//! `From`/coercion path; a candidate cannot become an authority by name.
//!
//! `SpendCapability` itself lives in LinearAccountant (the minting authority). Here we hold only its wire
//! mirror — transition-kernel consumes a capability, it never mints one.

use crate::transition_core::AdmissionCandidate;

/// Wire mirror of LinearAccountant's `SpendCapability` (the bounded execution key). We receive it; we
/// never construct authority from nothing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct LaCapability {
    pub capability_id: String,
    pub scope: String,
    pub target: String,
    pub effect_class: String,
    pub eligibility_reference: String,
    pub expires_at: u64,
    pub single_use: bool,
}

/// The exact operation an authorization is for: an operation hash + the consumer that may present it.
/// Bound at finalize so the receiver gate can check the requested effect matches what was authorized.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct OperationContext {
    pub operation_hash: String,
    pub consumer: String,
}

/// Proof that Standing/custody/freshness were re-checked at the **execution** clock (not inherited from
/// citation). Sealed: constructable only by [`ExecutionRevalidation::revalidate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRevalidation {
    standing_ref: String,
    revalidated_at: u64,
    _seal: (),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevalError {
    StandingNotLive,
    StaleAtExecution,
}

impl ExecutionRevalidation {
    /// Re-check at execution time. `live` is whether Standing is still live (not revoked/superseded) and
    /// `valid_until` its freshness horizon, both evaluated against the execution clock `now`.
    pub fn revalidate(
        standing_ref: &str,
        live: bool,
        valid_until: u64,
        now: u64,
    ) -> Result<Self, RevalError> {
        if !live {
            return Err(RevalError::StandingNotLive);
        }
        if now > valid_until {
            return Err(RevalError::StaleAtExecution);
        }
        Ok(Self { standing_ref: standing_ref.to_string(), revalidated_at: now, _seal: () })
    }

    pub fn standing_ref(&self) -> &str {
        &self.standing_ref
    }
}

/// Why the final binding refused. Each is a way the lineage failed to cohere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindError {
    /// The capability was minted against a different Standing reference than the candidate relied on.
    EligibilityMismatch,
    ScopeMismatch,
    TargetMismatch,
    /// The revalidation was for a different Standing reference than the candidate's.
    RevalidationSubjectMismatch,
    CapabilityNotSingleUse,
}

/// The terminal, authority-bearing object. Constructed only by [`Self::finalize`].
#[derive(Debug, Clone)]
pub struct AuthorizedTransition {
    candidate: AdmissionCandidate,
    capability: LaCapability,
    revalidation: ExecutionRevalidation,
    operation: OperationContext,
}

impl AuthorizedTransition {
    /// The single path from candidate to authority. Enforces the anti-recombination binding: the
    /// capability's `eligibility_reference` must equal the exact Standing reference the candidate relied
    /// on (never recomputed), scope/target must match, the revalidation must be for the same Standing,
    /// and the capability must be single-use.
    pub fn finalize(
        candidate: AdmissionCandidate,
        capability: LaCapability,
        revalidation: ExecutionRevalidation,
        operation: OperationContext,
    ) -> Result<Self, BindError> {
        if capability.eligibility_reference != candidate.standing_ref {
            return Err(BindError::EligibilityMismatch);
        }
        if capability.scope != candidate.scope {
            return Err(BindError::ScopeMismatch);
        }
        if capability.target != candidate.target {
            return Err(BindError::TargetMismatch);
        }
        if revalidation.standing_ref != candidate.standing_ref {
            return Err(BindError::RevalidationSubjectMismatch);
        }
        if !capability.single_use {
            return Err(BindError::CapabilityNotSingleUse);
        }
        Ok(Self { candidate, capability, revalidation, operation })
    }

    pub fn candidate(&self) -> &AdmissionCandidate {
        &self.candidate
    }
    pub fn capability(&self) -> &LaCapability {
        &self.capability
    }
    pub fn operation(&self) -> &OperationContext {
        &self.operation
    }
    pub fn revalidation(&self) -> &ExecutionRevalidation {
        &self.revalidation
    }
}
