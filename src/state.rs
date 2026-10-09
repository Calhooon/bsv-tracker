use crate::{ChainEvent, CheckError, CheckedProof, Headers, Height, Hint, Proof, Timestamp, TxId};
use std::sync::Arc;

/// Inclusion backed by an immutable checked capability; height is derived.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mined {
    checked: Arc<CheckedProof>,
}
impl Mined {
    pub fn height(&self) -> Height {
        self.checked.height()
    }
    pub fn checked(&self) -> &CheckedProof {
        &self.checked
    }
}
/// The nine words. `Mined` needs an unforgeable checked proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Word {
    Unknown,
    Built,
    Announced,
    Seen,
    Mined(Mined),
    Stale,
    Rejected { reason: String },
    Conflicted { competitors: Vec<TxId> },
    Abandoned,
}
impl Word {
    pub fn chain(&self) -> Option<&Self> {
        match self {
            Self::Mined(_) | Self::Stale | Self::Rejected { .. } | Self::Conflicted { .. } => {
                Some(self)
            }
            _ => None,
        }
    }
    pub fn height(&self) -> Option<Height> {
        if let Self::Mined(m) = self {
            Some(m.height())
        } else {
            None
        }
    }
}
/// A verdict explicitly supplied by the host's trusted node, never a broadcaster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeVerdict {
    Rejected { reason: String },
    Conflicted { competitors: Vec<TxId> },
}
/// The inputs that may write a chain word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Evidence {
    Proof(Proof),
    Competitor(Proof),
    Chain(ChainEvent),
    Verdict(NodeVerdict),
    Recheck,
}
/// Acts supplied by the host, including its clock and spend guard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostAction {
    Build,
    Abandon,
    SpendAttempt,
    Tick(Timestamp),
}
/// One input to `Tracker.step`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Evidence(Evidence),
    Hint(Hint),
    Host(HostAction),
}
/// A value for the host to execute; the crate never fetches anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reask {
    Reorg { fork_height: Height },
    Fork { height: Height },
    Hint(Hint),
    ProofFailed { height: Height },
    Recheck,
    Age,
    Spend,
}
/// Host selected scheduling parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Params {
    pub age_threshold: Timestamp,
}
/// The complete private state of one transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    txid: TxId,
    word: Word,
    evidence: Option<Arc<CheckedProof>>,
    suspect: bool,
    hints: Vec<Hint>,
    last_hint_at: Option<Timestamp>,
    reask: Option<Reask>,
}
impl State {
    pub fn new(txid: impl Into<TxId>) -> Self {
        Self {
            txid: txid.into().to_ascii_lowercase(),
            word: Word::Unknown,
            evidence: None,
            suspect: false,
            hints: vec![],
            last_hint_at: None,
            reask: None,
        }
    }
    pub fn txid(&self) -> &str {
        &self.txid
    }
    pub fn word(&self) -> &Word {
        &self.word
    }
    pub fn evidence(&self) -> Option<&CheckedProof> {
        self.evidence.as_deref()
    }
    pub fn suspect(&self) -> bool {
        self.suspect
    }
    pub fn hints(&self) -> &[Hint] {
        &self.hints
    }
    pub fn last_hint_at(&self) -> Option<Timestamp> {
        self.last_hint_at
    }
    pub fn reask(&self) -> Option<&Reask> {
        self.reask.as_ref()
    }
    pub fn step<H: Headers + ?Sized>(
        &mut self,
        _params: &Params,
        _headers: &H,
        _input: Input,
    ) -> Result<(), CheckError> {
        // The witness commit deliberately lacks the transition machine.
        Ok(())
    }
}
