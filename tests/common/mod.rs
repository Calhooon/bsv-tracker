#![allow(dead_code)]
use bsv_rs::transaction::{MerklePath, MerklePathLeaf};
use bsv_tracker::*;
use std::collections::BTreeMap;

pub fn hash(n: u64) -> String {
    format!("{n:064x}")
}
/// A length that is not 64, for a transaction the tests track by a made-up
/// txid: the length the helper `proof` states.
pub const STATED_LEN: u64 = 191;
/// The path of one level: the txid at 0, `hash(sibling)` at 1.
pub fn path_of(txid: &str, height: u32, sibling: u64) -> MerklePath {
    MerklePath::new(
        height,
        vec![vec![
            MerklePathLeaf::new_txid(0, txid.into()),
            MerklePathLeaf::new(1, hash(sibling)),
        ]],
    )
    .unwrap()
}
/// A proof for a txid nobody has bytes for, so the stated road, at
/// `STATED_LEN`.
pub fn proof(txid: &str, height: u32, sibling: u64) -> Proof {
    Proof::with_stated_len(txid, STATED_LEN, path_of(txid, height, sibling)).unwrap()
}
/// `len` bytes that differ for every `n`: `n` big-endian in the leading
/// eight, then 0x5a.
pub fn tx_bytes(n: u64, len: usize) -> Vec<u8> {
    let mut bytes = vec![0x5a; len];
    let lead = n.to_be_bytes();
    let k = len.min(8);
    bytes[..k].copy_from_slice(&lead[..k]);
    bytes
}
/// The txid of transaction bytes, as the crate computes it.
pub fn txid_of(tx_bytes: &[u8]) -> String {
    State::measured(tx_bytes).txid().to_owned()
}
/// A proof built from transaction bytes: the measured road.
pub fn proof_of(tx_bytes: &[u8], height: u32, sibling: u64) -> Proof {
    Proof::new(tx_bytes, path_of(&txid_of(tx_bytes), height, sibling)).unwrap()
}
/// A competitor's proof, built from the competitor's own bytes
/// (`tx_bytes(n, 191)`), as the competitor door asks.
pub fn competitor(n: u64, height: u32, sibling: u64) -> Proof {
    proof_of(&tx_bytes(n, 191), height, sibling)
}
#[derive(Clone, Debug, Default)]
pub struct TestHeaders {
    pub tip: u32,
    pub headers: BTreeMap<u32, Header>,
    pub fault: bool,
}
impl TestHeaders {
    pub fn insert(&mut self, p: &Proof, block: u64) {
        self.tip = self.tip.max(p.height());
        self.headers.insert(
            p.height(),
            Header {
                hash: hash(block),
                merkle_root: p.path().compute_root(Some(p.txid())).unwrap(),
            },
        );
    }
    /// The new branch's header at `height`: block `block`, a root no tracked
    /// proof reduces to, so a word mined there before has had its root moved.
    pub fn move_root(&mut self, height: u32, block: u64) {
        self.tip = self.tip.max(height);
        self.headers.insert(
            height,
            Header {
                hash: hash(block),
                merkle_root: hash(block + 5_000),
            },
        );
    }
}
impl Headers for TestHeaders {
    fn header_at(&self, height: u32) -> Result<Option<Header>, HeaderError> {
        if self.fault {
            Err(HeaderError("snapshot unavailable".into()))
        } else {
            Ok(self.headers.get(&height).cloned())
        }
    }
    fn tip_height(&self) -> Result<u32, HeaderError> {
        if self.fault {
            Err(HeaderError("tip unavailable".into()))
        } else {
            Ok(self.tip)
        }
    }
}
pub fn params() -> Params {
    Params::new(60, 1)
}
pub fn step(s: &mut State, h: &TestHeaders, i: Input) {
    s.step(&params(), h, i).unwrap();
}
pub fn hint(status: HintStatus, observed: u64) -> Input {
    Input::Hint(Hint::new("stub broadcaster", status, observed))
}
// Synthetic wire metadata; checked inclusion still uses TestHeaders and Proof.
pub fn event_header(height: u32, block: u64) -> EventHeader {
    EventHeader {
        version: 1,
        previous_hash: hash(block.saturating_sub(1)),
        merkle_root: hash(block + 1_000),
        time: 1_757_280_000 + height,
        bits: 545_259_519,
        nonce: 0,
        height,
        hash: hash(block),
        chain_work: hash(u64::from(height) + 1),
    }
}
pub fn tip(height: u32, block: u64) -> ChainEvent {
    let header = event_header(height, block);
    ChainEvent::Tip {
        height,
        hash: header.hash.clone(),
        time: header.time,
        header,
    }
}
pub fn fork(height: u32) -> ChainEvent {
    ChainEvent::Fork {
        height,
        competing_tips: [event_header(height, 90), event_header(height, 91)],
        depth: 1,
    }
}
pub fn reorg(height: u32) -> ChainEvent {
    ChainEvent::Reorg {
        fork_height: height,
        depth: 1,
        deactivated_headers: vec![event_header(height, 100)],
        new_tip: event_header(height + 1, 200),
    }
}
// Synthetic version-2 wire data, keyed by the common ancestor's hash.
pub fn split(fork_height: u32) -> ChainEvent {
    let tip = |block: u64, rung: &str| SplitTip {
        hash: hash(block),
        height: fork_height + 2,
        chain_work: hash(u64::from(fork_height) + 3),
        rungs: vec![rung.into()],
    };
    ChainEvent::Split {
        key: format!("split:{}", hash(u64::from(fork_height) - 1)),
        fork_height,
        tips: [tip(300, "woc"), tip(301, "arcade")],
    }
}
pub fn split_resolved(fork_height: u32) -> ChainEvent {
    ChainEvent::SplitResolved {
        key: format!("split:{}", hash(u64::from(fork_height) - 1)),
        fork_height,
    }
}
// The rows of the doubts' reducer, mirrors of the helpers of bsv-stack-lean's
// lean/Headers/Doubts.lean section 8 at 7213ef4a (`tipRow`, `forkRow`,
// `reorgRow`) and of the split rows of lean/Headers/Host.lean section 11. They
// build values and pass no decoder; the reducer reads their heights alone.
/// The Lean's `tipRow h`.
pub fn tip_row(h: u32) -> ChainEvent {
    tip(h, u64::from(h))
}
/// The Lean's `forkRow f t n`: forking at `f`, the served tip at `t`, then
/// the competitor at `n`.
pub fn fork_row(f: u32, t: u32, n: u32) -> ChainEvent {
    ChainEvent::Fork {
        height: f,
        competing_tips: [
            event_header(t, u64::from(t)),
            event_header(n, u64::from(n) + 10_000),
        ],
        depth: (t + 1).saturating_sub(f),
    }
}
/// The Lean's `reorgRow fh old nt`: the headers from `old` down to `fh`
/// removed, the new tip at `nt`.
pub fn reorg_row(fh: u32, old: u32, nt: u32) -> ChainEvent {
    ChainEvent::Reorg {
        fork_height: fh,
        depth: (old + 1).saturating_sub(fh),
        deactivated_headers: (fh..=old)
            .rev()
            .map(|h| event_header(h, u64::from(h)))
            .collect(),
        new_tip: event_header(nt, u64::from(nt) + 20_000),
    }
}
/// The Lean's `.reorg fh 0 [] (evAt nt)`: a reorg that names no removed
/// header. Built by hand: the decoder refuses a depth of 0.
pub fn reorg_none(fh: u32, nt: u32) -> ChainEvent {
    ChainEvent::Reorg {
        fork_height: fh,
        depth: 0,
        deactivated_headers: Vec::new(),
        new_tip: event_header(nt, u64::from(nt) + 20_000),
    }
}
/// A split at fork height `fh` with witness tips at `h1` and `h2`. The key
/// is `split:` and `hash(fh)`, which no rule of the doubts reads.
pub fn split_row(fh: u32, h1: u32, h2: u32) -> ChainEvent {
    let tip = |block: u64, height: u32, rung: &str| SplitTip {
        hash: hash(block),
        height,
        chain_work: hash(u64::from(height) + 1),
        rungs: vec![rung.into()],
    };
    ChainEvent::Split {
        key: format!("split:{}", hash(u64::from(fh))),
        fork_height: fh,
        tips: [tip(300, h1, "woc"), tip(301, h2, "arcade")],
    }
}
/// The `splitResolved` row at fork height `fh`, under `split_row`'s key.
pub fn resolved_row(fh: u32) -> ChainEvent {
    ChainEvent::SplitResolved {
        key: format!("split:{}", hash(u64::from(fh))),
        fork_height: fh,
    }
}
/// The Lean's `splitRow k fh lo hi` under the key `k` (`split:k` on the
/// wire): the witness tips at `lo` and `hi`.
pub fn keyed_split_row(k: &str, fh: u32, lo: u32, hi: u32) -> ChainEvent {
    let tip = |block: u64, height: u32, rung: &str| SplitTip {
        hash: hash(block),
        height,
        chain_work: hash(u64::from(height) + 1),
        rungs: vec![rung.into()],
    };
    ChainEvent::Split {
        key: format!("split:{k}"),
        fork_height: fh,
        tips: [tip(300, lo, "r1"), tip(301, hi, "r2")],
    }
}
/// The Lean's `.splitResolved k fh` under the key `k` (`split:k` on the wire).
pub fn keyed_resolved_row(k: &str, fh: u32) -> ChainEvent {
    ChainEvent::SplitResolved {
        key: format!("split:{k}"),
        fork_height: fh,
    }
}
pub fn mined(s: &State, height: u32) {
    assert_eq!(s.word().height(), Some(height), "word: {:?}", s.word());
    let evidence = s.evidence().unwrap();
    assert_eq!(evidence.height(), height);
    assert_eq!(
        evidence
            .proof()
            .path()
            .compute_root(Some(s.txid()))
            .unwrap(),
        evidence.header().merkle_root
    );
}
pub fn announce(s: &mut State, h: &TestHeaders) {
    assert_eq!(s.word(), &Word::Unknown);
    step(s, h, Input::Host(HostAction::Build));
    assert_eq!(s.word(), &Word::Built);
    step(s, h, hint(HintStatus::Accepted, 0));
    assert_eq!(s.word(), &Word::Announced);
    step(s, h, hint(HintStatus::Seen, 1));
    assert_eq!(s.word(), &Word::Seen);
}
