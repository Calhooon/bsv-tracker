use crate::evidence::Len;
use crate::{
    ChainEvent, CheckError, CheckedProof, CompactError, Headers, Height, Hint, HintStatus, Proof,
    Timestamp, TxId, AMBIGUOUS_TX_LEN,
};
use serde::{Deserialize, Serialize};
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
    #[must_use]
    pub fn height(&self) -> Height {
        self.checked.height()
    }
    /// The checked capability behind the word.
    #[must_use]
    pub fn checked(&self) -> &CheckedProof {
        &self.checked
    }
}
/// The ten words, mirrored from `Tracker.Word` (bsv-stack-lean
/// `lean/Tracker.lean`). `Mined` needs an unforgeable checked proof.
///
/// Three tiers: the host's (`Unknown`, `Built`, `Handed`, `Abandoned`,
/// written by the host's own acts), the hint tier (`Announced`, `Seen`,
/// written by a broadcaster's word and claiming nothing of the chain) and the
/// chain tier (`Mined`, `Stale`, `Rejected`, `Conflicted`, written only on
/// evidence).
///
/// `Word` is not `#[non_exhaustive]`, on purpose: a word is a decision each
/// host makes about what it may do, so a new word breaks every exhaustive
/// match on it outside this crate until the host names it. `Handed`, the
/// tenth, did so: a match over the nine words of 0.9.0-beta.11 no longer
/// compiles.
///
/// ```compile_fail,E0004
/// use bsv_tracker::Word;
/// fn label(word: &Word) -> &'static str {
///     match word {
///         Word::Unknown => "unknown",
///         Word::Built => "built",
///         Word::Announced => "announced",
///         Word::Seen => "seen",
///         Word::Mined(_) => "mined",
///         Word::Stale => "stale",
///         Word::Rejected { .. } => "rejected",
///         Word::Conflicted { .. } => "conflicted",
///         Word::Abandoned => "abandoned",
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Word {
    /// Tracked, with no claim from any tier.
    Unknown,
    /// The host built the transaction.
    Built,
    /// The bytes left the host's hands for one or more named instances and
    /// none has answered: written by the host's own act
    /// [`HostAction::HandOver`] before the post leaves, never at a timeout. It
    /// claims nothing about the chain and nothing about any instance; the
    /// instances stand beside it in [`State::handed_to`]. A host that feeds
    /// no hand-over never sees it. `Tracker.Word.handed`.
    Handed,
    /// A broadcaster accepted the transaction.
    Announced,
    /// A broadcaster reports the transaction seen on the network.
    Seen,
    /// Included, backed by a checked proof against the active headers.
    Mined(Mined),
    /// Was mined in a block that left the active chain.
    Stale,
    /// The host's trusted node rejected the transaction.
    Rejected {
        /// The node's reason.
        reason: String,
    },
    /// A competing transaction is mined or named by the host's trusted node.
    Conflicted {
        /// The competing txids, the most recent at the front.
        competitors: Vec<TxId>,
    },
    /// The host abandoned the transaction before any broadcaster accepted it.
    Abandoned,
}
impl Word {
    /// `Tracker.Word.chain`, ignoring the host and hint tiers.
    #[must_use]
    pub fn chain(&self) -> Option<&Self> {
        match self {
            Self::Mined(_) | Self::Stale | Self::Rejected { .. } | Self::Conflicted { .. } => {
                Some(self)
            }
            _ => None,
        }
    }
    /// A height only when this is an evidenced mined word.
    #[must_use]
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
    /// The node rejected the transaction.
    Rejected {
        /// The node's reason.
        reason: String,
    },
    /// The node names competing transactions.
    Conflicted {
        /// The competing txids the node names.
        competitors: Vec<TxId>,
    },
}
/// The inputs that may write a chain word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// A proof of this transaction's inclusion. A transaction of exactly 64
    /// bytes is never taken as proven by a merkle path, so the door reads a
    /// length beside the txid: the state's own measure
    /// ([`State::tx_len`]) when it holds one, against which a proof of
    /// another length is refused ([`CheckError::LengthMismatch`]); else the
    /// proof's, where a proof built from bytes ([`Proof::new`]) measures the
    /// state and a stated one ([`Proof::with_stated_len`]) is taken on the
    /// host's word. A state measured at 64 bytes takes no proof
    /// ([`CheckError::AmbiguousLength`]), and an unmeasured state refuses a
    /// proof taken out of a record, which carries no length
    /// ([`CheckError::Unmeasured`]).
    Proof(Proof),
    /// A proof of a competing transaction's inclusion. What the host must
    /// have shown: that the competitor spends an input of the tracked
    /// transaction, which the crate cannot see. The proof is built from the
    /// competitor's own bytes ([`Proof::new`]), so its txid is their hash
    /// and its length is theirs; any other proof is refused
    /// ([`CheckError::Unmeasured`]). The competitor is judged by its own
    /// length, never by the tracked transaction's measure. A competitor
    /// known only by someone's word is a hint or a node's verdict, never
    /// this.
    Competitor(Proof),
    /// A chain event from the host's header feed.
    Chain(ChainEvent),
    /// A verdict from the host's trusted node.
    Verdict(NodeVerdict),
    /// Recheck a mined word against the current headers.
    Recheck,
}
/// One instance the bytes were handed to and that has not been discharged,
/// with the host's time of its hand-over: one pair of the Lean's
/// `State.handedTo : List (String × Nat)`.
///
/// A host stores the list ([`State::handed_to`]) by this serde form,
/// `{"instance":"arcade","at":1700000000}`, and reads it back by it, never by
/// a hand map. `#[non_exhaustive]`: build one with [`HandedTo::new`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HandedTo {
    /// The instance's name, as the host's [`HostAction::HandOver`] gave it;
    /// a [`HostAction::Discharge`] is compared with it, never with a hint's
    /// `source`.
    pub instance: String,
    /// The host's time of the instance's latest hand-over.
    pub at: Timestamp,
}
impl HandedTo {
    /// One instance and the time of its hand-over.
    pub fn new(instance: impl Into<String>, at: Timestamp) -> Self {
        Self {
            instance: instance.into(),
            at,
        }
    }
}
/// Acts supplied by the host, including its clock and spend guard.
///
/// `#[non_exhaustive]` (bsv-stack-lean `docs/DECISIONS-2026-10-19.md`
/// 28.8.1 (c)): a later minor may add an act, so a host's match outside this
/// crate carries a wildcard arm.
///
/// ```compile_fail,E0004
/// use bsv_tracker::HostAction;
/// fn kind(action: &HostAction) -> u8 {
///     match action {
///         HostAction::Build => 0,
///         HostAction::Abandon => 1,
///         HostAction::SpendAttempt => 2,
///         HostAction::Tick(_) => 3,
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HostAction {
    /// The host built the transaction.
    Build,
    /// The host abandons the transaction.
    Abandon,
    /// The host is about to spend an output of the transaction.
    SpendAttempt,
    /// The host's clock reads this time.
    Tick(Timestamp),
    /// The bytes are about to leave for the named instance, at the host's
    /// time `at`: fed, and its result stored, before the post leaves. From
    /// `Unknown`, `Built` and `Handed` it writes `Handed` with the instance on
    /// [`State::handed_to`] (added, or its time renewed in its place) and
    /// clears a pending [`Reask::Reannounce`]; an `Abandoned` state is left
    /// whole (an abandoned template is never announced); on any other word it
    /// writes no word and only clears a pending `Reannounce`.
    /// `Tracker.HostAction.handOver`.
    HandOver {
        /// The instance the bytes are handed to.
        instance: String,
        /// The host's time of the post.
        at: Timestamp,
    },
    /// The host's reduction found that the named instance answered and keeps
    /// nothing: from `Handed` the instance leaves the list, and the word
    /// returns to `Built` when the list is empty. Every other word is
    /// untouched: one instance's refusal does not unsay another's take.
    /// `Tracker.HostAction.discharge`.
    Discharge {
        /// The instance, compared with the hand-over's `instance`.
        instance: String,
    },
    /// The end of a received payment's hold: `Unknown`, `Built` and `Handed`
    /// read `Abandoned`, the list emptied and a pending
    /// [`Reask::Reannounce`] cleared; every other word is untouched. The crate
    /// cannot see where a transaction came from: this is the act of a payment
    /// the host received, never of a transaction of its own.
    /// `Tracker.HostAction.endHold`.
    EndHold,
}
/// One input to `Tracker.step`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// Evidence that may write a chain word.
    Evidence(Evidence),
    /// A broadcaster's hint, which never writes a chain word.
    Hint(Hint),
    /// An act of the host.
    Host(HostAction),
}
/// A value for the host to execute; the crate never fetches anything.
///
/// `#[non_exhaustive]` (bsv-stack-lean `docs/DECISIONS-2026-10-19.md`
/// 28.8.1 (c)): a later minor may add a kind, so a host's match outside this
/// crate carries a wildcard arm. A re-ask of a kind the host does not name is
/// kept and surfaced by [`Reask::label`], never dropped by a wildcard arm.
///
/// ```compile_fail,E0004
/// use bsv_tracker::Reask;
/// fn kind(reask: &Reask) -> u8 {
///     match reask {
///         Reask::Reorg { .. } => 0,
///         Reask::Fork { .. } => 1,
///         Reask::Hint(_) => 2,
///         Reask::ProofFailed { .. } => 3,
///         Reask::Recheck => 4,
///         Reask::Age => 5,
///         Reask::Spend => 6,
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reask {
    /// The root moved: the active header at the mined height no longer
    /// carries the stored proof's root, so the word is stale; fetch a fresh
    /// proof. After a chain event a host fetches only for these, a few a pass;
    /// the pacing is the host's.
    Reorg {
        /// The reorg's fork height when a reorg's recheck found the root
        /// moved; otherwise the mined height (an invalidation, a recheck).
        fork_height: Height,
    },
    /// A fork, or a version-2 split, at or below the mined height; the word
    /// is suspect. A split reuses this re-ask with its `forkHeight`: the host's
    /// act, a recheck at its depth rule, is the same for both.
    Fork {
        /// The height at which the branches diverge (a split's `fork_height`).
        height: Height,
    },
    /// A hint that disagrees with the word; reconcile it.
    Hint(Hint),
    /// A proof or competitor proof failed its check.
    ProofFailed {
        /// The height the failed proof named.
        height: Height,
    },
    /// Recheck the mined word against the current headers.
    Recheck,
    /// The last hint is older than the age threshold.
    Age,
    /// The host is about to spend; recheck before it does.
    Spend,
    /// A `Handed` word whose oldest hand-over is one age threshold old: post
    /// the bytes again to each instance of [`State::handed_to`]. Never a proof
    /// request, and never [`Reask::Age`]; the next
    /// [`HostAction::HandOver`] clears it. `Tracker.Reask.reannounce`.
    Reannounce,
}
impl Reask {
    /// Every kind's word, in declaration order, which is the order of the
    /// compact form's re-ask tags (tag `i` is `LABELS[i]`). A word never
    /// changes and is never reused; a later minor only appends.
    pub const LABELS: &'static [&'static str] = &[
        "reorg",
        "fork",
        "hint",
        "proof_failed",
        "recheck",
        "age",
        "spend",
        "reannounce",
    ];
    /// The kind's stable word, the payload not in it: `reorg` for
    /// `Reorg { fork_height: 10 }`, `hint` for any `Hint(_)`.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Reorg { .. } => "reorg",
            Self::Fork { .. } => "fork",
            Self::Hint(_) => "hint",
            Self::ProofFailed { .. } => "proof_failed",
            Self::Recheck => "recheck",
            Self::Age => "age",
            Self::Spend => "spend",
            Self::Reannounce => "reannounce",
        }
    }
}
/// The host's parameters: one host, one set for an act.
///
/// `#[non_exhaustive]`: build them with [`Params::new`], which takes what a
/// host must say (the age threshold and the spend depth) and defaults the
/// rest; set a later field by assignment or by its `with_` method. A later
/// minor adds a field with a default here and breaks no host.
///
/// ```compile_fail,E0639
/// let _ = bsv_tracker::Params { age_threshold: 60, spend_depth: 1, bound: None, versions: vec![1, 2] };
/// ```
///
/// ```
/// let params = bsv_tracker::Params::new(60, 1).with_bound(Some(4));
/// assert_eq!((params.spend_depth, params.bound), (1, Some(4)));
/// assert_eq!(params.versions, [1, 2]);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Params {
    /// Seconds after the last hint before an announced or seen word is re-asked.
    pub age_threshold: Timestamp,
    /// The feed's named number: the most split keys the header service's
    /// contract says it can have open at once, as the host was told it.
    /// `None` until the feed names one, and `None` is no check: the host of
    /// today, row for row.
    ///
    /// With a number, a host whose open keys pass it after any row treats the
    /// feed as out of its contract: [`doubts::step_checked`] and
    /// [`doubts::run_checked`] answer [`doubts::Checked::Out`]. The number is
    /// the host's own and is never read from a snapshot or a feed row: a check
    /// against a number the checked party sends checks nothing. The tracker's
    /// reducer does not read it. The stack's `StartParams.bound`
    /// (`lean/Headers/Host.lean` section 12).
    ///
    /// [`doubts::step_checked`]: crate::doubts::step_checked
    /// [`doubts::run_checked`]: crate::doubts::run_checked
    /// [`doubts::Checked::Out`]: crate::doubts::Checked::Out
    pub bound: Option<u32>,
    /// The depth a `mined` word must have before a coin of the transaction is
    /// spendable with no application rule: the block at its height counts
    /// one, each height above it to the tip one more. The host sets it once,
    /// when it builds its parameters, and [`doubts::spendable_eff`] and
    /// [`doubts::spendable_c`] read it:
    /// it is never a number a caller passes per call. No floor is applied
    /// here: a depth of 0 passes a `mined` word whose root the headers carry
    /// even under an open split, as the Lean's guard does; a host's floor of
    /// 1 is the host's. The tracker's reducer does not read it. The stack's
    /// `Params.spendDepth` (`lean/Tracker.lean`).
    ///
    /// [`doubts::spendable_eff`]: crate::doubts::spendable_eff
    /// [`doubts::spendable_c`]: crate::doubts::spendable_c
    pub spend_depth: u64,
    /// The row versions the host's decoder reads, against which a served
    /// snapshot's folding decoder is checked ([`doubts::accepts`]). Defaults
    /// to [`Params::VERSIONS`], the versions [`decode_envelope`] reads. The
    /// stack's `StartParams.versions` (`lean/Headers/Host.lean` section 12).
    ///
    /// [`doubts::accepts`]: crate::doubts::accepts
    /// [`decode_envelope`]: crate::decode_envelope
    pub versions: Vec<u64>,
}
impl Params {
    /// The row versions [`decode_envelope`](crate::decode_envelope) reads: 1,
    /// and 2 for the split kinds.
    pub const VERSIONS: [u64; 2] = [1, 2];

    /// The parameters a host must say: the age threshold, in the clock's
    /// seconds, and the spend depth. The feed's named number is absent (no
    /// check, today's host) and the row versions are [`Params::VERSIONS`].
    #[must_use]
    pub fn new(age_threshold: Timestamp, spend_depth: u64) -> Self {
        Self {
            age_threshold,
            bound: None,
            spend_depth,
            versions: Self::VERSIONS.to_vec(),
        }
    }

    /// The same parameters with the feed's named number `bound`.
    #[must_use]
    pub fn with_bound(mut self, bound: Option<u32>) -> Self {
        self.bound = bound;
        self
    }

    /// The same parameters with the spend depth `spend_depth`.
    #[must_use]
    pub fn with_spend_depth(mut self, spend_depth: u64) -> Self {
        self.spend_depth = spend_depth;
        self
    }

    /// The same parameters with the row versions `versions`.
    #[must_use]
    pub fn with_versions(mut self, versions: Vec<u64>) -> Self {
        self.versions = versions;
        self
    }
}
/// The complete private state of one transaction.
///
/// Two states are equal when every field is, the measured length
/// ([`State::tx_len`]) among them: a measured state and an unmeasured one are
/// not equal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    txid: TxId,
    word: Word,
    evidence: Option<Arc<CheckedProof>>,
    suspect: bool,
    hints: Vec<Hint>,
    last_hint_at: Option<Timestamp>,
    reask: Option<Reask>,
    handed_to: Vec<HandedTo>,
    tx_len: Option<u64>,
}
impl State {
    /// Birth with no chain claim or stored evidence, as in `Tracker.State.init`.
    /// A state born from a txid is unmeasured: [`State::tx_len`] reads `None`
    /// until the host hands the transaction's bytes ([`State::measure`]).
    pub fn new(txid: impl Into<TxId>) -> Self {
        Self::born(txid.into().to_ascii_lowercase(), None)
    }
    /// Birth from the transaction's serialized bytes: the txid is their hash
    /// (the double SHA-256, reversed, as 64 lowercase hex digits),
    /// [`State::tx_len`] is their count, and the word is `Unknown` with no
    /// stored evidence. The bytes are hashed and counted, never decoded, and
    /// this never fails: any bytes hash to some txid, and the host compares
    /// [`State::txid`] with the txid it expects.
    ///
    /// A transaction of exactly 64 bytes is never taken as proven by a merkle
    /// path: a state measured at [`AMBIGUOUS_TX_LEN`] is tracked like any
    /// other and takes no proof, so it never reads `mined`.
    ///
    /// Ahead of the Lean: bsv-stack-lean `lean/Tracker.lean` at `9a727a64`
    /// has no measured birth yet (`State.init` with the field `txLen` set is
    /// its mirror when it lands).
    ///
    /// ```
    /// use bsv_tracker::{State, Word, AMBIGUOUS_TX_LEN};
    /// let state = State::measured(&[0x5a; 64]);
    /// assert_eq!(state.tx_len(), Some(AMBIGUOUS_TX_LEN));
    /// assert_eq!(state.word(), &Word::Unknown);
    /// ```
    #[must_use]
    pub fn measured(tx_bytes: &[u8]) -> Self {
        let (txid, len) = crate::evidence::digest(tx_bytes);
        Self::born(txid, Some(len))
    }
    /// A birth: the txid already in its lowercase form.
    pub(crate) fn born(txid: TxId, tx_len: Option<u64>) -> Self {
        Self {
            txid,
            word: Word::Unknown,
            evidence: None,
            suspect: false,
            hints: vec![],
            last_hint_at: None,
            reask: None,
            handed_to: vec![],
            tx_len,
        }
    }
    /// The host's tracked transaction.
    #[must_use]
    pub fn txid(&self) -> &str {
        &self.txid
    }
    /// The current word. It is immutable outside the transition function.
    #[must_use]
    pub fn word(&self) -> &Word {
        &self.word
    }
    /// The retained record, including when the word has become stale.
    #[must_use]
    pub fn evidence(&self) -> Option<&CheckedProof> {
        self.evidence.as_deref()
    }
    /// Whether an unresolved fork requires a header recheck before acting.
    #[must_use]
    pub fn suspect(&self) -> bool {
        self.suspect
    }
    /// Observations in newest received order, matching the Lean list.
    #[must_use]
    pub fn hints(&self) -> &[Hint] {
        &self.hints
    }
    /// Time of the most recently received hint, even for out of order clocks.
    #[must_use]
    pub fn last_hint_at(&self) -> Option<Timestamp> {
        self.last_hint_at
    }
    /// The next re-ask, which the host must execute and feed back.
    #[must_use]
    pub fn reask(&self) -> Option<&Reask> {
        self.reask.as_ref()
    }
    /// The instances the bytes were handed to and not discharged, each with
    /// the time of its hand-over, in the order of each instance's earliest
    /// hand-over (`Tracker.State.handedTo`). Not empty while the word is
    /// [`Word::Handed`] (`handedNamed_run`); emptied by evidence that writes
    /// `Mined` or `Rejected`, by a hint that raises the word and by
    /// [`HostAction::EndHold`]; kept under a `Conflicted` word that was
    /// `Handed`. A host stores it by [`HandedTo`]'s serde form.
    #[must_use]
    pub fn handed_to(&self) -> &[HandedTo] {
        &self.handed_to
    }
    /// The transaction's measured length in bytes, or `None` where the host
    /// never handed the bytes. It is written only from bytes the crate hashed
    /// to this state's txid ([`State::measured`], [`State::measure`],
    /// [`Tracker::track_bytes`](crate::Tracker::track_bytes)), never from a
    /// length a host states, and once written it does not change.
    ///
    /// A transaction of exactly 64 bytes is never taken as proven by a merkle
    /// path: a state that reads `Some(AMBIGUOUS_TX_LEN)` takes no proof, and
    /// its host asks none for it.
    ///
    /// Ahead of the Lean: the field's name there is `State.txLen`, which
    /// bsv-stack-lean `lean/Tracker.lean` at `9a727a64` does not hold yet.
    #[must_use]
    pub fn tx_len(&self) -> Option<u64> {
        self.tx_len
    }
    /// Hand the transaction's serialized bytes to a state born without them,
    /// so that it holds its measured length ([`State::tx_len`]). The bytes
    /// are hashed and counted, never decoded.
    ///
    /// - Bytes that hash to another txid: [`CheckError::TxidMismatch`], the
    ///   state left whole (no re-ask is written: this is not a proof).
    /// - A state already measured at the same length: `Ok`, nothing moves.
    /// - A state already measured at another length:
    ///   [`CheckError::LengthMismatch`], the state left whole.
    /// - An unmeasured state: the length is written. Off 64 bytes nothing
    ///   else moves, whatever the word.
    ///
    /// # The fault road
    ///
    /// A transaction of exactly 64 bytes is never taken as proven by a merkle
    /// path. When the measure taken here is [`AMBIGUOUS_TX_LEN`] and the
    /// state held a proof, the proof is unsaid AND the fault is returned: the
    /// length is written, the stored record is dropped ([`State::evidence`]
    /// reads `None`), a `Mined` or `Stale` word reads `Unknown` with the
    /// suspect mark and the re-ask cleared, any other word keeps its word and
    /// its re-ask, and the call answers
    /// `Err(`[`CheckError::AmbiguousLength`]`)`. So this one fault, from this
    /// call, means the state has moved; the other two leave it whole.
    ///
    /// This is a fault, never a transition a host may lean on. It is reached
    /// only when the state took its proof unmeasured (a state born from a
    /// txid, or a row restored from version 1 bytes) and the bytes handed
    /// later are 64 long. The Lean's machine has no such step: there the
    /// length a host gives for a transaction is a hypothesis (the host's
    /// statement is true), and this is what the crate does when that
    /// hypothesis is found broken.
    ///
    /// Ahead of the Lean: bsv-stack-lean `lean/Tracker.lean` at `9a727a64`
    /// has no measure yet.
    ///
    /// # Errors
    ///
    /// [`CheckError::TxidMismatch`], [`CheckError::LengthMismatch`] and, on
    /// the fault road above, [`CheckError::AmbiguousLength`].
    pub fn measure(&mut self, tx_bytes: &[u8]) -> Result<(), CheckError> {
        let (txid, len) = crate::evidence::digest(tx_bytes);
        if txid != self.txid {
            return Err(CheckError::TxidMismatch);
        }
        self.take_len(len)
    }

    /// The arms of [`State::measure`] after the txid: a length measured from
    /// bytes of this transaction, or held for it by a
    /// [`Tracker`](crate::Tracker) across a restore.
    pub(crate) fn take_len(&mut self, len: u64) -> Result<(), CheckError> {
        match self.tx_len {
            Some(held) if held == len => return Ok(()),
            Some(held) => {
                return Err(CheckError::LengthMismatch {
                    measured: held,
                    offered: len,
                })
            }
            None => self.tx_len = Some(len),
        }
        if len == AMBIGUOUS_TX_LEN && self.holds_a_proof() {
            self.unsay();
            return Err(CheckError::AmbiguousLength);
        }
        Ok(())
    }

    /// Whether a proof was taken for this transaction and still shows: a
    /// stored record, or a word only a proof writes. The record is the
    /// tracked transaction's own wherever it sits; a competitor's proof is
    /// never kept on it, so a `conflicted` word written by a competitor's
    /// proof holds none of ours.
    fn holds_a_proof(&self) -> bool {
        self.evidence.is_some() || matches!(self.word, Word::Mined(_) | Word::Stale)
    }

    /// Unsay a proof taken for a transaction measured at 64 bytes: the record
    /// is dropped; a `Mined` or `Stale` word reads `Unknown`, its mark and
    /// its re-ask cleared; any other word keeps its word and its re-ask.
    fn unsay(&mut self) {
        self.evidence = None;
        if matches!(self.word, Word::Mined(_) | Word::Stale) {
            self.word = Word::Unknown;
            self.suspect = false;
            self.reask = None;
        }
    }
    /// The compact byte form (bsv-stack-lean #92), version 2: a version byte,
    /// the txid, the word, the evidence, the suspect mark, the hints, the last
    /// hint time, the re-ask, the instances of [`State::handed_to`], two
    /// fields this release holds empty (the record of a conflict's
    /// competitors and the word held when it began), and the measured length
    /// ([`State::tx_len`]). The layout is in the source of `src/compact.rs`; the
    /// version byte changes when the layout does, so a later release tells an
    /// earlier layout from its own rather than misreading it.
    #[must_use]
    pub fn to_compact(&self) -> Vec<u8> {
        crate::compact::encode(&crate::compact::Fields {
            txid: &self.txid,
            word: &self.word,
            evidence: self.evidence.as_deref(),
            suspect: self.suspect,
            hints: &self.hints,
            last_hint_at: self.last_hint_at,
            reask: self.reask.as_ref(),
            handed_to: &self.handed_to,
            tx_len: self.tx_len,
        })
    }
    /// Restore a state from [`State::to_compact`]'s bytes.
    ///
    /// The rule: a restored `mined` word is still suspect until its proof is
    /// re-checked against the host's headers. [`CheckedProof`] has no
    /// deserializer, because bytes are not a check: the chain may have moved
    /// while the state was stored. So the stored proof comes back as the
    /// record (its path reduces for this txid to the stored root, which equals
    /// the stored header's root), the word comes back `mined` on that record,
    /// and the state is marked [`suspect`](State::suspect) with
    /// [`Reask::Recheck`] due when no other re-ask was pending. A
    /// [`Tracker`](crate::Tracker) holding it rechecks it on the next tip,
    /// as it does any suspect word; a host holding the state alone feeds
    /// [`Evidence::Recheck`] before it serves the inclusion.
    ///
    /// The measured length returns as it was written: a version 2 row
    /// restores [`State::tx_len`], and a version 1 row restores unmeasured.
    /// A `mined` row of version 1 was written before the crate measured a
    /// transaction; it restores `mined` as before and nothing more is asked
    /// of it, and a host that later hands its bytes meets
    /// [`State::measure`]'s rules.
    ///
    /// Every other field returns exactly: a state whose word is not `mined`,
    /// or that was already suspect with a re-ask pending, restores equal to
    /// the state that wrote the bytes, and a restored state writes bytes that
    /// restore equal to it.
    ///
    /// # Errors
    ///
    /// [`CompactError::UnknownVersion`] for a version this crate does not
    /// read (it reads 1 and 2; a version 1 row restores with no instance), [`CompactError::Truncated`], [`CompactError::TrailingBytes`] and
    /// [`CompactError::Malformed`] for bytes the encoder never writes (a
    /// `mined` word with no stored proof among them), and
    /// [`CompactError::Evidence`] for a stored proof that does not hold
    /// together, or that stands beside a measured length of 64 bytes
    /// ([`CheckError::AmbiguousLength`]).
    pub fn from_compact(bytes: &[u8]) -> Result<Self, CompactError> {
        let parts = crate::compact::decode(bytes)?;
        let evidence = parts.evidence.map(Arc::new);
        let mined = parts.word.is_none();
        let word = if let Some(word) = parts.word {
            word
        } else {
            let checked = evidence
                .as_ref()
                .ok_or(CompactError::Malformed("mined word with no stored proof"))?;
            Word::Mined(Mined {
                checked: Arc::clone(checked),
            })
        };
        let mut state = Self {
            txid: parts.txid,
            word,
            evidence,
            suspect: parts.suspect,
            hints: parts.hints,
            last_hint_at: parts.last_hint_at,
            reask: parts.reask,
            handed_to: parts.handed_to,
            tx_len: parts.tx_len,
        };
        if mined {
            state.suspect = true;
            state.reask.get_or_insert(Reask::Recheck);
        }
        Ok(state)
    }
    /// Execute one Lean transition. Proof and header failures retain the word
    /// and record the re-ask before returning an error to the host.
    ///
    /// A transaction of exactly 64 bytes is never taken as proven by a merkle
    /// path: a state measured at [`AMBIGUOUS_TX_LEN`] ([`State::tx_len`])
    /// answers every [`Evidence::Proof`] for its txid with
    /// [`CheckError::AmbiguousLength`] and is left whole, with no re-ask
    /// written (a re-ask is an order to fetch, and no fetch helps). Any other
    /// measured state takes a proof of its own length alone; an unmeasured
    /// state is measured by a proof built from bytes, takes a stated one on
    /// the host's word, and refuses a record's proof, which carries no
    /// length. A competitor's proof is judged by the competitor's own
    /// length, read from its bytes.
    ///
    /// # Errors
    ///
    /// The [`CheckError`] of a proof, competitor, chain event or recheck whose
    /// check fails; the word is kept and the re-ask recorded.
    /// [`CheckError::AmbiguousLength`] for a proof offered to a state
    /// measured at 64 bytes; nothing is recorded.
    /// [`CheckError::LengthMismatch`] for a proof of another length than the
    /// state's measure, and [`CheckError::Unmeasured`] for a proof with no
    /// length an unmeasured state can read or a competitor's proof not built
    /// from its bytes: the word and the record kept, the re-ask recorded.
    pub fn step<H: Headers + ?Sized>(
        &mut self,
        params: &Params,
        headers: &H,
        input: Input,
    ) -> Result<(), CheckError> {
        match input {
            Input::Hint(hint) => self.on_hint(hint),
            Input::Host(action) => self.on_host(params, &action),
            Input::Evidence(Evidence::Proof(proof)) => self.on_proof(headers, proof)?,
            Input::Evidence(Evidence::Competitor(proof)) => self.on_competitor(headers, proof)?,
            Input::Evidence(Evidence::Chain(event)) => self.on_chain(headers, &event)?,
            Input::Evidence(Evidence::Verdict(verdict)) => self.on_verdict(verdict),
            Input::Evidence(Evidence::Recheck) => self.recheck(headers, None)?,
        }
        Ok(())
    }

    fn on_hint(&mut self, hint: Hint) {
        let before = self.word.clone();
        match (&self.word, &hint.status) {
            // A node we did not broadcast through holds the bytes, so someone
            // announced it; the chain has no author (ruling (2), bsv-stack-lean
            // docs/RULINGS-2026-10-12.md, 2026-10-20 19:4xZ).
            (
                Word::Built | Word::Handed,
                HintStatus::Accepted | HintStatus::MempoolPresent { .. },
            ) => {
                self.word = Word::Announced;
            }
            (Word::Built | Word::Handed | Word::Announced, HintStatus::Seen) => {
                self.word = Word::Seen;
            }
            _ => {}
        }
        // A hint that moves the word empties the list (`Tracker.onHint`): a
        // take is not unsaid by another instance's silence.
        if self.word != before {
            self.handed_to.clear();
        }
        if !agrees(&self.word, &hint.status) {
            self.reask = Some(Reask::Hint(hint.clone()));
        }
        self.last_hint_at = Some(hint.observed);
        self.hints.insert(0, hint);
    }

    fn on_host(&mut self, params: &Params, action: &HostAction) {
        match action {
            HostAction::Build if self.word == Word::Unknown => self.word = Word::Built,
            HostAction::Abandon if matches!(self.word, Word::Unknown | Word::Built) => {
                self.word = Word::Abandoned;
            }
            HostAction::SpendAttempt => self.reask = Some(Reask::Spend),
            // Lean adds unbounded naturals. Checked addition keeps an
            // overflowing deadline from becoming prematurely due.
            HostAction::Tick(now)
                if matches!(self.word, Word::Announced | Word::Seen)
                    && self
                        .last_hint_at
                        .and_then(|t| t.checked_add(params.age_threshold))
                        .is_some_and(|deadline| deadline <= *now) =>
            {
                self.reask = Some(Reask::Age);
            }
            HostAction::Tick(now) if self.handed_due(params, *now) => {
                self.reask = Some(Reask::Reannounce);
            }
            HostAction::HandOver { instance, at } => self.on_hand_over(instance, *at),
            HostAction::Discharge { instance } => self.on_discharge(instance),
            HostAction::EndHold => {
                if matches!(self.word, Word::Unknown | Word::Built | Word::Handed) {
                    self.word = Word::Abandoned;
                    self.handed_to.clear();
                    self.clear_reannounce();
                }
            }
            _ => {}
        }
    }

    /// `Tracker.onHandOver`.
    fn on_hand_over(&mut self, instance: &str, at: Timestamp) {
        match self.word {
            Word::Abandoned => {}
            Word::Unknown | Word::Built | Word::Handed => {
                self.word = Word::Handed;
                renew(&mut self.handed_to, instance, at);
                self.clear_reannounce();
            }
            _ => self.clear_reannounce(),
        }
    }

    /// `Tracker.onDischarge`.
    fn on_discharge(&mut self, instance: &str) {
        if self.word == Word::Handed {
            self.handed_to.retain(|h| h.instance != instance);
            if self.handed_to.is_empty() {
                self.word = Word::Built;
            }
        }
    }

    /// `Tracker.clearReannounce`.
    fn clear_reannounce(&mut self) {
        if self.reask == Some(Reask::Reannounce) {
            self.reask = None;
        }
    }

    /// `Tracker.oldestHandOver`.
    fn oldest_hand_over(&self) -> Option<Timestamp> {
        self.handed_to.iter().map(|h| h.at).min()
    }

    /// `Tracker.handedDue`: a `Handed` word whose oldest hand-over plus the
    /// age threshold is at most `now`. Lean adds unbounded naturals; checked
    /// addition keeps an overflowing deadline from becoming prematurely due.
    fn handed_due(&self, params: &Params, now: Timestamp) -> bool {
        self.word == Word::Handed
            && self
                .oldest_hand_over()
                .and_then(|t| t.checked_add(params.age_threshold))
                .is_some_and(|deadline| deadline <= now)
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
        // A transaction of exactly 64 bytes is never taken as proven by a
        // merkle path: the state is left whole.
        if self.tx_len == Some(AMBIGUOUS_TX_LEN) {
            return Err(CheckError::AmbiguousLength);
        }
        // The length: the state's measure when it holds one, else the
        // proof's. No proof carries 64, by its constructors.
        match (self.tx_len, proof.length()) {
            (Some(measured), Some(Len::Measured(offered) | Len::Stated(offered)))
                if offered != measured =>
            {
                self.reask = Some(Reask::ProofFailed { height });
                return Err(CheckError::LengthMismatch { measured, offered });
            }
            (None, None) => {
                self.reask = Some(Reask::ProofFailed { height });
                return Err(CheckError::Unmeasured);
            }
            (None, Some(Len::Measured(len))) => self.tx_len = Some(len),
            (Some(_), _) | (None, Some(Len::Stated(_))) => {}
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
                self.handed_to.clear();
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
        // The competitor's own length, read from its bytes; never the tracked
        // transaction's measure.
        if !matches!(proof.length(), Some(Len::Measured(_))) {
            self.reask = Some(Reask::ProofFailed { height });
            return Err(CheckError::Unmeasured);
        }
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

    /// The local recheck: the stored proof's root against the active header's
    /// root at the word's height, a header read and never a fetch. Equal, the
    /// word stands and the marks clear; different, the word is stale with
    /// [`Reask::Reorg`] at `fork_height` (the word's height when none is
    /// given); no header, the word stands with [`Reask::Recheck`].
    fn recheck<H: Headers + ?Sized>(
        &mut self,
        headers: &H,
        fork_height: Option<Height>,
    ) -> Result<(), CheckError> {
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
            Ok(Some(_)) => self.stale(fork_height.unwrap_or(height)),
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

    /// A chain event names the words to recheck; only the headers move one.
    ///
    /// The rule by the root (ruling (5), bsv-stack-lean
    /// docs/RULINGS-2026-10-12.md, 2026-10-20 19:4xZ): a `mined` word is
    /// `stale` when, and only when, its stored proof's root is not the active
    /// header's root at its height in the host's headers. A reorg at or below
    /// the word's height and an invalidation of its checked header both run
    /// the local recheck; the event's numbers are the bound of that recheck,
    /// never the stale mark. The word is suspect while the recheck is open, so
    /// a missing header leaves it suspect with [`Reask::Recheck`] (fail
    /// closed) and the next tip rechecks it. A fork or a split marks the word
    /// suspect with [`Reask::Fork`]; a resolved split and a tip recheck the
    /// suspect words.
    pub(crate) fn on_chain<H: Headers + ?Sized>(
        &mut self,
        headers: &H,
        event: &ChainEvent,
    ) -> Result<(), CheckError> {
        match event {
            ChainEvent::Tip { .. } if self.suspect => self.recheck(headers, None)?,
            ChainEvent::Fork { height, .. } if self.word.height().is_some_and(|h| *height <= h) => {
                self.suspect = true;
                self.reask = Some(Reask::Fork { height: *height });
            }
            ChainEvent::Reorg { fork_height, .. }
                if self.word.height().is_some_and(|h| *fork_height <= h) =>
            {
                self.suspect = true;
                self.recheck(headers, Some(*fork_height))?;
            }
            // A split is an observation, not a reorg: the service still answers
            // its most-work chain and depth is the host's (bsv-stack-lean
            // docs/RULINGS-2026-10-12.md:323-329, 2026-10-16 19:3xZ). It writes no word.
            ChainEvent::Split { fork_height, .. }
                if self.word.height().is_some_and(|h| *fork_height <= h) =>
            {
                self.suspect = true;
                self.reask = Some(Reask::Fork {
                    height: *fork_height,
                });
            }
            // A resolved split moves a word only through the checked snapshot.
            ChainEvent::SplitResolved { fork_height, .. }
                if self.suspect && self.word.height().is_some_and(|h| *fork_height <= h) =>
            {
                self.recheck(headers, None)?;
            }
            ChainEvent::Invalidated { block_hash } if self.checked_against(block_hash) => {
                self.suspect = true;
                self.recheck(headers, None)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn checked_against(&self, block_hash: &str) -> bool {
        matches!(&self.word, Word::Mined(m) if m.checked().header().hash == block_hash)
    }

    fn on_verdict(&mut self, verdict: NodeVerdict) {
        if matches!(self.word, Word::Mined(_)) {
            self.reask = Some(Reask::Recheck);
            return;
        }
        self.word = match verdict {
            // `Tracker.verdictHanded`: `rejected` empties the list,
            // `conflicted` keeps it.
            NodeVerdict::Rejected { reason } => {
                self.handed_to.clear();
                Word::Rejected { reason }
            }
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

/// `Tracker.renew`: the instance added at the end, or its time renewed in its
/// place.
fn renew(list: &mut Vec<HandedTo>, instance: &str, at: Timestamp) {
    if let Some(held) = list.iter_mut().find(|h| h.instance == instance) {
        held.at = at;
    } else {
        list.push(HandedTo::new(instance, at));
    }
}

fn agrees(word: &Word, status: &HintStatus) -> bool {
    match (word, status) {
        (
            Word::Announced | Word::Seen,
            HintStatus::Accepted | HintStatus::MempoolPresent { .. },
        )
        | (Word::Seen, HintStatus::Seen)
        | (Word::Stale, HintStatus::StaleBlock)
        | (Word::Rejected { .. }, HintStatus::Rejected { .. })
        | (Word::Conflicted { .. }, HintStatus::DoubleSpend { .. }) => true,
        (Word::Mined(mined), HintStatus::Mined { height }) => mined.height() == *height,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{Evidence, Input, Params, Reask, State, Word};
    use crate::evidence::Len;
    use crate::{CheckError, Header, HeaderError, Headers, Height, Proof};
    use bsv_rs::transaction::{MerklePath, MerklePathLeaf};

    /// Headers that carry every root asked of them at their one height.
    struct Carry(Height, String);
    impl Headers for Carry {
        fn header_at(&self, height: Height) -> Result<Option<Header>, HeaderError> {
            Ok((height == self.0).then(|| Header {
                hash: "cc".repeat(32),
                merkle_root: self.1.clone(),
            }))
        }
        fn tip_height(&self) -> Result<Height, HeaderError> {
            Ok(self.0)
        }
    }

    /// The row of the proof door no public road reaches without two byte
    /// strings of one hash: a state measured at one length offered a proof
    /// built from bytes of its txid at another. Refused, the word and the
    /// record kept, the re-ask written; the same proof at the state's own
    /// length is taken.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn a_proof_from_bytes_of_another_length_is_refused_and_the_word_kept() {
        let tx_bytes = [0x5a_u8; 191];
        let state = State::measured(&tx_bytes);
        let txid = state.txid().to_owned();
        let path = MerklePath::new(
            10,
            vec![vec![
                MerklePathLeaf::new_txid(0, txid.clone()),
                MerklePathLeaf::new(1, "bb".repeat(32)),
            ]],
        )
        .unwrap();
        let proof = Proof::new(&tx_bytes, path).unwrap();
        let headers = Carry(10, proof.root().unwrap());
        let params = Params::new(60, 1);
        let mut s = state.clone();
        let other = proof.clone().on_length(Some(Len::Measured(190)));
        assert_eq!(
            s.step(&params, &headers, Input::Evidence(Evidence::Proof(other))),
            Err(CheckError::LengthMismatch {
                measured: 191,
                offered: 190,
            })
        );
        assert_eq!(s.word(), &Word::Unknown);
        assert_eq!(s.evidence(), None);
        assert_eq!(s.reask(), Some(&Reask::ProofFailed { height: 10 }));
        assert_eq!(s.tx_len(), Some(191));
        assert_eq!(
            s.step(&params, &headers, Input::Evidence(Evidence::Proof(proof))),
            Ok(())
        );
        assert!(matches!(s.word(), Word::Mined(_)));
        assert_eq!(s.tx_len(), Some(191));
    }

    /// The arm no public road reaches without two byte strings of one hash:
    /// a state measured at one length handed bytes of its txid at another.
    #[test]
    fn a_second_measure_of_another_length_is_refused_and_the_state_left_whole() {
        let tx_bytes = [0x5a_u8; 191];
        let (txid, len) = crate::evidence::digest(&tx_bytes);
        assert_eq!(len, 191);
        let mut state = State::born(txid, Some(190));
        let before = state.clone();
        assert_eq!(
            state.measure(&tx_bytes),
            Err(CheckError::LengthMismatch {
                measured: 190,
                offered: 191,
            })
        );
        assert_eq!(state, before);
    }
}
