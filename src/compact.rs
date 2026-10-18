//! The compact byte form of one [`State`](crate::State) (bsv-stack-lean #92).
//!
//! Version 1, every field in order:
//!
//! | field | encoding |
//! | --- | --- |
//! | version | one byte, `1` |
//! | txid | an id |
//! | word | a tag byte (`0` unknown, `1` built, `2` announced, `3` seen, `4` mined, `5` stale, `6` rejected and its reason, `7` conflicted and its ids, `8` abandoned) |
//! | evidence | `0`, or `1` and the checked proof's txid (an id), path, header hash and root (ids), root (an id) and depth (a varint) |
//! | suspect | `0` or `1` |
//! | hints | a varint count, then each hint, the newest at the front |
//! | last hint time | `0`, or `1` and a varint |
//! | re-ask | `0`, or `1` and a tag byte (`0` reorg, `1` fork, `2` hint, `3` proof failed, `4` recheck, `5` age, `6` spend) and its payload |
//!
//! A varint is unsigned LEB128 in its shortest form. A string is a varint
//! length and its UTF-8 bytes. An id is `0` and 32 raw bytes when the text is
//! 64 lowercase hex digits, else `1` and a string. A hint is its source (a
//! string), its status (a tag byte in [`HintStatus`] declaration order and its
//! payload), its observed time (a varint) and its verdict (`0`, or `1` and the
//! status, the optional height, the optional reason and the competitor ids).
//! A path is its block height, a varint count of levels, and per level a
//! varint count of leaves, each an offset (a varint), a flag byte (`1`
//! duplicate, `2` txid, `4` a hash follows) and the hash (an id).
//!
//! The decoder accepts only the bytes the encoder writes: a non-canonical
//! varint, an id written as text that has the raw form, a flag outside its set
//! or trailing bytes are each refused.

use crate::{
    CheckError, CheckedProof, Header, Height, Hint, HintStatus, Proof, Reask, Timestamp, TxId,
    Verdict, Word,
};
use bsv_rs::transaction::{MerklePath, MerklePathLeaf};
use std::fmt;

/// The version byte [`State::to_compact`](crate::State::to_compact) writes.
pub const COMPACT_VERSION: u8 = 1;

/// Bytes [`State::from_compact`](crate::State::from_compact) refuses. A
/// refusal restores nothing; the host re-tracks the transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompactError {
    /// The version byte is not one this crate reads.
    UnknownVersion(u8),
    /// The bytes end inside a field.
    Truncated,
    /// Bytes remain after the last field.
    TrailingBytes,
    /// A tag, flag, varint, id or string the encoder never writes.
    Malformed(&'static str),
    /// The stored proof record does not hold together: its path does not
    /// reduce, it names another transaction, or its root differs from its
    /// header's. Nothing here consults the host's headers.
    Evidence(CheckError),
}
impl fmt::Display for CompactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CompactError {}

/// The fields of a state, as the codec reads and writes them.
pub(crate) struct Parts {
    pub txid: TxId,
    /// None is the mined word, backed by the evidence field.
    pub word: Option<Word>,
    pub evidence: Option<CheckedProof>,
    pub suspect: bool,
    pub hints: Vec<Hint>,
    pub last_hint_at: Option<Timestamp>,
    pub reask: Option<Reask>,
}

pub(crate) struct Fields<'a> {
    pub txid: &'a str,
    pub word: &'a Word,
    pub evidence: Option<&'a CheckedProof>,
    pub suspect: bool,
    pub hints: &'a [Hint],
    pub last_hint_at: Option<Timestamp>,
    pub reask: Option<&'a Reask>,
}

pub(crate) fn encode(fields: &Fields<'_>) -> Vec<u8> {
    let mut w = Writer(vec![COMPACT_VERSION]);
    w.id(fields.txid);
    w.word(fields.word);
    match fields.evidence {
        None => w.byte(0),
        Some(checked) => {
            w.byte(1);
            w.id(checked.proof().txid());
            w.path(checked.proof().path());
            w.id(&checked.header().hash);
            w.id(&checked.header().merkle_root);
            w.id(checked.root());
            w.varint(checked.depth());
        }
    }
    w.flag(fields.suspect);
    w.len(fields.hints.len());
    for hint in fields.hints {
        w.hint(hint);
    }
    w.option(fields.last_hint_at, Writer::varint);
    match fields.reask {
        None => w.byte(0),
        Some(reask) => {
            w.byte(1);
            w.reask(reask);
        }
    }
    w.0
}

pub(crate) fn decode(bytes: &[u8]) -> Result<Parts, CompactError> {
    let mut r = Reader(bytes);
    let version = r.byte()?;
    if version != COMPACT_VERSION {
        return Err(CompactError::UnknownVersion(version));
    }
    let txid = r.id()?;
    if txid != txid.to_ascii_lowercase() {
        return Err(CompactError::Malformed("txid not lowercase"));
    }
    let word = r.word()?;
    let evidence = if r.flag()? {
        let proof_txid = r.id()?;
        let path = r.path()?;
        let header = Header {
            hash: r.id()?,
            merkle_root: r.id()?,
        };
        let root = r.id()?;
        let depth = r.varint()?;
        let proof = Proof::new(proof_txid, path).map_err(CompactError::Evidence)?;
        if proof.txid() != txid {
            return Err(CompactError::Evidence(CheckError::TxidMismatch));
        }
        Some(CheckedProof::restored(proof, header, root, depth).map_err(CompactError::Evidence)?)
    } else {
        None
    };
    let suspect = r.flag()?;
    let count = r.count()?;
    let mut hints = Vec::with_capacity(count);
    for _ in 0..count {
        hints.push(r.hint()?);
    }
    let last_hint_at = r.option(Reader::varint)?;
    let reask = r.option(Reader::reask)?;
    if !r.0.is_empty() {
        return Err(CompactError::TrailingBytes);
    }
    Ok(Parts {
        txid,
        word,
        evidence,
        suspect,
        hints,
        last_hint_at,
        reask,
    })
}

struct Writer(Vec<u8>);
impl Writer {
    fn byte(&mut self, b: u8) {
        self.0.push(b);
    }
    fn flag(&mut self, b: bool) {
        self.byte(u8::from(b));
    }
    fn varint(&mut self, mut n: u64) {
        loop {
            // The low seven bits; the mask makes the truncation exact.
            #[allow(clippy::cast_possible_truncation)]
            let low = (n & 0x7f) as u8;
            n >>= 7;
            if n == 0 {
                self.byte(low);
                return;
            }
            self.byte(low | 0x80);
        }
    }
    fn len(&mut self, n: usize) {
        self.varint(n as u64);
    }
    fn height(&mut self, h: Height) {
        self.varint(u64::from(h));
    }
    fn string(&mut self, s: &str) {
        self.len(s.len());
        self.0.extend_from_slice(s.as_bytes());
    }
    fn id(&mut self, s: &str) {
        if let Some(raw) = raw_id(s) {
            self.byte(0);
            self.0.extend_from_slice(&raw);
        } else {
            self.byte(1);
            self.string(s);
        }
    }
    fn ids(&mut self, ids: &[TxId]) {
        self.len(ids.len());
        for id in ids {
            self.id(id);
        }
    }
    fn option<T>(&mut self, value: Option<T>, put: impl FnOnce(&mut Self, T)) {
        match value {
            None => self.byte(0),
            Some(v) => {
                self.byte(1);
                put(self, v);
            }
        }
    }
    fn word(&mut self, word: &Word) {
        match word {
            Word::Unknown => self.byte(0),
            Word::Built => self.byte(1),
            Word::Announced => self.byte(2),
            Word::Seen => self.byte(3),
            // The checked proof is the evidence field; a mined word shares it.
            Word::Mined(_) => self.byte(4),
            Word::Stale => self.byte(5),
            Word::Rejected { reason } => {
                self.byte(6);
                self.string(reason);
            }
            Word::Conflicted { competitors } => {
                self.byte(7);
                self.ids(competitors);
            }
            Word::Abandoned => self.byte(8),
        }
    }
    fn path(&mut self, path: &MerklePath) {
        self.height(path.block_height);
        self.len(path.path.len());
        for level in &path.path {
            self.len(level.len());
            for leaf in level {
                self.varint(leaf.offset);
                let flags = u8::from(leaf.duplicate)
                    | (u8::from(leaf.txid) << 1)
                    | (u8::from(leaf.hash.is_some()) << 2);
                self.byte(flags);
                if let Some(hash) = &leaf.hash {
                    self.id(hash);
                }
            }
        }
    }
    fn status(&mut self, status: &HintStatus) {
        match status {
            HintStatus::Accepted => self.byte(0),
            HintStatus::Seen => self.byte(1),
            HintStatus::Mined { height } => {
                self.byte(2);
                self.height(*height);
            }
            HintStatus::StaleBlock => self.byte(3),
            HintStatus::Rejected { reason } => {
                self.byte(4);
                self.string(reason);
            }
            HintStatus::DoubleSpend { competitors } => {
                self.byte(5);
                self.ids(competitors);
            }
            HintStatus::OrphanMempool => self.byte(6),
            HintStatus::Unknown => self.byte(7),
            HintStatus::MempoolPresent { source } => {
                self.byte(8);
                self.string(source);
            }
        }
    }
    fn hint(&mut self, hint: &Hint) {
        self.string(&hint.source);
        self.status(&hint.status);
        self.varint(hint.observed);
        self.option(hint.verdict.as_ref(), |w, v| {
            w.string(&v.status);
            w.option(v.height, Self::height);
            w.option(v.reason.as_deref(), Self::string);
            w.ids(&v.competitors);
        });
    }
    fn reask(&mut self, reask: &Reask) {
        match reask {
            Reask::Reorg { fork_height } => {
                self.byte(0);
                self.height(*fork_height);
            }
            Reask::Fork { height } => {
                self.byte(1);
                self.height(*height);
            }
            Reask::Hint(hint) => {
                self.byte(2);
                self.hint(hint);
            }
            Reask::ProofFailed { height } => {
                self.byte(3);
                self.height(*height);
            }
            Reask::Recheck => self.byte(4),
            Reask::Age => self.byte(5),
            Reask::Spend => self.byte(6),
        }
    }
}

struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], CompactError> {
        if n > self.0.len() {
            return Err(CompactError::Truncated);
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }
    fn byte(&mut self) -> Result<u8, CompactError> {
        let (&b, rest) = self.0.split_first().ok_or(CompactError::Truncated)?;
        self.0 = rest;
        Ok(b)
    }
    fn flag(&mut self) -> Result<bool, CompactError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CompactError::Malformed("flag")),
        }
    }
    fn varint(&mut self) -> Result<u64, CompactError> {
        let mut n = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.byte()?;
            let low = u64::from(b & 0x7f);
            if shift == 63 && low > 1 {
                return Err(CompactError::Malformed("varint overflow"));
            }
            n |= low << shift;
            if b & 0x80 == 0 {
                if b == 0 && shift > 0 {
                    return Err(CompactError::Malformed("varint not shortest"));
                }
                return Ok(n);
            }
        }
        Err(CompactError::Malformed("varint overflow"))
    }
    fn height(&mut self) -> Result<Height, CompactError> {
        Height::try_from(self.varint()?).map_err(|_| CompactError::Malformed("height"))
    }
    /// A count of items, each at least one byte, so a count beyond the
    /// remaining bytes is refused before anything is allocated.
    fn count(&mut self) -> Result<usize, CompactError> {
        let n = usize::try_from(self.varint()?).map_err(|_| CompactError::Truncated)?;
        if n > self.0.len() {
            return Err(CompactError::Truncated);
        }
        Ok(n)
    }
    fn string(&mut self) -> Result<String, CompactError> {
        let n = self.count()?;
        let bytes = self.take(n)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| CompactError::Malformed("utf-8"))
    }
    fn id(&mut self) -> Result<String, CompactError> {
        match self.byte()? {
            0 => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let mut s = String::with_capacity(64);
                for b in self.take(32)? {
                    s.push(char::from(HEX[usize::from(b >> 4)]));
                    s.push(char::from(HEX[usize::from(b & 0xf)]));
                }
                Ok(s)
            }
            1 => {
                let s = self.string()?;
                if raw_id(&s).is_some() {
                    return Err(CompactError::Malformed("id not in its raw form"));
                }
                Ok(s)
            }
            _ => Err(CompactError::Malformed("id tag")),
        }
    }
    fn ids(&mut self) -> Result<Vec<TxId>, CompactError> {
        let n = self.count()?;
        (0..n).map(|_| self.id()).collect()
    }
    fn option<T>(
        &mut self,
        get: impl FnOnce(&mut Self) -> Result<T, CompactError>,
    ) -> Result<Option<T>, CompactError> {
        if self.flag()? {
            get(self).map(Some)
        } else {
            Ok(None)
        }
    }
    /// None is the mined word; the state attaches the evidence field's proof.
    fn word(&mut self) -> Result<Option<Word>, CompactError> {
        Ok(Some(match self.byte()? {
            0 => Word::Unknown,
            1 => Word::Built,
            2 => Word::Announced,
            3 => Word::Seen,
            4 => return Ok(None),
            5 => Word::Stale,
            6 => Word::Rejected {
                reason: self.string()?,
            },
            7 => Word::Conflicted {
                competitors: self.ids()?,
            },
            8 => Word::Abandoned,
            _ => return Err(CompactError::Malformed("word tag")),
        }))
    }
    fn path(&mut self) -> Result<MerklePath, CompactError> {
        let block_height = self.height()?;
        let levels = self.count()?;
        let mut path = Vec::with_capacity(levels);
        for _ in 0..levels {
            let n = self.count()?;
            let mut level = Vec::with_capacity(n);
            for _ in 0..n {
                let offset = self.varint()?;
                let flags = self.byte()?;
                if flags & !0b111 != 0 {
                    return Err(CompactError::Malformed("leaf flags"));
                }
                let hash = if flags & 4 == 0 {
                    None
                } else {
                    Some(self.id()?)
                };
                level.push(MerklePathLeaf {
                    offset,
                    hash,
                    txid: flags & 2 != 0,
                    duplicate: flags & 1 != 0,
                });
            }
            path.push(level);
        }
        Ok(MerklePath { block_height, path })
    }
    fn status(&mut self) -> Result<HintStatus, CompactError> {
        Ok(match self.byte()? {
            0 => HintStatus::Accepted,
            1 => HintStatus::Seen,
            2 => HintStatus::Mined {
                height: self.height()?,
            },
            3 => HintStatus::StaleBlock,
            4 => HintStatus::Rejected {
                reason: self.string()?,
            },
            5 => HintStatus::DoubleSpend {
                competitors: self.ids()?,
            },
            6 => HintStatus::OrphanMempool,
            7 => HintStatus::Unknown,
            8 => HintStatus::MempoolPresent {
                source: self.string()?,
            },
            _ => return Err(CompactError::Malformed("hint status tag")),
        })
    }
    fn hint(&mut self) -> Result<Hint, CompactError> {
        let source = self.string()?;
        let status = self.status()?;
        let observed = self.varint()?;
        let verdict = self.option(|r| {
            Ok(Verdict {
                status: r.string()?,
                height: r.option(Self::height)?,
                reason: r.option(Self::string)?,
                competitors: r.ids()?,
            })
        })?;
        Ok(Hint {
            source,
            status,
            observed,
            verdict,
        })
    }
    fn reask(&mut self) -> Result<Reask, CompactError> {
        Ok(match self.byte()? {
            0 => Reask::Reorg {
                fork_height: self.height()?,
            },
            1 => Reask::Fork {
                height: self.height()?,
            },
            2 => Reask::Hint(self.hint()?),
            3 => Reask::ProofFailed {
                height: self.height()?,
            },
            4 => Reask::Recheck,
            5 => Reask::Age,
            6 => Reask::Spend,
            _ => return Err(CompactError::Malformed("re-ask tag")),
        })
    }
}

/// The 32 bytes of 64 lowercase hex digits, in text order.
fn raw_id(s: &str) -> Option<[u8; 32]> {
    fn nibble(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            _ => None,
        }
    }
    let bytes = s.as_bytes();
    if bytes.len() != 64 {
        return None;
    }
    let mut raw = [0u8; 32];
    for (out, pair) in raw.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *out = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Some(raw)
}
