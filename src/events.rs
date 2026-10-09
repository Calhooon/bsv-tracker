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
    pub version: u32,
    pub previous_hash: Hash,
    pub merkle_root: Hash,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
    pub height: Height,
    pub hash: Hash,
    pub chain_work: String,
}

/// A frozen output's display-order txid and output index.
/// \[SRC\] rust-chaintracks@a62f9ed src/events.rs:43-48.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outpoint {
    pub txid: Hash,
    pub vout: u32,
}

/// The six #32 events, serialized and deserialized with `v: 1` and `kind`.
/// \[SRC\] rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:24-35.
/// For typed host faults, use [`decode_envelope`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainEvent {
    /// A new active tip. Only suspect transactions need a recheck.
    Tip {
        height: Height,
        hash: Hash,
        time: u32,
        header: EventHeader,
    },
    /// A fork that has not resolved.
    Fork {
        height: Height,
        competing_tips: [EventHeader; 2],
        depth: u32,
    },
    /// Blocks at or above a fork point have been replaced.
    Reorg {
        fork_height: Height,
        depth: u32,
        deactivated_headers: Vec<EventHeader>,
        new_tip: EventHeader,
    },
    /// The alert system invalidated a block.
    Invalidated { block_hash: Hash },
    /// A frozen output. The wallet owns its meaning; it moves no tracker word.
    Frozen { outpoint: Outpoint },
    /// Tip age. The header snapshot implements the fail closed response.
    TipAge { seconds: u64, tip: EventHeader },
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
}

impl Serialize for ChainEvent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct EnvelopeRef<'a> {
            v: u64,
            #[serde(flatten, with = "EventFields")]
            event: &'a ChainEvent,
        }
        EnvelopeRef { v: 1, event: self }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ChainEvent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        decode_value(value).map_err(serde::de::Error::custom)
    }
}

/// The flat version 1 chain-event envelope from rust-chaintracks@a62f9ed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChainEnvelope {
    /// The envelope version; currently exactly 1.
    v: u64,
    /// The event's kind and fields, flattened beside the version.
    #[serde(flatten, with = "EventFields")]
    event: ChainEvent,
}

impl ChainEnvelope {
    /// Wrap an event in the supported version.
    pub fn new(event: ChainEvent) -> Self {
        Self { v: 1, event }
    }
    /// The validated wire version.
    pub fn version(&self) -> u64 {
        self.v
    }
    /// The decoded event.
    pub fn event(&self) -> &ChainEvent {
        &self.event
    }
    /// Consume the envelope to deliver its decoded event.
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
        version: Option<u64>,
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
    if v != 1 {
        return Err(DecodeError::UnknownVersion(v));
    }
    value
        .as_object_mut()
        .expect("v was read from an object")
        .remove("v");
    let kind = value
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| DecodeError::UnknownShape {
            version: Some(v),
            detail: "missing or malformed kind".into(),
        })?;
    if !matches!(
        kind,
        "tip" | "fork" | "reorg" | "invalidated" | "frozen" | "tipAge"
    ) {
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
// src/events.rs:125-185. The tracker still obtains evidence via Headers::check.
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
    }
}

fn valid_header(header: &EventHeader) -> bool {
    header.height <= i32::MAX as u32
        && valid_hash(&header.hash)
        && valid_hash(&header.previous_hash)
        && valid_hash(&header.merkle_root)
        && valid_hash(&header.chain_work)
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}
