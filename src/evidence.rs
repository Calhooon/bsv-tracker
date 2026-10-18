use std::fmt;

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

/// A failed evidence check. An error is never a transaction word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    /// An invalid txid or BRC-74 path.
    InvalidProof(String),
    /// The host could not answer from a fresh verified active chain.
    Headers(HeaderError),
    /// The snapshot has no active header at this height.
    Unavailable(Height),
    /// The computed root differs from the active header's root.
    RootMismatch(Height),
    /// A proof belongs to a different transaction.
    TxidMismatch,
}
impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CheckError {}

/// An owned, structurally validated SDK path bound to the transaction it proves.
#[derive(Clone, Debug)]
pub struct Proof {
    txid: TxId,
    path: MerklePath,
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
    /// Bind the path to its txid, revalidating public SDK fields before reduction.
    ///
    /// # Errors
    ///
    /// [`CheckError::InvalidProof`] when the txid is not 32 bytes of hex, the
    /// path fails bsv-rs's validation, or the path does not reduce for the txid.
    pub fn new(txid: impl Into<TxId>, path: MerklePath) -> Result<Self, CheckError> {
        let txid = txid.into().to_ascii_lowercase();
        if txid.len() != 64 || !txid.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(CheckError::InvalidProof(
                "txid must be 32 bytes of hex".into(),
            ));
        }
        let path = MerklePath::new_unchecked(path.block_height, path.path)
            .map_err(|e| CheckError::InvalidProof(e.to_string()))?;
        path.compute_root(Some(&txid))
            .map_err(|e| CheckError::InvalidProof(e.to_string()))?;
        Ok(Self { txid, path })
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

/// A capability produced only by [`Headers::check`]. There is no unchecked
/// constructor or deserializer; the retained proof and header are immutable.
///
/// ```compile_fail
/// use bsv_tracker::{CheckedProof, Header, Proof};
/// use bsv_rs::transaction::MerklePath;
/// let txid = "01".repeat(32);
/// let proof = Proof::new(&txid, MerklePath::from_coinbase_txid(&txid, 10)).unwrap();
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
    /// The path whose root was checked.
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
    /// Tracker transitions always invoke this default implementation through
    /// a snapshot wrapper, so host overrides cannot bypass current lookups.
    ///
    /// # Errors
    ///
    /// [`CheckError::InvalidProof`] when the path does not reduce,
    /// [`CheckError::Headers`] when a lookup fails, [`CheckError::Unavailable`]
    /// when the snapshot has no active header at the height or its tip is
    /// below it, and [`CheckError::RootMismatch`] when the roots differ.
    fn check(&self, proof: Proof) -> Result<CheckedProof, CheckError> {
        let root = proof.root()?;
        let height = proof.height();
        let header = self
            .header_at(height)
            .map_err(CheckError::Headers)?
            .ok_or(CheckError::Unavailable(height))?;
        let tip = self.tip_height().map_err(CheckError::Headers)?;
        if tip < height {
            return Err(CheckError::Unavailable(height));
        }
        if header.merkle_root != root {
            return Err(CheckError::RootMismatch(height));
        }
        Ok(CheckedProof {
            proof,
            header,
            root,
            depth: u64::from(tip) - u64::from(height) + 1,
        })
    }
}

pub(crate) fn check_snapshot<H: Headers + ?Sized>(
    headers: &H,
    proof: Proof,
) -> Result<CheckedProof, CheckError> {
    struct Snapshot<'a, H: ?Sized>(&'a H);

    impl<H: Headers + ?Sized> Headers for Snapshot<'_, H> {
        fn header_at(&self, height: Height) -> Result<Option<Header>, HeaderError> {
            self.0.header_at(height)
        }

        fn tip_height(&self) -> Result<Height, HeaderError> {
            self.0.tip_height()
        }
    }

    Headers::check(&Snapshot(headers), proof)
}
