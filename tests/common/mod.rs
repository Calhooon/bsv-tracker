#![allow(dead_code)]
use bsv_rs::transaction::{MerklePath, MerklePathLeaf};
use bsv_tracker::*;
use std::collections::BTreeMap;

pub fn hash(n: u64) -> String {
    format!("{n:064x}")
}
pub fn proof(txid: &str, height: u32, sibling: u64) -> Proof {
    let path = MerklePath::new(
        height,
        vec![vec![
            MerklePathLeaf::new_txid(0, txid.into()),
            MerklePathLeaf::new(1, hash(sibling)),
        ]],
    )
    .unwrap();
    Proof::new(txid, path).unwrap()
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
    Params { age_threshold: 60 }
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
