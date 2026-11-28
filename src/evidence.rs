use std::fmt;

use bsv_rs::primitives::hash::sha256d;
use bsv_rs::transaction::MerklePath;

use crate::{Hash, Height, TxId};

/// The active header projection retained with a proof, as in `Tracker.Header`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    /// The block's display order hash.
    pub hash: Hash,
    /// The header's display order merkle root.
    pub merkle_root: Hash,
}

/// A fault reported by the host's header snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderError(pub String);

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for HeaderError {}

/// The one length, in bytes, at which a merkle path proves nothing: a
/// transaction of exactly 64 bytes is never taken as proven by a merkle path
/// (BRC-237's condition for an anchor; bsv-rs enforces it in its own
/// verifier, its public advisory GHSA-jjhc-7hvh-c3hm).
///
/// A host and a test read this constant, never the literal: a state whose
/// [`State::tx_len`](crate::State::tx_len) is `Some(AMBIGUOUS_TX_LEN)` takes
/// no proof, so its host asks none for it.
///
/// Ahead of the Lean: bsv-stack-lean `lean/Tracker.lean` at `9a727a64` holds
/// no such constant yet; the rule's words are its
/// `docs/DECISIONS-2026-10-19.md` 32.13.
pub const AMBIGUOUS_TX_LEN: u64 = 64;

/// The txid of serialized transaction bytes and their count: the double
/// SHA-256 of the bytes, reversed, as 64 lowercase hex digits (display
/// order), and the length in bytes as a `u64` (never a `usize`: the stored
/// form must not depend on the host's word size). The bytes are hashed and
/// counted, never decoded.
pub(crate) fn digest(tx_bytes: &[u8]) -> (TxId, u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let hash = sha256d(tx_bytes);
    let mut txid = String::with_capacity(64);
    for b in hash.iter().rev() {
        txid.push(char::from(HEX[usize::from(b >> 4)]));
        txid.push(char::from(HEX[usize::from(b & 0xf)]));
    }
    (txid, tx_bytes.len() as u64)
}

/// A failed evidence check. An error is never a transaction word.
///
/// `#[non_exhaustive]`: a later release may add a fault, so a host's match
/// outside this crate names the faults it reads and carries a wildcard arm.
/// A fault a host does not name still means "not `mined`", so the wildcard
/// arm is safe here, where [`Word`](crate::Word) stays exhaustive on purpose.
/// A match over the five faults of 0.9.0-beta.11 no longer compiles.
///
/// ```compile_fail,E0004
/// use bsv_tracker::CheckError;
/// fn kind(fault: &CheckError) -> &'static str {
///     match fault {
///         CheckError::InvalidProof(_) => "invalid",
///         CheckError::Headers(_) => "headers",
///         CheckError::Unavailable(_) => "unavailable",
///         CheckError::RootMismatch(_) => "root",
///         CheckError::TxidMismatch => "txid",
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckError {
    /// An invalid txid or BRC-74 path.
    InvalidProof(String),
    /// The host could not answer from a fresh verified active chain.
    Headers(HeaderError),
    /// The snapshot has no active header at this height.
    Unavailable(Height),
    /// The computed root differs from the active header's root.
    RootMismatch(Height),
    /// A proof belongs to a different transaction; or, from
    /// [`State::measure`](crate::State::measure) and
    /// [`Tracker::track_bytes`](crate::Tracker::track_bytes), the bytes are
    /// another transaction's.
    TxidMismatch,
    /// The transaction is exactly [`AMBIGUOUS_TX_LEN`] bytes: a transaction of
    /// exactly 64 bytes is never taken as proven by a merkle path. Answered by
    /// [`Proof::new`] on 64 bytes and [`Proof::with_stated_len`] on a stated
    /// 64, which yield no proof; by the proof door of a state measured at
    /// that length, which takes no proof and is left whole; by
    /// [`Tracker::restore`](crate::Tracker::restore) when the state it holds
    /// is measured at that length and the restored row, which carries no
    /// length, holds a proof, which the call has then unsaid; inside
    /// [`CompactError::Evidence`](crate::CompactError::Evidence) for stored
    /// bytes that hold that length beside a proof; and by
    /// [`State::measure`](crate::State::measure) when the measure it has just
    /// taken is of that length and the state held a proof, which the call
    /// has then unsaid.
    AmbiguousLength,
    /// Two lengths for one transaction: the state's own measure, and the
    /// length a proof or a second measure brought. The state's word and
    /// record are kept: a statement is checked against the measure, never
    /// trusted over it.
    LengthMismatch {
        /// The length the state holds, measured from bytes.
        measured: u64,
        /// The length that was offered against it.
        offered: u64,
    },
    /// A door that needs the transaction's length was offered a proof with
    /// none it can read: a proof taken out of a state's record
    /// ([`CheckedProof::proof`]) carries no length of its own, so
    /// [`Headers::check`], [`doubts::may_enter`](crate::doubts::may_enter)
    /// and the proof door of an unmeasured state refuse it; and the
    /// competitor door refuses a proof not built from the competitor's bytes
    /// ([`Proof::new`]).
    Unmeasured,
}
impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CheckError {}

/// The length a proof was built on: measured from the bytes the crate
/// hashed, or stated by the host. Never [`AMBIGUOUS_TX_LEN`]: no constructor
/// yields a proof at that length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Len {
    /// The count of the bytes [`Proof::new`] hashed to the proof's txid.
    Measured(u64),
    /// The length the host stated to [`Proof::with_stated_len`].
    Stated(u64),
}

/// An owned, structurally validated SDK path bound to the transaction it proves.
///
/// A transaction of exactly 64 bytes is never taken as proven by a merkle
/// path (BRC-237's condition for an anchor; bsv-rs enforces it in its own
/// verifier, its public advisory GHSA-jjhc-7hvh-c3hm). The crate holds the
/// path and the txid, so it is told the length at the constructor: from the
/// transaction's bytes ([`Proof::new`]), or as the host's word
/// ([`Proof::with_stated_len`]). Neither yields a proof at
/// [`AMBIGUOUS_TX_LEN`].
///
/// Two proofs are equal when their txid, height and path are: the length a
/// proof was built on is read at a door and is no part of the comparison.
#[derive(Clone, Debug)]
pub struct Proof {
    txid: TxId,
    path: MerklePath,
    len: Option<Len>,
}
impl PartialEq for Proof {
    fn eq(&self, other: &Self) -> bool {
        self.txid == other.txid
            && self.path.block_height == other.path.block_height
            && self.path.path == other.path.path
    }
}
impl Eq for Proof {}

impl Proof {
    /// Bind the path to the transaction whose serialized bytes these are:
    /// the txid is their hash (the double SHA-256, reversed, as 64 lowercase
    /// hex digits) and the proof carries their count. The bytes are hashed
    /// and counted, never decoded: bytes in another form (an extended format,
    /// a BEEF) give another txid, for which the path does not reduce.
    ///
    /// A transaction of exactly 64 bytes is never taken as proven by a merkle
    /// path: 64 bytes yield no proof, whatever the path.
    ///
    /// A state offered this proof for its txid is measured by it, if it was
    /// not already ([`State::tx_len`](crate::State::tx_len)); a measured state
    /// refuses it when the counts differ.
    ///
    /// ```
    /// use bsv_rs::transaction::{MerklePath, MerklePathLeaf};
    /// use bsv_tracker::{CheckError, Proof, State};
    /// let raw_tx = [0x5a_u8; 191];
    /// let txid = State::measured(&raw_tx).txid().to_owned();
    /// let leaves = vec![MerklePathLeaf::new_txid(0, txid.clone()), MerklePathLeaf::new(1, "bb".repeat(32))];
    /// let path = MerklePath::new(100, vec![leaves]).unwrap();
    /// let proof = Proof::new(&raw_tx, path.clone()).unwrap();
    /// assert_eq!(proof.txid(), txid);
    /// assert_eq!(Proof::new(&[0x5a_u8; 64], path), Err(CheckError::AmbiguousLength));
    /// ```
    ///
    /// # Errors
    ///
    /// [`CheckError::AmbiguousLength`] for exactly 64 bytes, before anything
    /// is hashed; [`CheckError::InvalidProof`] when the path fails bsv-rs's
    /// validation or does not reduce for the txid of the bytes.
    pub fn new(tx_bytes: &[u8], path: MerklePath) -> Result<Self, CheckError> {
        if tx_bytes.len() as u64 == AMBIGUOUS_TX_LEN {
            return Err(CheckError::AmbiguousLength);
        }
        let (txid, len) = digest(tx_bytes);
        let (txid, path) = bind(&txid, path)?;
        Ok(Self {
            txid,
            path,
            len: Some(Len::Measured(len)),
        })
    }
    /// Bind the path to a txid on a length the host states: the road of a
    /// host with no transaction bytes at the place where it builds the proof.
    /// The crate cannot check a statement; the trust is the caller's, and a
    /// host with no length has no proof. A state that holds its own measure
    /// takes this proof only when the stated length equals it.
    ///
    /// A transaction of exactly 64 bytes is never taken as proven by a merkle
    /// path: a stated length of 64 yields no proof, whatever the path. No
    /// other number is refused.
    ///
    /// # Errors
    ///
    /// [`CheckError::AmbiguousLength`] for a stated length of
    /// [`AMBIGUOUS_TX_LEN`], before anything else is read;
    /// [`CheckError::InvalidProof`] when the txid is not 32 bytes of hex, the
    /// path fails bsv-rs's validation, or the path does not reduce for the
    /// txid.
    pub fn with_stated_len(
        txid: impl Into<TxId>,
        stated_len: u64,
        path: MerklePath,
    ) -> Result<Self, CheckError> {
        if stated_len == AMBIGUOUS_TX_LEN {
            return Err(CheckError::AmbiguousLength);
        }
        let (txid, path) = bind(&txid.into(), path)?;
        Ok(Self {
            txid,
            path,
            len: Some(Len::Stated(stated_len)),
        })
    }
    /// The proof a record keeps and a restore reads back: no length of its
    /// own, since the state's measure, kept beside it, is the length.
    pub(crate) fn record(txid: &str, path: MerklePath) -> Result<Self, CheckError> {
        let (txid, path) = bind(txid, path)?;
        Ok(Self {
            txid,
            path,
            len: None,
        })
    }
    /// The length this proof was built on; `None` for a record's proof.
    pub(crate) fn length(&self) -> Option<Len> {
        self.len
    }
    /// The same proof on another length, for the unit tests of the arms no
    /// public constructor reaches.
    #[cfg(test)]
    pub(crate) fn on_length(self, len: Option<Len>) -> Self {
        Self { len, ..self }
    }
    /// The transaction whose inclusion is being checked.
    #[must_use]
    pub fn txid(&self) -> &str {
        &self.txid
    }
    /// The height named by the path.
    #[must_use]
    pub fn height(&self) -> Height {
        self.path.block_height
    }
    /// The SDK path, borrowed without allowing mutation.
    #[must_use]
    pub fn path(&self) -> &MerklePath {
        &self.path
    }
    /// Compute the root for the bound txid through bsv-rs.
    ///
    /// # Errors
    ///
    /// [`CheckError::InvalidProof`] when bsv-rs cannot reduce the path.
    pub fn root(&self) -> Result<Hash, CheckError> {
        self.path
            .compute_root(Some(&self.txid))
            .map_err(|e| CheckError::InvalidProof(e.to_string()))
    }
}

/// The txid's form and the path's reduction, shared by every constructor of
/// [`Proof`]: the txid lowercased and 32 bytes of hex, the path revalidated
/// through bsv-rs and reduced for the txid.
fn bind(txid: &str, path: MerklePath) -> Result<(TxId, MerklePath), CheckError> {
    let txid = txid.to_ascii_lowercase();
    if txid.len() != 64 || !txid.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(CheckError::InvalidProof(
            "txid must be 32 bytes of hex".into(),
        ));
    }
    let path = MerklePath::new_unchecked(path.block_height, path.path)
        .map_err(|e| CheckError::InvalidProof(e.to_string()))?;
    path.compute_root(Some(&txid))
        .map_err(|e| CheckError::InvalidProof(e.to_string()))?;
    Ok((txid, path))
}

/// A capability produced only by [`Headers::check`]. There is no unchecked
/// constructor or deserializer; the retained proof and header are immutable.
///
/// ```compile_fail,E0451
/// use bsv_tracker::{CheckedProof, Header, Proof};
/// use bsv_rs::transaction::MerklePath;
/// let txid = "01".repeat(32);
/// let proof = Proof::with_stated_len(&txid, 191, MerklePath::from_coinbase_txid(&txid, 10)).unwrap();
/// let header = Header { hash: "02".repeat(32), merkle_root: txid.clone() };
/// let _ = CheckedProof { proof, header, root: txid, depth: 1 };
/// ```
///
/// ```compile_fail
/// use bsv_tracker::CheckedProof;
/// let _ = serde_json::from_str::<CheckedProof>("{}");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedProof {
    proof: Proof,
    header: Header,
    root: Hash,
    depth: u64,
}
impl CheckedProof {
    /// The path whose root was checked. It carries no length of its own:
    /// the length is the state's measure, kept beside the record
    /// ([`State::tx_len`](crate::State::tx_len)). So a state measured from
    /// bytes takes it again, and [`Headers::check`],
    /// [`doubts::may_enter`](crate::doubts::may_enter) and an unmeasured
    /// state refuse it ([`CheckError::Unmeasured`]).
    #[must_use]
    pub fn proof(&self) -> &Proof {
        &self.proof
    }
    /// The active header used at the moment of the check.
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }
    /// The computed root, equal to the retained header's root.
    #[must_use]
    pub fn root(&self) -> &str {
        &self.root
    }
    /// The proof's height.
    #[must_use]
    pub fn height(&self) -> Height {
        self.proof.height()
    }
    /// Confirmations at the snapshot that checked this proof.
    #[must_use]
    pub fn depth(&self) -> u64 {
        self.depth
    }
    /// A stored record, held together but not checked against any headers:
    /// only [`State::from_compact`](crate::State::from_compact) calls this,
    /// and it marks the record's mined word suspect with a recheck due.
    ///
    /// The proof it holds carries no length ([`Proof::record`]); the state's
    /// [`State::tx_len`](crate::State::tx_len), restored beside it, is the
    /// measure. An unmeasured state's record was taken on a length its host
    /// stated, or, in a version 1 row, before the crate measured a
    /// transaction.
    pub(crate) fn restored(
        proof: Proof,
        header: Header,
        root: Hash,
        depth: u64,
    ) -> Result<Self, CheckError> {
        let height = proof.height();
        if proof.root()? != root || header.merkle_root != root {
            return Err(CheckError::RootMismatch(height));
        }
        if depth == 0 {
            return Err(CheckError::InvalidProof("depth must be at least 1".into()));
        }
        Ok(Self {
            proof,
            header,
            root,
            depth,
        })
    }
}

/// A host supplied snapshot of fresh, verified active headers. Both methods
/// must answer from the same snapshot and fail closed when it is unavailable.
pub trait Headers {
    /// Look up an active header, or no answer while behind or unverified.
    ///
    /// # Errors
    ///
    /// A [`HeaderError`] when the host cannot answer from a fresh verified
    /// active chain.
    fn header_at(&self, height: Height) -> Result<Option<Header>, HeaderError>;
    /// The active height from the same snapshot.
    ///
    /// # Errors
    ///
    /// A [`HeaderError`] when the snapshot is unavailable.
    fn tip_height(&self) -> Result<Height, HeaderError>;
    /// Reduce a bound SDK proof and check it against the active header.
    /// Tracker transitions never call this method: they run the same check
    /// on `header_at` and `tip_height` alone, so host overrides cannot bypass
    /// current lookups.
    ///
    /// A transaction of exactly 64 bytes is never taken as proven by a merkle
    /// path. This door holds no state, so it reads the proof's own length: a
    /// proof built by [`Proof::new`] or [`Proof::with_stated_len`] (never at
    /// 64 bytes) is checked, and a proof taken out of a record, which carries
    /// none, is refused.
    ///
    /// # Errors
    ///
    /// [`CheckError::Unmeasured`] for a proof with no length,
    /// [`CheckError::InvalidProof`] when the path does not reduce,
    /// [`CheckError::Headers`] when a lookup fails, [`CheckError::Unavailable`]
    /// when the snapshot has no active header at the height or its tip is
    /// below it, and [`CheckError::RootMismatch`] when the roots differ.
    fn check(&self, proof: Proof) -> Result<CheckedProof, CheckError> {
        if proof.length().is_none() {
            return Err(CheckError::Unmeasured);
        }
        checked(self, proof)
    }
}

/// The comparison: the proof's root against the active header's root at its
/// height, read from `header_at` and `tip_height` alone. The record it builds
/// keeps the proof with no length: the measure is the state's.
fn checked<H: Headers + ?Sized>(headers: &H, proof: Proof) -> Result<CheckedProof, CheckError> {
    let root = proof.root()?;
    let height = proof.height();
    let header = headers
        .header_at(height)
        .map_err(CheckError::Headers)?
        .ok_or(CheckError::Unavailable(height))?;
    let tip = headers.tip_height().map_err(CheckError::Headers)?;
    if tip < height {
        return Err(CheckError::Unavailable(height));
    }
    if header.merkle_root != root {
        return Err(CheckError::RootMismatch(height));
    }
    Ok(CheckedProof {
        proof: Proof { len: None, ..proof },
        header,
        root,
        depth: u64::from(tip) - u64::from(height) + 1,
    })
}

/// The check every transition runs, after the state has judged the proof's
/// length: never the host's override of [`Headers::check`].
pub(crate) fn check_snapshot<H: Headers + ?Sized>(
    headers: &H,
    proof: Proof,
) -> Result<CheckedProof, CheckError> {
    checked(headers, proof)
}
