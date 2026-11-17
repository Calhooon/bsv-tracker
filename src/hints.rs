use crate::{Height, Timestamp, TxId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A broadcaster's claim reduced to the Lean hint vocabulary.
///
/// `#[non_exhaustive]` (the room made before 0.9.0, bsv-stack-lean
/// `docs/DECISIONS-2026-10-19.md` 28.8.1 (c)): a later minor may add a kind,
/// so a host's match outside this crate carries a wildcard arm.
///
/// ```compile_fail,E0004
/// use bsv_tracker::HintStatus;
/// fn kind(status: &HintStatus) -> u8 {
///     match status {
///         HintStatus::Accepted => 0,
///         HintStatus::Seen => 1,
///         HintStatus::Mined { .. } => 2,
///         HintStatus::StaleBlock => 3,
///         HintStatus::Rejected { .. } => 4,
///         HintStatus::DoubleSpend { .. } => 5,
///         HintStatus::OrphanMempool => 6,
///         HintStatus::Unknown => 7,
///         HintStatus::MempoolPresent { .. } => 8,
///     }
/// }
/// ```
///
/// **The stored form.** The serde form below is this crate's stored form of
/// a status: an object whose key `status` holds the kind's word
/// ([`HintStatus::label`]) and whose payload sits beside it under its
/// field's name: `height` for `mined`, `reason` for `rejected`,
/// `competitors` for `double_spend` (left out when there are none) and
/// `courier` for `mempool_present` (the courier that answered, so a status
/// sits beside a hint's own `source` in one object). So
/// `{"status":"accepted"}`, `{"status":"mined","height":900}` and
/// `{"status":"mempool_present","courier":"c"}`. The reader refuses a word it
/// does not know and a kind whose payload is missing; it never reads either
/// as [`HintStatus::Unknown`]. The tag key, the words and the field names are
/// stable under the rule of [`HintStatus::LABELS`]: a word never changes and
/// is never reused, and a later minor only appends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[non_exhaustive]
pub enum HintStatus {
    /// The broadcaster accepted the transaction.
    Accepted,
    /// The broadcaster reports the transaction seen on the network.
    Seen,
    /// The broadcaster claims a block at a height; never inclusion evidence.
    Mined {
        /// The height the broadcaster names.
        height: Height,
    },
    /// The broadcaster reports the transaction mined in a block that went stale.
    StaleBlock,
    /// The broadcaster reports a rejection.
    Rejected {
        /// The broadcaster's reason, or its status word when it gave none.
        reason: String,
    },
    /// The broadcaster reports a double spend.
    DoubleSpend {
        /// The competing txids the broadcaster names.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        competitors: Vec<TxId>,
    },
    /// The broadcaster reports the transaction in its orphan mempool.
    OrphanMempool,
    /// A status the vocabulary does not reduce to any other hint.
    Unknown,
    /// A courier holds the transaction in a mempool (bsv-stack-lean #93): a
    /// node the host did not broadcast through returned its bytes, unmined.
    /// Someone announced it, since the chain has no author, so it moves
    /// `built` to `announced` as a broadcaster's acceptance does (ruling (2),
    /// bsv-stack-lean docs/RULINGS-2026-10-12.md, 2026-10-20 19:4xZ); it never
    /// writes `seen` or a chain word and touches no evidence. It agrees with an
    /// `announced` or `seen` word and, as every hint does,
    /// restarts the age clock, so the age re-ask falls due `age_threshold`
    /// after the last presence; against any other word it schedules the
    /// hint's re-ask. [`Verdict::into_hint`] never produces it.
    MempoolPresent {
        /// The courier that answered the presence (the host's name for it).
        /// Its stored form's field is `courier`.
        #[serde(rename = "courier")]
        source: String,
    },
}
impl HintStatus {
    /// Every kind's word, in declaration order, which is the order of the
    /// compact form's status tags (tag `i` is `LABELS[i]`). A word never
    /// changes and is never reused; a later minor only appends.
    pub const LABELS: &'static [&'static str] = &[
        "accepted",
        "seen",
        "mined",
        "stale_block",
        "rejected",
        "double_spend",
        "orphan_mempool",
        "unknown",
        "mempool_present",
    ];
    /// The kind's stable word, the payload not in it: `mined` for
    /// `Mined { height: 900 }`. It is the `status` of the stored form.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Seen => "seen",
            Self::Mined { .. } => "mined",
            Self::StaleBlock => "stale_block",
            Self::Rejected { .. } => "rejected",
            Self::DoubleSpend { .. } => "double_spend",
            Self::OrphanMempool => "orphan_mempool",
            Self::Unknown => "unknown",
            Self::MempoolPresent { .. } => "mempool_present",
        }
    }
}
/// An observed hint, carrying its source and host supplied time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    /// The host's name for the broadcaster or feed that supplied the hint.
    pub source: String,
    /// The claim, in the Lean hint vocabulary.
    pub status: HintStatus,
    /// When the host observed the hint, on its own clock.
    pub observed: Timestamp,
    /// The original broadcaster metadata, when reduced through `Verdict`.
    /// This retains the raw status, reason and named competitors for re-asks.
    pub verdict: Option<Verdict>,
}
impl Hint {
    /// A reduced hint, for hosts whose source already supplies the Lean vocabulary.
    pub fn new(source: impl Into<String>, status: HintStatus, observed: Timestamp) -> Self {
        Self {
            source: source.into(),
            status,
            observed,
            verdict: None,
        }
    }
}
/// Raw broadcaster metadata. HTTP acceptance is not chain evidence.
///
/// `#[non_exhaustive]` (bsv-stack-lean `docs/DECISIONS-2026-10-19.md`
/// 28.8.1 (c)): a later minor may add a field, so a host outside this crate
/// builds one with [`Verdict::new`] and assigns the fields after it; the
/// four fields stay public to read and to assign.
///
/// ```compile_fail,E0639
/// use bsv_tracker::Verdict;
/// let _ = Verdict {
///     status: "SEEN_ON_NETWORK".into(),
///     height: None,
///     reason: None,
///     competitors: vec![],
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Verdict {
    /// The `txStatus` word, trimmed and upper-cased by [`Verdict::from_arc_body`].
    pub status: String,
    /// The raw `blockHeight`, when present.
    pub height: Option<Height>,
    /// The raw `extraInfo`, when present: text, never a status.
    pub reason: Option<String>,
    /// The `competingTxs`, lowercased and checked for shape and uniqueness.
    pub competitors: Vec<TxId>,
}
impl Verdict {
    /// A verdict holding `status` as given, not trimmed and not checked, with
    /// no height, no reason and no competitors. A host assigns the other
    /// fields after it. [`Verdict::from_arc_body`] stays the checked reader
    /// of a broadcaster's body; this constructor checks nothing.
    #[must_use]
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            height: None,
            reason: None,
            competitors: Vec::new(),
        }
    }
    /// Read an ARC body as metadata, checking its echoed txid and the bounded
    /// status vocabulary at bsv-rs@7bc623c. HTTP status is never consulted.
    ///
    /// The word is trimmed before the vocabulary check and kept trimmed and
    /// upper-cased, so `" SEEN_ON_NETWORK "` reads as `SEEN_ON_NETWORK`
    /// (0.4.1; bsv-stack-lean #169, the spaced body of LOW's door test
    /// `tracker_vocabulary_differs_from_the_door_exactly_here` in
    /// bsv-overlay-cloudflare `crates/overlay-cloudflare/src/door_words.rs`).
    ///
    /// # Errors
    ///
    /// [`VerdictError::Shape`] when the body does not decode,
    /// [`VerdictError::TxidMismatch`] when it echoes another or a malformed
    /// txid, [`VerdictError::InvalidMetadata`] when its text or competitors
    /// break the bounds, and [`VerdictError::UnknownStatus`] for a status word
    /// outside the vocabulary.
    pub fn from_arc_body(expected_txid: &str, bytes: &[u8]) -> Result<Self, VerdictError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            txid: Option<String>,
            tx_status: String,
            block_height: Option<Height>,
            extra_info: Option<String>,
            competing_txs: Option<Vec<TxId>>,
        }
        let body: Body =
            serde_json::from_slice(bytes).map_err(|e| VerdictError::Shape(e.to_string()))?;
        if body
            .txid
            .as_deref()
            .is_some_and(|txid| !is_txid(txid) || !txid.eq_ignore_ascii_case(expected_txid))
        {
            return Err(VerdictError::TxidMismatch);
        }
        if !bounded_text(&body.tx_status, 128, false)
            || body
                .extra_info
                .as_deref()
                .is_some_and(|info| !bounded_text(info, 8_192, true))
        {
            return Err(VerdictError::InvalidMetadata);
        }
        let competitors: Vec<_> = body
            .competing_txs
            .unwrap_or_default()
            .into_iter()
            .map(|id| id.to_ascii_lowercase())
            .collect();
        if competitors.len() > 256
            || competitors.iter().any(|id| !is_txid(id))
            || competitors
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != competitors.len()
        {
            return Err(VerdictError::InvalidMetadata);
        }
        let status = body.tx_status.trim().to_ascii_uppercase();
        let known = matches!(
            status.as_str(),
            "SUCCESS"
                | "UNKNOWN"
                | "RECEIVED"
                | "QUEUED"
                | "STORED"
                | "ANNOUNCED_TO_NETWORK"
                | "REQUESTED_BY_NETWORK"
                | "SENT_TO_NETWORK"
                | "ACCEPTED_BY_NETWORK"
                | "SEEN_ON_NETWORK"
                | "SEEN_MULTIPLE_NODES"
                | "MINED"
                | "CONFIRMED"
                | "IMMUTABLE"
                | "MINED_IN_STALE_BLOCK"
                | "REJECTED"
                | "INVALID"
                | "MALFORMED"
                | "DOUBLE_SPEND_ATTEMPTED"
                | "UTXO_SPENT"
                | "SEEN_IN_ORPHAN_MEMPOOL"
                | "PENDING_RETRY"
                | "STUMP_PROCESSING"
        );
        if !known {
            return Err(VerdictError::UnknownStatus(body.tx_status));
        }
        Ok(Self {
            status,
            height: body.block_height,
            reason: body.extra_info,
            competitors,
        })
    }
    /// Interpret a status as a hint; no result is a node verdict or a proof.
    ///
    /// Only the status word moves the status (0.4.1; bsv-stack-lean #169,
    /// LOW's door test `tracker_vocabulary_differs_from_the_door_exactly_here`
    /// in bsv-overlay-cloudflare `crates/overlay-cloudflare/src/door_words.rs`):
    /// `extraInfo` is text and stays on `reason`, so an `ORPHAN` or
    /// `reorg_unmined` there no longer overrides the word; only
    /// `SEEN_IN_ORPHAN_MEMPOOL` is [`HintStatus::OrphanMempool`] and only the
    /// stale-block words are [`HintStatus::StaleBlock`]. `PENDING_RETRY` and
    /// `STUMP_PROCESSING`, a broadcaster still working on it, are
    /// [`HintStatus::Accepted`]. A mined word with no height stays
    /// [`HintStatus::Unknown`], and named competitors still turn an accepting
    /// word into [`HintStatus::DoubleSpend`].
    pub fn into_hint(self, source: impl Into<String>, observed: Timestamp) -> Hint {
        let status = match self.status.to_ascii_uppercase().as_str() {
            "SUCCESS"
            | "SENT_TO_NETWORK"
            | "RECEIVED"
            | "QUEUED"
            | "STORED"
            | "ANNOUNCED_TO_NETWORK"
            | "REQUESTED_BY_NETWORK"
            | "ACCEPTED_BY_NETWORK"
            | "PENDING_RETRY"
            | "STUMP_PROCESSING" => HintStatus::Accepted,
            "SEEN_ON_NETWORK" | "SEEN_MULTIPLE_NODES" => HintStatus::Seen,
            "MINED" | "CONFIRMED" | "IMMUTABLE" => self
                .height
                .map_or(HintStatus::Unknown, |height| HintStatus::Mined { height }),
            "MINED_IN_STALE_BLOCK" | "REORG_UNMINED" | "REORG_UNCONFIRMED" => {
                HintStatus::StaleBlock
            }
            "SEEN_IN_ORPHAN_MEMPOOL" => HintStatus::OrphanMempool,
            "DOUBLE_SPEND_ATTEMPTED" | "UTXO_SPENT" => HintStatus::DoubleSpend {
                competitors: self.competitors.clone(),
            },
            "REJECTED" | "INVALID" | "MALFORMED" => HintStatus::Rejected {
                reason: self
                    .reason
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| self.status.clone()),
            },
            _ => HintStatus::Unknown,
        };
        let status = if !self.competitors.is_empty()
            && matches!(status, HintStatus::Accepted | HintStatus::Seen)
        {
            // A named competitor cannot disappear behind an accepting word.
            HintStatus::DoubleSpend {
                competitors: self.competitors.clone(),
            }
        } else {
            status
        };
        Hint {
            source: source.into(),
            status,
            observed,
            verdict: Some(self),
        }
    }
}

/// An invalid broadcaster body is a reported fault, never chain evidence.
///
/// `#[non_exhaustive]` (bsv-stack-lean `docs/DECISIONS-2026-10-19.md`
/// 28.8.1 (c), the keeper's reading of 2026-11-17 17:1xZ): a later minor's
/// reader may add a variant, so a host's match outside this crate carries a
/// wildcard arm that reports the fault.
///
/// ```compile_fail,E0004
/// use bsv_tracker::VerdictError;
/// fn kind(error: &VerdictError) -> u8 {
///     match error {
///         VerdictError::Shape(_) => 0,
///         VerdictError::TxidMismatch => 1,
///         VerdictError::InvalidMetadata => 2,
///         VerdictError::UnknownStatus(_) => 3,
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum VerdictError {
    /// The expected fields cannot be decoded.
    Shape(String),
    /// The body names another transaction or a malformed identifier.
    TxidMismatch,
    /// Text or competing identifiers violate the pinned metadata bounds.
    InvalidMetadata,
    /// The sender uses a word this adapter does not know.
    UnknownStatus(String),
}
impl fmt::Display for VerdictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for VerdictError {}

fn is_txid(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn bounded_text(value: &str, max: usize, empty: bool) -> bool {
    (empty || !value.is_empty())
        && value.len() <= max
        && !value
            .chars()
            .any(|c| matches!(c as u32, 0..=31 | 127..=159))
}
