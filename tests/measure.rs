//! The measure on the tracked state: a transaction of exactly 64 bytes is
//! never taken as proven by a merkle path (BRC-237's condition for an anchor;
//! bsv-rs enforces it in its own verifier, GHSA-jjhc-7hvh-c3hm). The crate
//! measures the transaction: a state born from bytes, or handed them later,
//! holds its length, and a state measured at 64 takes no proof.
//!
//! Each test is a table: a name, an input and the exact answer. Every proof
//! is built through the helper `proof(..)` of `tests/common/mod.rs`.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod common;
use bsv_tracker::*;
use common::*;

/// A length that is not 64, for the rows that show a measure off the rule:
/// the one length these tests give a transaction that a proof is taken for.
const OTHER_LEN: usize = 191;

/// `len` bytes that differ for every `n`: `n` big-endian in the leading
/// eight, then `0x5a`.
fn tx_bytes(n: u64, len: usize) -> Vec<u8> {
    let mut bytes = vec![0x5a; len];
    for (slot, b) in bytes.iter_mut().zip(n.to_be_bytes()) {
        *slot = b;
    }
    bytes
}

/// The txid the crate gives the bytes.
fn txid_of(tx_bytes: &[u8]) -> String {
    State::measured(tx_bytes).txid().to_owned()
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// The coinbase transaction of the chain's block 0, and its txid: a pin of
/// the hash and of its byte order that this crate did not compute.
const BLOCK_0_COINBASE: &str = concat!(
    "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d",
    "0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66",
    "207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe55",
    "48271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba",
    "0b8d578a4c702b6bf11d5fac00000000",
);
const BLOCK_0_COINBASE_TXID: &str =
    "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";

/// Every field a host can read but the measured length.
fn same_but_the_length(after: &State, before: &State, name: &str) {
    assert_eq!(after.txid(), before.txid(), "{name}");
    assert_eq!(after.word(), before.word(), "{name}");
    assert_eq!(after.evidence(), before.evidence(), "{name}");
    assert_eq!(after.suspect(), before.suspect(), "{name}");
    assert_eq!(after.hints(), before.hints(), "{name}");
    assert_eq!(after.last_hint_at(), before.last_hint_at(), "{name}");
    assert_eq!(after.reask(), before.reask(), "{name}");
    assert_eq!(after.handed_to(), before.handed_to(), "{name}");
}

/// A state born from `txid` alone and brought to the named word, with the
/// headers that took its proof where it has one. Each holds a re-ask.
fn reach(name: &str, txid: &str) -> (State, TestHeaders) {
    let mut h = TestHeaders::default();
    let mut s = State::new(txid);
    let verdict = |v| Input::Evidence(Evidence::Verdict(v));
    match name {
        "unknown" => {}
        "built" => step(&mut s, &h, Input::Host(HostAction::Build)),
        "handed" => step(
            &mut s,
            &h,
            Input::Host(HostAction::HandOver {
                instance: "arcade".into(),
                at: 5,
            }),
        ),
        "announced" => {
            step(&mut s, &h, Input::Host(HostAction::Build));
            step(&mut s, &h, hint(HintStatus::Accepted, 0));
        }
        "seen" => announce(&mut s, &h),
        "mined" | "stale" => {
            let p = proof(txid, 10, 2);
            h.insert(&p, 10);
            step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
            if name == "stale" {
                h.move_root(10, 77);
                step(&mut s, &h, Input::Evidence(Evidence::Recheck));
            } else {
                // Suspect, with its re-ask, so the test sees both cleared.
                step(&mut s, &h, Input::Evidence(Evidence::Chain(fork(10))));
                assert!(s.suspect());
            }
        }
        "rejected" => step(
            &mut s,
            &h,
            verdict(NodeVerdict::Rejected {
                reason: "node said no".into(),
            }),
        ),
        "conflicted" => step(
            &mut s,
            &h,
            verdict(NodeVerdict::Conflicted {
                competitors: vec![hash(9)],
            }),
        ),
        "abandoned" => step(&mut s, &h, Input::Host(HostAction::Abandon)),
        other => panic!("no such word: {other}"),
    }
    if !matches!(name, "mined" | "stale") {
        step(&mut s, &h, Input::Host(HostAction::SpendAttempt));
    }
    assert!(s.reask().is_some(), "{name}");
    assert_eq!(s.tx_len(), None, "{name}");
    (s, h)
}

const TEN_WORDS: [&str; 10] = [
    "unknown",
    "built",
    "handed",
    "announced",
    "seen",
    "mined",
    "stale",
    "rejected",
    "conflicted",
    "abandoned",
];

fn label(word: &Word) -> &'static str {
    match word {
        Word::Unknown => "unknown",
        Word::Built => "built",
        Word::Handed => "handed",
        Word::Announced => "announced",
        Word::Seen => "seen",
        Word::Mined(_) => "mined",
        Word::Stale => "stale",
        Word::Rejected { .. } => "rejected",
        Word::Conflicted { .. } => "conflicted",
        Word::Abandoned => "abandoned",
    }
}

#[test]
fn births() {
    // The hash and its byte order, against a txid this crate did not compute.
    let coinbase = unhex(BLOCK_0_COINBASE);
    let s = State::measured(&coinbase);
    assert_eq!(s.txid(), BLOCK_0_COINBASE_TXID);
    assert_eq!(s.tx_len(), Some(204));

    // Birth from a txid: unmeasured.
    let s = State::new(hash(1));
    assert_eq!(s.tx_len(), None);

    // Birth from bytes, at each length around the rule's: measured, and
    // otherwise the state a birth from the same txid gives. Never a failure.
    assert_eq!(AMBIGUOUS_TX_LEN, 64);
    for len in [0, 63, 64, 65, OTHER_LEN] {
        let bytes = tx_bytes(1, len);
        let s = State::measured(&bytes);
        assert_eq!(s.tx_len(), Some(len as u64), "{len}");
        assert_eq!(s.word(), &Word::Unknown, "{len}");
        assert_eq!(s.txid().len(), 64, "{len}");
        assert!(
            s.txid()
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
            "{len}"
        );
        let unmeasured = State::new(s.txid());
        same_but_the_length(&s, &unmeasured, "birth");
        assert_ne!(s, unmeasured, "{len}: the length is part of the state");
        assert_eq!(s.evidence(), None);
        assert_eq!(s.reask(), None);
    }
    // Bytes of one length that differ give two txids.
    assert_ne!(txid_of(&tx_bytes(1, 64)), txid_of(&tx_bytes(2, 64)));

    // The tracker, a txid not tracked: born measured.
    for len in [63, 64, 65] {
        let bytes = tx_bytes(1, len);
        let mut tracker = Tracker::new(params());
        let born = tracker.track_bytes(&bytes).unwrap().clone();
        assert_eq!(born, State::measured(&bytes), "{len}");
        assert_eq!(tracker.len(), 1);
        assert_eq!(tracker.get(born.txid()), Some(&born));
        // The same bytes again: `Ok`, the state equal to before.
        assert_eq!(tracker.track_bytes(&bytes), Ok(&born), "{len}");
        assert_eq!(tracker.len(), 1);
    }

    // The tracker, a txid tracked from `track`: it gains the measure and its
    // word is untouched.
    for len in [63, 64, 65] {
        let bytes = tx_bytes(1, len);
        let txid = txid_of(&bytes);
        let h = TestHeaders::default();
        let mut tracker = Tracker::new(params());
        assert_eq!(tracker.track(&txid).tx_len(), None);
        tracker
            .apply(&txid, &h, Input::Host(HostAction::Build))
            .unwrap();
        let before = tracker.get(&txid).unwrap().clone();
        let after = tracker.track_bytes(&bytes).unwrap().clone();
        assert_eq!(after.tx_len(), Some(len as u64), "{len}");
        assert_eq!(after.word(), &Word::Built, "{len}");
        same_but_the_length(&after, &before, "tracked");
        assert_eq!(tracker.track_bytes(&bytes), Ok(&after), "{len}");
        // `track` on it afterwards erases nothing.
        assert_eq!(tracker.track(&txid), &after, "{len}");
        assert_eq!(tracker.len(), 1);
    }
}

#[test]
fn measure_after_birth() {
    let bytes = tx_bytes(1, 64);
    let txid = txid_of(&bytes);

    // Bytes of another transaction, on every word: refused, the state whole.
    for name in TEN_WORDS {
        let (mut s, _) = reach(name, &txid);
        let before = s.clone();
        for other in [tx_bytes(2, 64), tx_bytes(2, OTHER_LEN), vec![]] {
            assert_eq!(s.measure(&other), Err(CheckError::TxidMismatch), "{name}");
            assert_eq!(s, before, "{name}");
        }
    }

    // 64 bytes on each of the ten words, each reached on a state born from
    // its txid.
    for name in TEN_WORDS {
        let (mut s, h) = reach(name, &txid);
        assert_eq!(label(s.word()), name);
        let before = s.clone();
        let answer = s.measure(&bytes);
        assert_eq!(s.tx_len(), Some(AMBIGUOUS_TX_LEN), "{name}");
        assert_eq!(s.evidence(), None, "{name}");
        if matches!(name, "mined" | "stale") {
            // The fault road: the fault is returned AND the word has moved.
            assert!(before.evidence().is_some(), "{name}");
            assert_eq!(answer, Err(CheckError::AmbiguousLength), "{name}");
            assert_eq!(s.word(), &Word::Unknown, "{name}");
            assert!(!s.suspect(), "{name}");
            assert_eq!(s.reask(), None, "{name}");
            assert_eq!(s.hints(), before.hints(), "{name}");
            assert_eq!(s.last_hint_at(), before.last_hint_at(), "{name}");
            assert_eq!(s.handed_to(), before.handed_to(), "{name}");
        } else {
            // No proof was held: the measure is taken and nothing else moves.
            assert_eq!(answer, Ok(()), "{name}");
            same_but_the_length(&s, &before, name);
            assert!(s.reask().is_some(), "{name}");
        }
        // The same bytes again: `Ok`, nothing moved.
        let measured = s.clone();
        assert_eq!(s.measure(&bytes), Ok(()), "{name}");
        assert_eq!(s, measured, "{name}");
        // And from here the proof door answers the rule and writes nothing.
        let p = proof(&txid, 10, 2);
        assert_eq!(
            s.step(&params(), &h, Input::Evidence(Evidence::Proof(p))),
            Err(CheckError::AmbiguousLength),
            "{name}"
        );
        assert_eq!(s, measured, "{name}");
        // The bytes hold the measure and restore it.
        assert_eq!(State::from_compact(&s.to_compact()).unwrap(), s, "{name}");
    }

    // A record beside a word no proof writes (bytes only a hand makes: a
    // stale row with its word tag set to `unknown`): the record is dropped,
    // the word and the re-ask are kept, and the fault is returned.
    let (stale, _) = reach("stale", &txid);
    let mut row = stale.to_compact();
    assert_eq!(row[34], 5);
    row[34] = 0;
    let mut s = State::from_compact(&row).unwrap();
    assert_eq!(s.word(), &Word::Unknown);
    assert!(s.evidence().is_some());
    let before = s.clone();
    assert_eq!(s.measure(&bytes), Err(CheckError::AmbiguousLength));
    assert_eq!(s.evidence(), None);
    assert_eq!(s.word(), before.word());
    assert_eq!(s.reask(), before.reask());
    assert_eq!(s.reask(), Some(&Reask::Reorg { fork_height: 10 }));
    assert_eq!(s.tx_len(), Some(AMBIGUOUS_TX_LEN));

    // A length that is not 64, on each of the ten words: `tx_len` is set and
    // nothing else moves. The `mined` word stands.
    let bytes = tx_bytes(1, OTHER_LEN);
    let txid = txid_of(&bytes);
    for name in TEN_WORDS {
        let (mut s, h) = reach(name, &txid);
        let before = s.clone();
        assert_eq!(s.measure(&bytes), Ok(()), "{name}");
        assert_eq!(s.tx_len(), Some(OTHER_LEN as u64), "{name}");
        same_but_the_length(&s, &before, name);
        assert_eq!(label(s.word()), name);
        let measured = s.clone();
        assert_eq!(s.measure(&bytes), Ok(()), "{name}");
        assert_eq!(s, measured, "{name}");
        if name == "mined" {
            mined(&s, 10);
            // The word still answers its recheck.
            step(&mut s, &h, Input::Evidence(Evidence::Recheck));
            mined(&s, 10);
            assert!(!s.suspect());
            assert_eq!(s.tx_len(), Some(OTHER_LEN as u64));
        }
    }
}

/// The fault road through the tracker: the fault is returned, the word has
/// moved, and the indexes followed it.
#[test]
fn a_measure_of_64_on_a_tracked_mined_word_returns_the_fault_and_moves_the_indexes() {
    let bytes = tx_bytes(1, 64);
    let txid = txid_of(&bytes);
    let mut h = TestHeaders::default();
    let p = proof(&txid, 10, 2);
    h.insert(&p, 10);
    let mut tracker = Tracker::new(params());
    tracker
        .apply(&txid, &h, Input::Evidence(Evidence::Proof(p)))
        .unwrap();
    mined(tracker.get(&txid).unwrap(), 10);
    // The mined index holds it: a fork at its height selects it.
    assert_eq!(tracker.clone().on_chain(&h, fork(10)).len(), 1);

    assert_eq!(
        tracker.track_bytes(&bytes),
        Err(CheckError::AmbiguousLength)
    );
    let s = tracker.get(&txid).unwrap();
    assert_eq!(s.word(), &Word::Unknown);
    assert_eq!(s.evidence(), None);
    assert_eq!(s.reask(), None);
    assert_eq!(s.tx_len(), Some(AMBIGUOUS_TX_LEN));
    assert_eq!(tracker.len(), 1);
    // No index holds it now: no chain event selects it.
    assert!(tracker.clone().on_chain(&h, fork(10)).is_empty());
    assert!(tracker.clone().on_chain(&h, reorg(10)).is_empty());
    assert!(tracker.clone().on_chain(&h, tip(11, 90)).is_empty());
    let block = h.headers[&10].hash.clone();
    assert!(tracker
        .on_chain(&h, ChainEvent::Invalidated { block_hash: block })
        .is_empty());
    // Handing the bytes again is quiet: the fault was the proof's unsaying.
    let after = tracker.get(&txid).unwrap().clone();
    assert_eq!(tracker.track_bytes(&bytes), Ok(&after));
}

/// A `mined` row of version 1 restores unmeasured and stays `mined`; a host
/// that later hands its bytes meets the rules of `State::measure`.
#[test]
fn a_mined_row_of_version_1_restores_unmeasured_and_stays_mined() {
    for (len, fault) in [(OTHER_LEN, false), (64, true)] {
        let bytes = tx_bytes(1, len);
        let txid = txid_of(&bytes);
        let mut h = TestHeaders::default();
        let p = proof(&txid, 10, 2);
        h.insert(&p, 10);
        let mut s = State::new(&txid);
        step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
        // Version 1 is version 2 without its last four fields.
        let v2 = s.to_compact();
        let mut v1 = v2[..v2.len() - 4].to_vec();
        v1[0] = 1;

        let mut restored = State::from_compact(&v1).unwrap();
        mined(&restored, 10);
        assert_eq!(restored.tx_len(), None, "{len}");
        assert!(restored.suspect());
        assert_eq!(restored.reask(), Some(&Reask::Recheck));
        // Nothing more is asked of it: its recheck clears it as before.
        step(&mut restored, &h, Input::Evidence(Evidence::Recheck));
        mined(&restored, 10);
        assert!(!restored.suspect());
        assert_eq!(restored.tx_len(), None, "{len}");

        let answer = restored.measure(&bytes);
        assert_eq!(restored.tx_len(), Some(len as u64));
        if fault {
            assert_eq!(answer, Err(CheckError::AmbiguousLength));
            assert_eq!(restored.word(), &Word::Unknown);
            assert_eq!(restored.evidence(), None);
        } else {
            assert_eq!(answer, Ok(()));
            mined(&restored, 10);
        }
    }
}

#[test]
fn a_state_measured_at_64_refuses_every_proof() {
    let bytes = tx_bytes(1, 64);
    let txid = txid_of(&bytes);
    let mut h = TestHeaders::default();
    let p = proof(&txid, 10, 2);
    // The headers carry the proof's root: only the length refuses it.
    h.insert(&p, 10);

    // On each word a state measured at 64 can hold.
    let roads: [(&str, &[Input]); 5] = [
        ("unknown", &[]),
        ("built", &[Input::Host(HostAction::Build)]),
        (
            "handed",
            &[Input::Host(HostAction::HandOver {
                instance: "arcade".into(),
                at: 5,
            })],
        ),
        (
            "announced",
            &[
                Input::Host(HostAction::Build),
                hint(HintStatus::Accepted, 0),
            ],
        ),
        (
            "seen",
            &[
                Input::Host(HostAction::Build),
                hint(HintStatus::Accepted, 0),
                hint(HintStatus::Seen, 1),
            ],
        ),
    ];
    for (name, road) in roads {
        let mut s = State::measured(&bytes);
        for input in road {
            step(&mut s, &h, input.clone());
        }
        assert_eq!(label(s.word()), name);
        let before = s.clone();
        // The same proof, another height and another sibling: each refused.
        for offered in [p.clone(), proof(&txid, 11, 2), proof(&txid, 10, 3)] {
            assert_eq!(
                s.step(&params(), &h, Input::Evidence(Evidence::Proof(offered))),
                Err(CheckError::AmbiguousLength),
                "{name}"
            );
            assert_eq!(s, before, "{name}: the state is equal to its clone");
        }
        // Under headers that cannot answer, the length still answers.
        let fault = TestHeaders {
            fault: true,
            ..TestHeaders::default()
        };
        assert_eq!(
            s.step(
                &params(),
                &fault,
                Input::Evidence(Evidence::Proof(p.clone()))
            ),
            Err(CheckError::AmbiguousLength),
            "{name}"
        );
        assert_eq!(s, before, "{name}");
    }
    // A proof of another transaction is still named as such, and re-asked.
    let mut s = State::measured(&bytes);
    assert_eq!(
        s.step(
            &params(),
            &h,
            Input::Evidence(Evidence::Proof(proof(&hash(9), 10, 2)))
        ),
        Err(CheckError::TxidMismatch)
    );
    assert_eq!(s.reask(), Some(&Reask::ProofFailed { height: 10 }));

    // The same through the tracker, with no index moved.
    let mut tracker = Tracker::new(params());
    tracker.track_bytes(&bytes).unwrap();
    let before = tracker.get(&txid).unwrap().clone();
    assert_eq!(
        tracker.apply(&txid, &h, Input::Evidence(Evidence::Proof(p.clone()))),
        Err(CheckError::AmbiguousLength)
    );
    assert_eq!(tracker.get(&txid), Some(&before));
    assert_eq!(before.word(), &Word::Unknown);
    assert!(tracker.clone().on_chain(&h, fork(10)).is_empty());
    assert!(tracker.clone().on_chain(&h, reorg(10)).is_empty());
    assert!(tracker.on_chain(&h, tip(11, 90)).is_empty());

    // The refusal is the length's: the same road at another length is taken.
    let other = tx_bytes(1, OTHER_LEN);
    let other_txid = txid_of(&other);
    let mut h = TestHeaders::default();
    let p = proof(&other_txid, 10, 2);
    h.insert(&p, 10);
    let mut s = State::measured(&other);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    mined(&s, 10);
    assert_eq!(s.tx_len(), Some(OTHER_LEN as u64));
}
