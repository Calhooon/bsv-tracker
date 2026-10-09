use crate::{Height, Timestamp, TxId};
use serde::Deserialize;
use std::fmt;

/// A broadcaster's claim reduced to the Lean hint vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HintStatus {
    Accepted,
    Seen,
    Mined { height: Height },
    StaleBlock,
    Rejected { reason: String },
    DoubleSpend { competitors: Vec<TxId> },
    OrphanMempool,
    Unknown,
}
/// An observed hint, carrying its source and host supplied time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    pub source: String,
    pub status: HintStatus,
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub status: String,
    pub height: Option<Height>,
    pub reason: Option<String>,
    pub competitors: Vec<TxId>,
}
impl Verdict {
    /// Read an ARC body as metadata, checking its echoed txid and the bounded
    /// status vocabulary at bsv-rs@7bc623c. HTTP status is never consulted.
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
        let known = matches!(
            body.tx_status.to_ascii_uppercase().as_str(),
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
            status: body.tx_status,
            height: body.block_height,
            reason: body.extra_info,
            competitors,
        })
    }
    /// Interpret a status as a hint; no result is a node verdict or a proof.
    pub fn into_hint(self, source: impl Into<String>, observed: Timestamp) -> Hint {
        let status = match self.status.to_ascii_uppercase().as_str() {
            "SUCCESS"
            | "SENT_TO_NETWORK"
            | "RECEIVED"
            | "QUEUED"
            | "STORED"
            | "ANNOUNCED_TO_NETWORK"
            | "REQUESTED_BY_NETWORK"
            | "ACCEPTED_BY_NETWORK" => HintStatus::Accepted,
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
        let status = if self
            .reason
            .as_deref()
            .is_some_and(|r| r.eq_ignore_ascii_case("reorg_unmined"))
        {
            HintStatus::StaleBlock
        } else if self.status.to_ascii_uppercase().contains("ORPHAN")
            || self
                .reason
                .as_deref()
                .is_some_and(|r| r.to_ascii_uppercase().contains("ORPHAN"))
        {
            HintStatus::OrphanMempool
        } else if !self.competitors.is_empty()
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
