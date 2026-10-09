use crate::{Height, Timestamp, TxId};

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
    /// Interpret a status as a hint; no result is a node verdict or a proof.
    pub fn into_hint(self, source: impl Into<String>, observed: Timestamp) -> Hint {
        let status = match self.status.to_ascii_uppercase().as_str() {
            "RECEIVED"
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
                competitors: self.competitors,
            },
            "REJECTED" | "INVALID" | "MALFORMED" => HintStatus::Rejected {
                reason: self.reason.unwrap_or(self.status),
            },
            _ => HintStatus::Unknown,
        };
        Hint {
            source: source.into(),
            status,
            observed,
        }
    }
}
