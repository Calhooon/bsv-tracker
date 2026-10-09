//! A pure transaction tracker following `Tracker.step` in the tracker charter.
//! The host supplies every input and executes every returned re-ask.

mod events;
mod evidence;
mod hints;
mod state;

pub use events::{decode_envelope, ChainEnvelope, ChainEvent, DecodeError};
pub use evidence::{CheckError, CheckedProof, Header, HeaderError, Headers, Proof};
pub use hints::{Hint, HintStatus, Verdict};
pub use state::{Evidence, HostAction, Input, Mined, NodeVerdict, Params, Reask, State, Word};

/// The height named by a BRC-74 path.
pub type Height = u32;
/// Seconds supplied by the host's clock.
pub type Timestamp = u64;
/// A display order transaction identifier.
pub type TxId = String;
/// A display order block hash or merkle root.
pub type Hash = String;
