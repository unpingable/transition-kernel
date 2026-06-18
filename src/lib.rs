//! transition_kernel — the governed-transition office.
//!
//! Given a proposed transition plus its cited basis, decide
//! `Admit(AdmissionCandidate) | Refuse(kind, reasons) | Escalate(required_authority)`.
//!
//! Structural discipline (see NON_CLAIMS.md):
//! - The decision stage produces a **non-authority-bearing** [`AdmissionCandidate`]. The only object
//!   that can cross into execution is [`authority::AuthorizedTransition`], which is **unconstructable**
//!   until an LA capability is bound and execution-time revalidation passes (Stage 3, not A1).
//! - The kernel core ([`transition_core`]) answers **only its own clause**. It does not recompute
//!   Standing, Wicket, or LA semantics. The historical multi-field contract is reproduced by the
//!   [`legacy_corpus_adapter`], which feeds the core typed, already-issued office outputs.

pub mod transition_core;
pub mod legacy_corpus_adapter;
pub mod authority;
pub mod live;
pub mod memory_custody;
pub mod receiver_gate;
pub mod chain;
pub mod composed_snapshot;

pub use transition_core::{
    AdmissionCandidate, Escalation, RefusalKind, RefusingSeam, TransitionDecision,
};

/// The seven frozen verdict fields the legacy corpus contract pins
/// (`agent_governor.corpus.v1`). Produced by the [`legacy_corpus_adapter`]'s projection, **not** by
/// the kernel core directly — the core answers its own clause; the adapter projects history.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LegacyVerdict {
    pub outcome: String,
    pub refusal_kind: Option<String>,
    pub refusing_seam: Option<String>,
    pub effect_count: u32,
    pub consumed: bool,
    pub operational: bool,
    pub proposal_packet_present: bool,
}
