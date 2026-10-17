use std::fmt;

use crate::{Hash, Height};
use serde::{Deserialize, Serialize};

/// The eight public header fields and cumulative work in the #32 envelope.
/// \[SRC\] rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:13-23;
/// src/events.rs:13-25 fixes the Rust integer widths.
/// These are wire data, never a checked inclusion capability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventHeader {
    /// The block version.
    pub version: u32,
    /// The display order hash of the parent block.
    pub previous_hash: Hash,
    /// The display order merkle root the header commits to.
    pub merkle_root: Hash,
    /// The block timestamp, in seconds.
    pub time: u32,
    /// The compact difficulty target.
    pub bits: u32,
    /// The proof of work nonce.
    pub nonce: u32,
    /// The block's height on the emitter's chain.
    pub height: Height,
    /// The display order hash of this header.
    pub hash: Hash,
    /// The cumulative chain work at this header, as the emitter's string.
    pub chain_work: String,
}

/// A frozen output's display-order txid and output index.
/// \[SRC\] rust-chaintracks@a62f9ed src/events.rs:43-48.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outpoint {
    /// The display order txid of the transaction that created the output.
    pub txid: Hash,
    /// The output index within that transaction.
    pub vout: u32,
}

/// One tip of a split, as the header service observed it over its rungs.
/// \[SRC\] chaintracks-cloudflare@v0.4.1 src/events.rs:50-57, docs/SPLITS.md:75-92.
/// These are wire data: neither `chain_work` nor `rungs` is checked here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SplitTip {
    /// The display order hash of this branch's witness tip.
    pub hash: Hash,
    /// The witness tip's height, at or above the split's `fork_height`.
    pub height: Height,
    /// The cumulative chain work at the tip, as the emitter's string.
    pub chain_work: String,
    /// The rung names reporting exactly this tip, sorted and non-empty.
    pub rungs: Vec<String>,
}

/// The six #32 events of version 1 and the two split events of version 2,
/// serialized and deserialized with `v` and `kind`; each kind selects its
/// version. \[SRC\] rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:24-35;
/// chaintracks-cloudflare@v0.4.1 src/events.rs:59-108, docs/SPLITS.md:68-83.
/// For typed host faults, use [`decode_envelope`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainEvent {
    /// A new active tip. Only suspect transactions need a recheck.
    Tip {
        /// The new tip's height.
        height: Height,
        /// The new tip's display order hash.
        hash: Hash,
        /// The new tip's block timestamp, in seconds.
        time: u32,
        /// The new tip's header.
        header: EventHeader,
    },
    /// A fork that has not resolved.
    Fork {
        /// The height at which the two branches diverge.
        height: Height,
        /// The tips of the two competing branches.
        competing_tips: [EventHeader; 2],
        /// The fork's depth, in blocks, as the emitter reports it.
        depth: u32,
    },
    /// Blocks at or above a fork point have been replaced.
    Reorg {
        /// The lowest height whose block was replaced.
        fork_height: Height,
        /// The reorg's depth, in blocks, as the emitter reports it.
        depth: u32,
        /// The headers that left the active chain.
        deactivated_headers: Vec<EventHeader>,
        /// The active tip after the reorg.
        new_tip: EventHeader,
    },
    /// The alert system invalidated a block.
    Invalidated {
        /// The display order hash of the invalidated block.
        block_hash: Hash,
    },
    /// A frozen output. The wallet owns its meaning; it moves no tracker word.
    Frozen {
        /// The frozen output.
        outpoint: Outpoint,
    },
    /// Tip age. The header snapshot implements the fail closed response.
    TipAge {
        /// The tip's age, in seconds, as the emitter measured it.
        seconds: u64,
        /// The tip whose age is reported.
        tip: EventHeader,
    },
    /// Version 2: two rungs follow different branches past the split depth.
    /// An observation, not a reorg: it changes no tip and writes no word.
    Split {
        /// `split:` and the common ancestor's display order hash.
        key: String,
        /// The lowest affected height, one above the common ancestor.
        fork_height: Height,
        /// A witness tip on each branch, with the rungs reporting it.
        tips: [SplitTip; 2],
    },
    /// Version 2: no pair of rungs still straddles the split with this key.
    SplitResolved {
        /// The key of the split that resolved.
        key: String,
        /// The split's lowest affected height.
        fork_height: Height,
    },
}

impl ChainEvent {
    /// The envelope version this kind is carried in: 2 for the split kinds,
    /// 1 for the six original kinds. \[SRC\] chaintracks-cloudflare@v0.4.1
    /// docs/SPLITS.md:68-73.
    #[must_use]
    pub fn version(&self) -> u64 {
        match self {
            Self::Split { .. } | Self::SplitResolved { .. } => 2,
            _ => 1,
        }
    }
}

// Remote serde derives keep the unversioned body private. Both public decode
// paths pass through decode_value before a ChainEvent can be delivered.
#[derive(Serialize, Deserialize)]
#[serde(
    remote = "ChainEvent",
    tag = "kind",
    rename_all = "camelCase",
    deny_unknown_fields
)]
enum EventFields {
    Tip {
        height: Height,
        hash: Hash,
        time: u32,
        header: EventHeader,
    },
    Fork {
        height: Height,
        #[serde(rename = "competingTips")]
        competing_tips: [EventHeader; 2],
        depth: u32,
    },
    Reorg {
        #[serde(rename = "forkHeight")]
        fork_height: Height,
        depth: u32,
        #[serde(rename = "deactivatedHeaders")]
        deactivated_headers: Vec<EventHeader>,
        #[serde(rename = "newTip")]
        new_tip: EventHeader,
    },
    Invalidated {
        #[serde(rename = "blockHash")]
        block_hash: Hash,
    },
    Frozen {
        outpoint: Outpoint,
    },
    TipAge {
        seconds: u64,
        tip: EventHeader,
    },
    Split {
        key: String,
        #[serde(rename = "forkHeight")]
        fork_height: Height,
        tips: [SplitTip; 2],
    },
    SplitResolved {
        key: String,
        #[serde(rename = "forkHeight")]
        fork_height: Height,
    },
}

impl Serialize for ChainEvent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct EnvelopeRef<'a> {
            v: u64,
            #[serde(flatten, with = "EventFields")]
            event: &'a ChainEvent,
        }
        EnvelopeRef {
            v: self.version(),
            event: self,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ChainEvent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        decode_value(value).map_err(serde::de::Error::custom)
    }
}

/// The flat chain-event envelope: version 1 from rust-chaintracks@a62f9ed,
/// version 2 from chaintracks-cloudflare@v0.4.1.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChainEnvelope {
    /// The envelope version: 1, or 2 for the split kinds.
    v: u64,
    /// The event's kind and fields, flattened beside the version.
    #[serde(flatten, with = "EventFields")]
    event: ChainEvent,
}

impl ChainEnvelope {
    /// Wrap an event in the version its kind selects.
    #[must_use]
    pub fn new(event: ChainEvent) -> Self {
        Self {
            v: event.version(),
            event,
        }
    }
    /// The validated wire version.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.v
    }
    /// The decoded event.
    #[must_use]
    pub fn event(&self) -> &ChainEvent {
        &self.event
    }
    /// Consume the envelope to deliver its decoded event.
    #[must_use]
    pub fn into_event(self) -> ChainEvent {
        self.event
    }
}

impl<'de> Deserialize<'de> for ChainEnvelope {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        ChainEvent::deserialize(deserializer).map(Self::new)
    }
}

/// A fault the host must report, without feeding an event to the tracker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// A missing, malformed or unrecognized body.
    UnknownShape {
        /// The envelope's version, when it was read before the fault.
        version: Option<u64>,
        /// What was wrong with the body.
        detail: String,
    },
    /// The sender uses a version this client does not know.
    UnknownVersion(u64),
    /// The sender uses an event kind this client does not know.
    UnknownKind(String),
}
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DecodeError {}

/// Pure decoding, with the version gated before the shape, independent of JSON
/// key order. Unknown fields, kinds and versions are reported errors.
///
/// # Errors
///
/// [`DecodeError::UnknownVersion`] when `v` is a number other than 1 or 2,
/// [`DecodeError::UnknownKind`] for an event kind its version does not carry
/// (the six original kinds are version 1 only, the two split kinds version 2
/// only),
/// and [`DecodeError::UnknownShape`] for bytes that are not JSON, a missing or
/// malformed `v` or `kind`, an unknown field, or fields that are inconsistent
/// with each other.
pub fn decode_envelope(bytes: &[u8]) -> Result<ChainEvent, DecodeError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| DecodeError::UnknownShape {
            version: None,
            detail: e.to_string(),
        })?;
    decode_value(value)
}

fn decode_value(mut value: serde_json::Value) -> Result<ChainEvent, DecodeError> {
    let version = value.get("v").and_then(serde_json::Value::as_u64);
    let Some(v) = version else {
        return Err(DecodeError::UnknownShape {
            version,
            detail: "missing or malformed v".into(),
        });
    };
    if !matches!(v, 1 | 2) {
        return Err(DecodeError::UnknownVersion(v));
    }
    // `v` was read through `get`, so the value is an object.
    if let Some(object) = value.as_object_mut() {
        object.remove("v");
    }
    let kind = value
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| DecodeError::UnknownShape {
            version: Some(v),
            detail: "missing or malformed kind".into(),
        })?;
    // Each version carries exactly its own kinds. The emitter's reader also
    // admits a version-1 kind under `v: 2` (chaintracks-cloudflare@v0.4.1
    // src/events.rs:139-146); this gate is stricter, since the emitter writes
    // the six original kinds as version 1 only (docs/SPLITS.md:116).
    let known = match v {
        1 => matches!(
            kind,
            "tip" | "fork" | "reorg" | "invalidated" | "frozen" | "tipAge"
        ),
        _ => matches!(kind, "split" | "splitResolved"),
    };
    if !known {
        return Err(DecodeError::UnknownKind(kind.into()));
    }
    let event = EventFields::deserialize(value).map_err(|e| DecodeError::UnknownShape {
        version: Some(v),
        detail: e.to_string(),
    })?;
    if !valid_event(&event) {
        return Err(DecodeError::UnknownShape {
            version: Some(v),
            detail: "inconsistent chain event".into(),
        });
    }
    Ok(event)
}

// Match the emitter's shape/relationship checks, without establishing PoW,
// active-chain membership or inclusion. [SRC] rust-chaintracks@a62f9ed
// src/events.rs:125-185; the split kinds chaintracks-cloudflare@v0.4.1
// src/events.rs:157-161,210-232. The tracker still obtains evidence via
// Headers::check.
fn valid_event(event: &ChainEvent) -> bool {
    match event {
        ChainEvent::Tip {
            height,
            hash,
            time,
            header,
        } => {
            valid_header(header)
                && *height == header.height
                && *hash == header.hash
                && *time == header.time
        }
        ChainEvent::Fork {
            height,
            competing_tips,
            depth,
        } => {
            *depth > 0
                && competing_tips.iter().all(valid_header)
                && competing_tips[0].hash != competing_tips[1].hash
                && competing_tips[0]
                    .height
                    .checked_sub(*depth)
                    .and_then(|h| h.checked_add(1))
                    == Some(*height)
        }
        ChainEvent::Reorg {
            fork_height,
            depth,
            deactivated_headers,
            new_tip,
        } => {
            *depth > 0
                && *depth <= 400
                && deactivated_headers.len() == *depth as usize
                && valid_header(new_tip)
                && deactivated_headers.iter().all(valid_header)
                && deactivated_headers
                    .last()
                    .is_some_and(|h| h.height == *fork_height)
                && deactivated_headers
                    .windows(2)
                    .all(|p| p[0].height == p[1].height + 1 && p[0].previous_hash == p[1].hash)
        }
        ChainEvent::Invalidated { block_hash } => valid_hash(block_hash),
        ChainEvent::Frozen { outpoint } => valid_hash(&outpoint.txid),
        ChainEvent::TipAge { tip, .. } => valid_header(tip),
        // Two distinct tips at or above the fork height, each with non-empty,
        // strictly sorted, non-empty rung names, no rung on both tips.
        // [SRC] chaintracks-cloudflare@v0.4.1 src/events.rs:210-231.
        ChainEvent::Split {
            key,
            fork_height,
            tips,
        } => {
            valid_split_key(key, *fork_height)
                && tips.iter().all(|tip| valid_split_tip(tip, *fork_height))
                && tips[0].hash != tips[1].hash
                && tips[0]
                    .rungs
                    .iter()
                    .all(|name| !tips[1].rungs.contains(name))
        }
        // [SRC] chaintracks-cloudflare@v0.4.1 src/events.rs:232.
        ChainEvent::SplitResolved { key, fork_height } => valid_split_key(key, *fork_height),
    }
}

// `split:` and the common ancestor's hash, above a non-zero fork height.
// [SRC] chaintracks-cloudflare@v0.4.1 src/events.rs:157-161, docs/SPLITS.md:88.
fn valid_split_key(key: &str, fork_height: Height) -> bool {
    fork_height > 0
        && i32::try_from(fork_height).is_ok()
        && key.strip_prefix("split:").is_some_and(valid_hash)
}

// [SRC] chaintracks-cloudflare@v0.4.1 src/events.rs:217-225.
fn valid_split_tip(tip: &SplitTip, fork_height: Height) -> bool {
    valid_hash(&tip.hash)
        && valid_hash(&tip.chain_work)
        && tip.height >= fork_height
        && i32::try_from(tip.height).is_ok()
        && !tip.rungs.is_empty()
        && tip.rungs.iter().all(|name| !name.is_empty())
        && tip.rungs.windows(2).all(|names| names[0] < names[1])
}

fn valid_header(header: &EventHeader) -> bool {
    i32::try_from(header.height).is_ok()
        && valid_hash(&header.hash)
        && valid_hash(&header.previous_hash)
        && valid_hash(&header.merkle_root)
        && valid_hash(&header.chain_work)
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}
