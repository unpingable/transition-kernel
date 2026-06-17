//! The kernel core: typed, already-issued office outputs → [`TransitionDecision`].
//!
//! This module answers **only the transition office's own clause**. It does *not* recompute Standing,
//! Wicket, or LA semantics — those arrive as already-decided office outputs (supplied in A1 by the
//! [`crate::legacy_corpus_adapter`], later by live office calls). The core's job is the composition:
//! which seam refuses first, whether a verified read-verdict and write-standing jointly admit, and what
//! authority an escalation requires.
//!
//! NOTE: the closed vocabularies below are seeded from `golden/README.md` +
//! `linear_accountant_client.py:109` and are being confirmed verbatim against `drill_runner.py`
//! (extraction in flight). Treat variant spelling as load-bearing wire values.

use std::fmt;

/// Read-side proof: the cited basis was authorized to be *read/relied upon* (Standing verified + not
/// expired, admission read). Constructed only via a checked path inside this crate. There is
/// deliberately **no** `From<WriteStanding>` — read-standing cannot become write-standing for free
/// (Lean `NoFreeStandingBridge`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadVerdict {
    _seal: (),
}

/// Write-side proof: the proposed step is *allowed to mutate* (Lean `StepAllowed`). Constructed only via
/// a checked path inside this crate. Distinct, non-coercible from [`ReadVerdict`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteStanding {
    _seal: (),
}

/// A non-authority-bearing admission candidate. Carries the two distinct proofs (read ⟂ write) plus the
/// opaque references it relied on. It is **not** an authorization: see [`crate::authority`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionCandidate {
    read: ReadVerdict,
    write: WriteStanding,
    /// The exact verified opaque Standing receipt reference relied upon (never recomputed from content;
    /// Standing digests are non-reproducible).
    pub standing_ref: String,
    pub scope: String,
    pub target: String,
}

impl AdmissionCandidate {
    /// The single constructor. Requires both proofs by value — an admission candidate cannot be formed
    /// from one proof, a default, or a bare assertion.
    pub(crate) fn new(
        read: ReadVerdict,
        write: WriteStanding,
        standing_ref: String,
        scope: String,
        target: String,
    ) -> Self {
        Self { read, write, standing_ref, scope, target }
    }

    pub fn read(&self) -> &ReadVerdict {
        &self.read
    }
    pub fn write(&self) -> &WriteStanding {
        &self.write
    }
}

/// The closed refusal taxonomy (12 kinds), owned with `linear_accountant_client.CLOSED_REFUSAL_KINDS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalKind {
    StandingRequired,
    StandingExpired,
    AdmissionDenied,
    AdmissionGapAccounted,
    CapacityRefused,
    AlreadyConsumed,
    DanglingReceiptReference,
    TokenExpired,
    TokenRevoked,
    UnknownToken,
    ScopeMismatch,
    StandingBeforeSpendabilityNotBounded,
}

impl RefusalKind {
    pub fn as_str(self) -> &'static str {
        use RefusalKind::*;
        match self {
            StandingRequired => "standing_required",
            StandingExpired => "standing_expired",
            AdmissionDenied => "admission_denied",
            AdmissionGapAccounted => "admission_gap_accounted",
            CapacityRefused => "capacity_refused",
            AlreadyConsumed => "already_consumed",
            DanglingReceiptReference => "dangling_receipt_reference",
            TokenExpired => "token_expired",
            TokenRevoked => "token_revoked",
            UnknownToken => "unknown_token",
            ScopeMismatch => "scope_mismatch",
            StandingBeforeSpendabilityNotBounded => "standing_before_spendability_not_bounded",
        }
    }
}

impl fmt::Display for RefusalKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The seam that refused (where the chain stopped). Closed set, confirmed against `drill_runner.py`.
/// `WicketSeam` and `ProposalValidatorSeam` are in the closed vocabulary but the *core* never emits
/// them: `WicketSeam` appears only on `gap_accounted` (a non-refusal presentation label, owned by the
/// adapter), and `ProposalValidatorSeam` only on D3 confabulation (outside the 9-case contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusingSeam {
    StandingSeam,
    StandingSpendabilitySeam,
    LaSeam,
    WicketSeam,
    ProposalValidatorSeam,
}

impl RefusingSeam {
    pub fn as_str(self) -> &'static str {
        match self {
            RefusingSeam::StandingSeam => "standing_seam",
            RefusingSeam::StandingSpendabilitySeam => "standing_spendability_seam",
            RefusingSeam::LaSeam => "la_seam",
            RefusingSeam::WicketSeam => "wicket_seam",
            RefusingSeam::ProposalValidatorSeam => "proposal_validator_seam",
        }
    }
}

impl fmt::Display for RefusingSeam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An escalation carries the authority that *would* be required — a typed obligation, not a default-open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escalation {
    pub required_authority: String,
    pub reasons: Vec<String>,
}

/// The office's own clause. `Admit` yields a *candidate*, never an authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionDecision {
    Admit(AdmissionCandidate),
    Refuse {
        kind: RefusalKind,
        seam: RefusingSeam,
        reasons: Vec<String>,
    },
    Escalate(Escalation),
}

// ---------------------------------------------------------------------------
// Office outputs — the INPUTS to the kernel's clause.
//
// These are already-decided results from peer offices (Standing, Wicket, LA, the standing-spendability
// gate). The core does NOT recompute them; it composes them. In A1 they are supplied by the
// `legacy_corpus_adapter` as scenario fixtures; later, by live office calls.
// ---------------------------------------------------------------------------

/// What the Standing office returned for the cited basis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandingOutput {
    /// Verified live; carries the exact opaque receipt reference (never recomputed from content).
    Verified { receipt_ref: String },
    /// No standing was presented / mintable.
    Required,
    /// A standing reference was presented but no longer bears authority (lapsed/unverifiable).
    Expired,
}

/// The standing-spendability (two-clock) gate, present only when a standing window applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpendabilityOutput {
    NotApplicable,
    Bounded,
    /// The gap exceeded the bound; carries the opaque two-clock block for the receipt.
    Unbounded { block: serde_json::Value },
}

/// The LA admission/request decision (the admission verifier at `request_capacity`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionOutput {
    Admitted,
    Denied,
    NotReached,
}

/// The LA consume decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumeOutput {
    Consumed,
    AlreadyConsumed,
    NotReached,
}

/// The bundle of already-issued office outputs the kernel composes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficeOutputs {
    pub standing: StandingOutput,
    pub spendability: SpendabilityOutput,
    pub la_admission: AdmissionOutput,
    pub la_consume: ConsumeOutput,
    pub scope: String,
    pub target: String,
}

/// The kernel's clause: compose office outputs into a decision by **seam precedence**, short-circuiting
/// on the first refusal. This is genuine composition logic, not a per-scenario lookup table.
///
/// Order (matches `cooked_context_orchestrator._run_chain`): standing (checked inside wicket) →
/// standing-spendability gate → LA request/admission → LA consume → admit.
pub fn decide(office: &OfficeOutputs) -> TransitionDecision {
    // 1. Standing — checked first (inside wicket). A standing failure refuses before anything downstream.
    match &office.standing {
        StandingOutput::Required => {
            return refuse(RefusalKind::StandingRequired, RefusingSeam::StandingSeam,
                "no standing presented for the cited basis");
        }
        StandingOutput::Expired => {
            return refuse(RefusalKind::StandingExpired, RefusingSeam::StandingSeam,
                "presented standing reference no longer bears authority");
        }
        StandingOutput::Verified { .. } => {}
    }

    // 2. Standing-spendability gate — fires after standing/wicket admit, BEFORE any capacity is reserved
    //    (a lapse must cost no budget).
    if let SpendabilityOutput::Unbounded { .. } = office.spendability {
        return refuse(
            RefusalKind::StandingBeforeSpendabilityNotBounded,
            RefusingSeam::StandingSpendabilitySeam,
            "standing observed before spendability window; gap exceeded the bound",
        );
    }

    // 3. LA request / admission.
    if let AdmissionOutput::Denied = office.la_admission {
        return refuse(RefusalKind::AdmissionDenied, RefusingSeam::LaSeam,
            "LA admission verifier denied the request");
    }

    // 4. LA consume.
    if let ConsumeOutput::AlreadyConsumed = office.la_consume {
        return refuse(RefusalKind::AlreadyConsumed, RefusingSeam::LaSeam,
            "consumption_event_id already consumed (replay)");
    }

    // 5. Admit — yields a *candidate*, never an authority. Both proofs are now established: the read
    //    side (standing verified, admission ok) and the write side (no seam refused the mutation).
    let receipt_ref = match &office.standing {
        StandingOutput::Verified { receipt_ref } => receipt_ref.clone(),
        _ => unreachable!("non-verified standing returns above"),
    };
    let read = ReadVerdict { _seal: () };
    let write = WriteStanding { _seal: () };
    TransitionDecision::Admit(AdmissionCandidate::new(
        read, write, receipt_ref, office.scope.clone(), office.target.clone(),
    ))
}

fn refuse(kind: RefusalKind, seam: RefusingSeam, reason: &str) -> TransitionDecision {
    TransitionDecision::Refuse { kind, seam, reasons: vec![reason.to_string()] }
}
