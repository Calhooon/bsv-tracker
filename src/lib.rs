#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod events;
mod evidence;
mod hints;
mod host;
mod state;
mod tracker;

pub use events::{decode_envelope, ChainEnvelope, ChainEvent, DecodeError, EventHeader, Outpoint};
pub use evidence::{CheckError, CheckedProof, Header, HeaderError, Headers, Proof};
pub use hints::{Hint, HintStatus, Verdict, VerdictError};
pub use host::{Clock, HintSource, ProofFetcher, SpendCheck};
pub use state::{Evidence, HostAction, Input, Mined, NodeVerdict, Params, Reask, State, Word};
pub use tracker::{ChainUpdate, Tracker};

/// The height named by a BRC-74 path.
pub type Height = u32;
/// Seconds supplied by the host's clock.
pub type Timestamp = u64;
/// A display order transaction identifier.
pub type TxId = String;
/// A display order block hash or merkle root.
pub type Hash = String;
