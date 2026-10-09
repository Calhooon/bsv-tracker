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
                merkle_root: p.root().unwrap(),
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
    Input::Hint(Hint {
        source: "stub broadcaster".into(),
        status,
        observed,
    })
}
pub fn reorg(height: u32) -> ChainEvent {
    ChainEvent::Reorg {
        fork_height: height,
        depth: 1,
        deactivated: vec![hash(100)],
        new_tip: hash(200),
    }
}
pub fn mined(s: &State, height: u32) {
    assert_eq!(s.word().height(), Some(height), "word: {:?}", s.word());
    let evidence = s.evidence().unwrap();
    assert_eq!(evidence.height(), height);
    assert_eq!(
        evidence.proof().root().unwrap(),
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
