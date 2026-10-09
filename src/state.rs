use crate::{
    ChainEvent, CheckError, CheckedProof, Headers, Height, Hint, HintStatus, Proof, Timestamp, TxId,
};
use std::sync::Arc;

/// Inclusion backed by an immutable checked capability; height is derived.
///
/// ```compile_fail
/// use bsv_tracker::Word;
/// let _ = Word::Mined { height: 100 };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mined {
    checked: Arc<CheckedProof>,
}
impl Mined {
    /// The height of the checked proof. It cannot differ from the evidence.
    pub fn height(&self) -> Height {
        self.checked.height()
    }
    /// The checked capability behind the word.
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
    /// `Tracker.Word.chain`, ignoring the host and hint tiers.
    pub fn chain(&self) -> Option<&Self> {
        match self {
            Self::Mined(_) | Self::Stale | Self::Rejected { .. } | Self::Conflicted { .. } => {
                Some(self)
            }
            _ => None,
        }
    }
    /// A height only when this is an evidenced mined word.
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
    /// Birth with no chain claim or stored evidence, as in `Tracker.State.init`.
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
    /// The host's tracked transaction.
    pub fn txid(&self) -> &str {
        &self.txid
    }
    /// The current word. It is immutable outside the transition function.
    pub fn word(&self) -> &Word {
        &self.word
    }
    /// The retained record, including when the word has become stale.
    pub fn evidence(&self) -> Option<&CheckedProof> {
        self.evidence.as_deref()
    }
    /// Whether an unresolved fork requires a header recheck before acting.
    pub fn suspect(&self) -> bool {
        self.suspect
    }
    /// Observations in newest received order, matching the Lean list.
    pub fn hints(&self) -> &[Hint] {
        &self.hints
    }
    /// Time of the most recently received hint, even for out of order clocks.
    pub fn last_hint_at(&self) -> Option<Timestamp> {
        self.last_hint_at
    }
    /// The next re-ask, which the host must execute and feed back.
    pub fn reask(&self) -> Option<&Reask> {
        self.reask.as_ref()
    }
    /// Execute one Lean transition. Proof and header failures retain the word
    /// and record the re-ask before returning an error to the host.
    pub fn step<H: Headers + ?Sized>(
        &mut self,
        params: &Params,
        headers: &H,
        input: Input,
    ) -> Result<(), CheckError> {
        match input {
            Input::Hint(hint) => self.on_hint(hint),
            Input::Host(action) => self.on_host(params, action),
            Input::Evidence(Evidence::Proof(proof)) => self.on_proof(headers, proof)?,
            Input::Evidence(Evidence::Competitor(proof)) => self.on_competitor(headers, proof)?,
            Input::Evidence(Evidence::Chain(event)) => self.on_chain(headers, &event)?,
            Input::Evidence(Evidence::Verdict(verdict)) => self.on_verdict(verdict),
            Input::Evidence(Evidence::Recheck) => self.recheck(headers)?,
        }
        Ok(())
    }

    fn on_hint(&mut self, hint: Hint) {
        match (&self.word, &hint.status) {
            (Word::Built, HintStatus::Accepted) => self.word = Word::Announced,
            (Word::Built | Word::Announced, HintStatus::Seen) => self.word = Word::Seen,
            _ => {}
        }
        if !agrees(&self.word, &hint.status) {
            self.reask = Some(Reask::Hint(hint.clone()));
        }
        self.last_hint_at = Some(hint.observed);
        self.hints.insert(0, hint);
    }

    fn on_host(&mut self, params: &Params, action: HostAction) {
        match action {
            HostAction::Build if self.word == Word::Unknown => self.word = Word::Built,
            HostAction::Abandon if matches!(self.word, Word::Unknown | Word::Built) => {
                self.word = Word::Abandoned
            }
            HostAction::SpendAttempt => self.reask = Some(Reask::Spend),
            // Lean adds unbounded naturals. Checked addition keeps an
            // overflowing deadline from becoming prematurely due.
            HostAction::Tick(now)
                if matches!(self.word, Word::Announced | Word::Seen)
                    && self
                        .last_hint_at
                        .and_then(|t| t.checked_add(params.age_threshold))
                        .is_some_and(|deadline| deadline <= now) =>
            {
                self.reask = Some(Reask::Age);
            }
            _ => {}
        }
    }

    fn on_proof<H: Headers + ?Sized>(
        &mut self,
        headers: &H,
        proof: Proof,
    ) -> Result<(), CheckError> {
        let height = proof.height();
        if proof.txid() != self.txid {
            self.reask = Some(Reask::ProofFailed { height });
            return Err(CheckError::TxidMismatch);
        }
        match crate::evidence::check_snapshot(headers, proof) {
            Ok(checked) => {
                let checked = Arc::new(checked);
                self.word = Word::Mined(Mined {
                    checked: Arc::clone(&checked),
                });
                self.evidence = Some(checked);
                self.suspect = false;
                self.reask = None;
                Ok(())
            }
            Err(error) => {
                self.reask = Some(Reask::ProofFailed { height });
                Err(error)
            }
        }
    }

    fn on_competitor<H: Headers + ?Sized>(
        &mut self,
        headers: &H,
        proof: Proof,
    ) -> Result<(), CheckError> {
        let height = proof.height();
        let competitor = proof.txid().to_owned();
        match crate::evidence::check_snapshot(headers, proof) {
            Ok(_) => {
                match &mut self.word {
                    Word::Mined(_) => self.reask = Some(Reask::Recheck),
                    Word::Conflicted { competitors } => competitors.insert(0, competitor),
                    _ => {
                        self.word = Word::Conflicted {
                            competitors: vec![competitor],
                        }
                    }
                }
                Ok(())
            }
            Err(error) => {
                self.reask = Some(Reask::ProofFailed { height });
                Err(error)
            }
        }
    }

    fn recheck<H: Headers + ?Sized>(&mut self, headers: &H) -> Result<(), CheckError> {
        let Word::Mined(mined) = &self.word else {
            return Ok(());
        };
        let height = mined.height();
        let answer = headers.header_at(height);
        match answer {
            Ok(Some(header)) if header.merkle_root == mined.checked().root() => {
                self.suspect = false;
                self.reask = None;
            }
            Ok(Some(_)) => self.stale(height),
            Ok(None) => {
                self.reask = Some(Reask::Recheck);
                return Err(CheckError::Unavailable(height));
            }
            Err(error) => {
                self.reask = Some(Reask::Recheck);
                return Err(CheckError::Headers(error));
            }
        }
        Ok(())
    }

    fn stale(&mut self, fork_height: Height) {
        self.word = Word::Stale;
        self.suspect = false;
        self.reask = Some(Reask::Reorg { fork_height });
    }

    pub(crate) fn on_chain<H: Headers + ?Sized>(
        &mut self,
        headers: &H,
        event: &ChainEvent,
    ) -> Result<(), CheckError> {
        match event {
            ChainEvent::Tip { .. } if self.suspect => self.recheck(headers)?,
            ChainEvent::Fork { height, .. } if self.word.height().is_some_and(|h| *height <= h) => {
                self.suspect = true;
                self.reask = Some(Reask::Fork { height: *height });
            }
            ChainEvent::Reorg { fork_height, .. }
                if self.word.height().is_some_and(|h| *fork_height <= h) =>
            {
                self.stale(*fork_height)
            }
            ChainEvent::Invalidated { block_hash } => {
                if let Word::Mined(mined) = &self.word {
                    if &mined.checked().header().hash == block_hash {
                        self.stale(mined.height());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_verdict(&mut self, verdict: NodeVerdict) {
        if matches!(self.word, Word::Mined(_)) {
            self.reask = Some(Reask::Recheck);
            return;
        }
        self.word = match verdict {
            NodeVerdict::Rejected { reason } => Word::Rejected { reason },
            NodeVerdict::Conflicted { mut competitors } => {
                if let Word::Conflicted {
                    competitors: previous,
                } = &self.word
                {
                    competitors.extend(previous.iter().cloned());
                }
                Word::Conflicted { competitors }
            }
        };
    }
}

fn agrees(word: &Word, status: &HintStatus) -> bool {
    match (word, status) {
        (Word::Announced | Word::Seen, HintStatus::Accepted)
        | (Word::Seen, HintStatus::Seen)
        | (Word::Stale, HintStatus::StaleBlock)
        | (Word::Rejected { .. }, HintStatus::Rejected { .. })
        | (Word::Conflicted { .. }, HintStatus::DoubleSpend { .. }) => true,
        (Word::Mined(mined), HintStatus::Mined { height }) => mined.height() == *height,
        _ => false,
    }
}
