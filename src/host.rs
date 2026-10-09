use crate::{CheckError, Hint, Proof, Reask, Timestamp, TxId};

/// The host's proof transport. The crate declares it but never calls it.
pub trait ProofFetcher {
    /// The transport's own error type.
    type Error;
    /// Execute a scheduled re-ask, with no proof represented explicitly.
    /// Competitor hints name the txids the host must also reconcile.
    fn fetch(&mut self, txid: &str, reask: &Reask) -> Result<Option<Proof>, Self::Error>;
}

/// The host's callbacks, SSE listener, polls or other hint inputs.
pub trait HintSource {
    /// The source's own error type.
    type Error;
    /// Supply the next named observation, or none. The tracker owns no loop.
    fn next_hint(&mut self) -> Result<Option<(TxId, Hint)>, Self::Error>;
}

/// The host's time source. The tracker never reads a platform clock.
pub trait Clock {
    /// Seconds in the same time domain used for hints and age thresholds.
    fn now(&self) -> Timestamp;
}

/// An error surface a host may use when rechecking before a spend.
pub type SpendCheck = Result<(), CheckError>;
