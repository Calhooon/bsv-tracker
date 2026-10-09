use std::fmt;

use crate::{Hash, Height};
use serde::{Deserialize, Serialize};

/// The six evidence events in `Tracker.ChainEvent` and the assumed #32 shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ChainEvent {
    /// A new active tip. Only suspect transactions need a recheck.
    Tip { height: Height, hash: Hash },
    /// A fork that has not resolved.
    Fork {
        height: Height,
        #[serde(rename = "competingTips")]
        competing_tips: Vec<Hash>,
        depth: u32,
    },
    /// Blocks at or above a fork point have been replaced.
    Reorg {
        #[serde(rename = "forkHeight")]
        fork_height: Height,
        depth: u32,
        deactivated: Vec<Hash>,
        #[serde(rename = "newTip")]
        new_tip: Hash,
    },
    /// The alert system invalidated a block.
    Invalidated {
        #[serde(rename = "blockHash")]
        block_hash: Hash,
    },
    /// A frozen output. The wallet owns its meaning; it moves no tracker word.
    Frozen { outpoint: String },
    /// Tip age. The header snapshot implements the fail closed response.
    TipAge { seconds: u64 },
}

/// The assumed flat version 1 chain-event envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChainEnvelope {
    /// The envelope version; currently exactly 1.
    v: u64,
    /// The event's kind and fields, flattened beside the version.
    #[serde(flatten)]
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
        let value = serde_json::Value::deserialize(deserializer)?;
        decode_value(value)
            .map(Self::new)
            .map_err(serde::de::Error::custom)
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
    serde_json::from_value(value).map_err(|e| DecodeError::UnknownShape {
        version: Some(v),
        detail: e.to_string(),
    })
}
