//! The two constructors and the doors: a transaction of exactly 64 bytes is
//! never taken as proven by a merkle path (BRC-237's condition for an anchor;
//! bsv-rs enforces it in its own verifier, GHSA-jjhc-7hvh-c3hm). The crate
//! is told the length at the constructor, from the transaction's bytes
//! (`Proof::new`) or as the host's word (`Proof::with_stated_len`), and each
//! door reads it: the state's own measure where it holds one, the proof's
//! where it does not. Each test is a table; each row a name, an input and
//! the exact answer.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod common;
use bsv_rs::transaction::MerklePath;
use bsv_tracker::doubts::{self, Bounds, Doubts};
use bsv_tracker::*;
use common::*;
use serde_json::Value;

/// BRC-237's vector `T64`, with its path and the header at its height.
const T64: &str = include_str!("vectors/brc237/t64.json");

fn unhex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// A path that fails bsv-rs's validation: no leaf at all.
fn malformed() -> MerklePath {
    MerklePath {
        block_height: 10,
        path: vec![],
    }
}

/// The proof's root carried by the headers at its height.
fn headers_for(p: &Proof) -> TestHeaders {
    let mut h = TestHeaders::default();
    h.insert(p, u64::from(p.height()));
    h
}

/// The record's proof of a state mined by `p` under `h`: it carries no
/// length of its own.
fn record_of(txid: &str, p: Proof, h: &TestHeaders) -> Proof {
    let mut s = State::new(txid);
    step(&mut s, h, Input::Evidence(Evidence::Proof(p)));
    s.evidence().unwrap().proof().clone()
}

fn offer(s: &mut State, h: &TestHeaders, p: Proof) -> Result<(), CheckError> {
    s.step(&params(), h, Input::Evidence(Evidence::Proof(p)))
}

#[test]
fn constructors_by_length() {
    for (name, len, answer) in [
        ("63 bytes", 63_usize, None),
        ("64 bytes", 64, Some(CheckError::AmbiguousLength)),
        ("65 bytes", 65, None),
    ] {
        let raw = tx_bytes(7, len);
        let path = path_of(&txid_of(&raw), 10, 2);
        let built = Proof::new(&raw, path.clone());
        let stated = Proof::with_stated_len(txid_of(&raw), len as u64, path);
        match &answer {
            None => {
                assert_eq!(
                    built.as_ref().map(Proof::txid),
                    Ok(&*txid_of(&raw)),
                    "{name}"
                );
                assert!(stated.is_ok(), "{name}: stated");
            }
            Some(fault) => {
                assert_eq!(built.as_ref().err(), Some(fault), "{name}");
                assert_eq!(stated.as_ref().err(), Some(fault), "{name}: stated");
            }
        }
        // The length is read before the path: 64 is AmbiguousLength whatever
        // the path, and any other length with a malformed path is InvalidProof.
        let bad_built = Proof::new(&raw, malformed());
        let bad_stated = Proof::with_stated_len(txid_of(&raw), len as u64, malformed());
        match &answer {
            None => {
                assert!(
                    matches!(bad_built, Err(CheckError::InvalidProof(_))),
                    "{name}"
                );
                assert!(
                    matches!(bad_stated, Err(CheckError::InvalidProof(_))),
                    "{name}"
                );
            }
            Some(fault) => {
                assert_eq!(bad_built.as_ref().err(), Some(fault), "{name}: malformed");
                assert_eq!(bad_stated.as_ref().err(), Some(fault), "{name}: malformed");
            }
        }
    }
    // A stated 64 beside a txid that is not hex is still AmbiguousLength.
    assert_eq!(
        Proof::with_stated_len("not a txid", AMBIGUOUS_TX_LEN, malformed()),
        Err(CheckError::AmbiguousLength)
    );
    // No other stated number is refused: the crate cannot check a statement.
    for len in [0, 1, 63, 65, u64::MAX] {
        assert!(Proof::with_stated_len(hash(1), len, path_of(&hash(1), 10, 2)).is_ok());
    }
}

#[test]
fn bytes_that_do_not_hash_to_the_txid() {
    let ours = tx_bytes(1, 191);
    let other = tx_bytes(2, 191);
    // Another transaction's bytes beside a path whose leaf is our txid: the
    // path does not reduce for the txid of the bytes.
    assert!(matches!(
        Proof::new(&other, path_of(&txid_of(&ours), 10, 2)),
        Err(CheckError::InvalidProof(_))
    ));
    // A proof built from another transaction's bytes, with its own path, at
    // our state's door: the txid differs.
    let p = proof_of(&other, 10, 2);
    let h = headers_for(&p);
    let mut s = State::measured(&ours);
    assert_eq!(offer(&mut s, &h, p), Err(CheckError::TxidMismatch));
    assert_eq!(s.word(), &Word::Unknown);
    assert_eq!(s.tx_len(), Some(191));
    assert_eq!(s.reask(), Some(&Reask::ProofFailed { height: 10 }));
}

/// A row of the pairings: its name, the state, the proof, the answer,
/// whether the word reads `mined` after, and `tx_len` after.
type Row = (
    &'static str,
    State,
    Proof,
    Result<(), CheckError>,
    bool,
    Option<u64>,
);

/// The pairings of the state's measure and the proof's length. The row of a
/// proof from bytes of another length than the state's is a unit test in
/// `src/state.rs`, where a proof can be built on a length by hand.
#[test]
fn the_proof_door_pairings() {
    let raw = tx_bytes(1, 191);
    let txid = txid_of(&raw);
    let built = proof_of(&raw, 10, 2);
    let h = headers_for(&built);
    let stated = |len| Proof::with_stated_len(&txid, len, path_of(&txid, 10, 2)).unwrap();
    let record = record_of(&txid, stated(191), &h);
    let measured = State::measured(&raw);
    let unmeasured = State::new(&txid);

    // (name, the state, the proof, the answer, mined after, tx_len after)
    let rows: Vec<Row> = vec![
        (
            "measured n, from bytes of n",
            measured.clone(),
            built.clone(),
            Ok(()),
            true,
            Some(191),
        ),
        (
            "measured n, stated n",
            measured.clone(),
            stated(191),
            Ok(()),
            true,
            Some(191),
        ),
        (
            "measured n, stated m",
            measured.clone(),
            stated(190),
            Err(CheckError::LengthMismatch {
                measured: 191,
                offered: 190,
            }),
            false,
            Some(191),
        ),
        (
            "measured n, a record's proof",
            measured.clone(),
            record.clone(),
            Ok(()),
            true,
            Some(191),
        ),
        (
            "unmeasured, from bytes of n",
            unmeasured.clone(),
            built.clone(),
            Ok(()),
            true,
            Some(191),
        ),
        (
            "unmeasured, stated n",
            unmeasured.clone(),
            stated(191),
            Ok(()),
            true,
            None,
        ),
        (
            "unmeasured, a record's proof",
            unmeasured.clone(),
            record.clone(),
            Err(CheckError::Unmeasured),
            false,
            None,
        ),
    ];
    for (name, mut s, p, answer, mined_after, len_after) in rows {
        let refused = answer.is_err();
        assert_eq!(offer(&mut s, &h, p), answer, "{name}");
        assert_eq!(s.tx_len(), len_after, "{name}: tx_len");
        if mined_after {
            mined(&s, 10);
            assert_eq!(s.reask(), None, "{name}");
        } else {
            assert_eq!(s.word(), &Word::Unknown, "{name}: word kept");
            assert_eq!(s.evidence(), None, "{name}: record kept");
        }
        if refused {
            assert_eq!(
                s.reask(),
                Some(&Reask::ProofFailed { height: 10 }),
                "{name}"
            );
        }
    }

    // An unmeasured state is measured by a proof from bytes whatever the
    // header check answers.
    let mut s = State::new(&txid);
    let elsewhere = TestHeaders::default();
    assert_eq!(
        offer(&mut s, &elsewhere, built.clone()),
        Err(CheckError::Unavailable(10))
    );
    assert_eq!(s.tx_len(), Some(191));
    assert_eq!(s.word(), &Word::Unknown);

    // A refusal keeps a mined word and its record: an unmeasured state mined
    // on a stated length, offered its own record again.
    let mut s = State::new(&txid);
    offer(&mut s, &h, stated(191)).unwrap();
    let saved = s.evidence().cloned();
    let again = s.evidence().unwrap().proof().clone();
    assert_eq!(offer(&mut s, &h, again), Err(CheckError::Unmeasured));
    mined(&s, 10);
    assert_eq!(s.evidence(), saved.as_ref());
    assert_eq!(s.reask(), Some(&Reask::ProofFailed { height: 10 }));

    // A state measured at 64 takes no proof, by any road: stated at 63, at
    // 65, or a record's proof. Left whole, no re-ask written.
    let raw64 = tx_bytes(1, 64);
    let txid64 = txid_of(&raw64);
    let at = |len| Proof::with_stated_len(&txid64, len, path_of(&txid64, 10, 2)).unwrap();
    let h64 = headers_for(&at(63));
    let record64 = record_of(&txid64, at(63), &h64);
    for (name, p) in [
        ("stated 63", at(63)),
        ("stated 65", at(65)),
        ("a record's proof", record64),
    ] {
        let mut s = State::measured(&raw64);
        let before = s.clone();
        assert_eq!(
            offer(&mut s, &h64, p),
            Err(CheckError::AmbiguousLength),
            "{name}"
        );
        assert_eq!(s, before, "{name}: left whole");
    }
}

#[test]
fn a_measured_state_against_a_stated_length_proof() {
    let raw = tx_bytes(3, 250);
    let txid = txid_of(&raw);
    let p = |len| Proof::with_stated_len(&txid, len, path_of(&txid, 20, 4)).unwrap();
    let h = headers_for(&p(250));
    for (name, len, answer) in [
        ("stated equal", 250, Ok(())),
        (
            "stated less",
            249,
            Err(CheckError::LengthMismatch {
                measured: 250,
                offered: 249,
            }),
        ),
        (
            "stated more",
            251,
            Err(CheckError::LengthMismatch {
                measured: 250,
                offered: 251,
            }),
        ),
    ] {
        let mut s = State::measured(&raw);
        step(&mut s, &h, Input::Host(HostAction::Build));
        assert_eq!(offer(&mut s, &h, p(len)), answer, "{name}");
        assert_eq!(s.tx_len(), Some(250), "{name}");
        if answer.is_ok() {
            mined(&s, 20);
        } else {
            assert_eq!(s.word(), &Word::Built, "{name}: word kept");
            assert_eq!(
                s.reask(),
                Some(&Reask::ProofFailed { height: 20 }),
                "{name}"
            );
        }
    }
}

#[test]
fn the_competitor_door() {
    let ours = hash(1);
    let rival = competitor(2, 10, 3);
    let rival_txid = rival.txid().to_owned();
    let h = headers_for(&rival);
    let stated = Proof::with_stated_len(&rival_txid, 191, path_of(&rival_txid, 10, 3)).unwrap();
    let record = record_of(&rival_txid, stated.clone(), &h);

    let mut s = State::new(&ours);
    let r = s.step(&params(), &h, Input::Evidence(Evidence::Competitor(rival)));
    assert_eq!(r, Ok(()), "from bytes");
    assert_eq!(
        s.word(),
        &Word::Conflicted {
            competitors: vec![rival_txid.clone()]
        }
    );

    for (name, p) in [("stated", stated), ("a record's proof", record)] {
        let mut s = State::new(&ours);
        step(&mut s, &h, Input::Host(HostAction::Build));
        let r = s.step(&params(), &h, Input::Evidence(Evidence::Competitor(p)));
        assert_eq!(r, Err(CheckError::Unmeasured), "{name}");
        assert_eq!(s.word(), &Word::Built, "{name}: word kept");
        assert_eq!(
            s.reask(),
            Some(&Reask::ProofFailed { height: 10 }),
            "{name}"
        );
    }
}

/// The fault road is for the tracked transaction's own proof; a competitor's
/// proof is judged by the competitor's own length, never by ours.
#[test]
fn the_competitor_is_judged_by_its_own_length() {
    // (name, our bytes' length, the competitor's bytes' length)
    for (name, ours, theirs) in [
        ("ours at 64, theirs at 191", 64_usize, 191_usize),
        ("ours at 191, theirs at 150", 191, 150),
        ("ours at 150, theirs at 63", 150, 63),
    ] {
        let raw = tx_bytes(1, ours);
        let rival = proof_of(&tx_bytes(2, theirs), 10, 3);
        let rival_txid = rival.txid().to_owned();
        let h = headers_for(&rival);
        let mut s = State::measured(&raw);
        let r = s.step(&params(), &h, Input::Evidence(Evidence::Competitor(rival)));
        assert_eq!(r, Ok(()), "{name}");
        assert_eq!(
            s.word(),
            &Word::Conflicted {
                competitors: vec![rival_txid]
            },
            "{name}"
        );
        assert_eq!(
            s.tx_len(),
            Some(ours as u64),
            "{name}: our measure untouched"
        );
    }

    // A conflicted word written by a competitor's proof holds no proof of
    // ours: our 64 bytes handed after it are measured with no fault, and the
    // word stands.
    let raw64 = tx_bytes(1, 64);
    let rival = competitor(2, 10, 3);
    let h = headers_for(&rival);
    let mut s = State::new(txid_of(&raw64));
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Competitor(rival.clone())),
    );
    assert_eq!(s.measure(&raw64), Ok(()));
    assert_eq!(s.tx_len(), Some(AMBIGUOUS_TX_LEN));
    assert_eq!(
        s.word(),
        &Word::Conflicted {
            competitors: vec![rival.txid().into()]
        }
    );
    assert_eq!(s.evidence(), None);
}

#[test]
fn the_doors_with_no_state() {
    let raw = tx_bytes(4, 191);
    let built = proof_of(&raw, 100, 2);
    let txid = built.txid().to_owned();
    let stated = Proof::with_stated_len(&txid, 191, path_of(&txid, 100, 2)).unwrap();
    let mut h = headers_for(&built);
    h.tip = 101;
    let record = record_of(&txid, stated.clone(), &h);
    let ds = doubts::run(&Bounds::B400, &Doubts::cold(99), &[tip_row(100)]);
    for (name, p, answer) in [
        ("from bytes", built, Ok(true)),
        ("stated", stated, Ok(true)),
        ("a record's proof", record, Err(CheckError::Unmeasured)),
    ] {
        assert_eq!(doubts::may_enter(&h, &ds, &p), answer, "may_enter: {name}");
        let checked = h.check(p.clone());
        match answer {
            Ok(_) => assert_eq!(checked.map(|c| c.proof().clone()), Ok(p), "check: {name}"),
            Err(fault) => assert_eq!(checked, Err(fault), "check: {name}"),
        }
    }
}

/// `Tracker::restore` never loses a measure the tracker holds: a restored
/// row with no length is held with the measure carried across, and a held
/// measure of 64 beside a restored proof is the fault road.
#[test]
fn restore_carries_the_held_measure() {
    let unmeasured_row = |txid: &str, mine: bool| {
        let mut s = State::new(txid);
        step(
            &mut s,
            &TestHeaders::default(),
            Input::Host(HostAction::Build),
        );
        if mine {
            let p = proof(txid, 10, 2);
            step(
                &mut s,
                &headers_for(&p),
                Input::Evidence(Evidence::Proof(p)),
            );
        }
        State::from_compact(&s.to_compact()).unwrap()
    };

    // Held at 191, a row with no length and a proof: carried, the word stands.
    let raw = tx_bytes(1, 191);
    let txid = txid_of(&raw);
    let mut t = Tracker::new(params());
    t.track_bytes(&raw).unwrap();
    let held = t.get(&txid).unwrap().clone();
    assert_eq!(t.restore(unmeasured_row(&txid, true)), Ok(Some(held)));
    let s = t.get(&txid).unwrap();
    assert_eq!(s.tx_len(), Some(191));
    mined(s, 10);
    assert!(s.suspect());

    // Held at 64, a row with no length and no proof: carried, nothing else moves.
    let raw64 = tx_bytes(1, 64);
    let txid64 = txid_of(&raw64);
    let mut t = Tracker::new(params());
    t.track_bytes(&raw64).unwrap();
    assert!(t.restore(unmeasured_row(&txid64, false)).unwrap().is_some());
    let s = t.get(&txid64).unwrap();
    assert_eq!(s.tx_len(), Some(AMBIGUOUS_TX_LEN));
    assert_eq!(s.word(), &Word::Built);

    // Held at 64, a row with no length and a proof: the fault road. The fault
    // is returned AND the row is held unsaid: no record, the word `Unknown`,
    // no mark, no re-ask, out of the suspect set.
    let mut t = Tracker::new(params());
    t.track_bytes(&raw64).unwrap();
    assert_eq!(
        t.restore(unmeasured_row(&txid64, true)),
        Err(CheckError::AmbiguousLength)
    );
    let s = t.get(&txid64).unwrap();
    assert_eq!(s.tx_len(), Some(AMBIGUOUS_TX_LEN));
    assert_eq!(s.word(), &Word::Unknown);
    assert_eq!(s.evidence(), None);
    assert!(!s.suspect());
    assert_eq!(s.reask(), None);
    assert!(t.on_chain(&TestHeaders::default(), tip(11, 90)).is_empty());

    // Held at 191, a row that carries another measure: refused, nothing moved.
    let mut bytes = State::new(&txid).to_compact();
    assert_eq!(bytes.pop(), Some(0));
    bytes.extend([1, 0xbe, 0x01]);
    let other = State::from_compact(&bytes).unwrap();
    assert_eq!(other.tx_len(), Some(190));
    let mut t = Tracker::new(params());
    t.track_bytes(&raw).unwrap();
    let before = t.get(&txid).unwrap().clone();
    assert_eq!(
        t.restore(other),
        Err(CheckError::LengthMismatch {
            measured: 191,
            offered: 190
        })
    );
    assert_eq!(t.get(&txid), Some(&before));

    // Held unmeasured, a measured row: the row's measure stands.
    let mut t = Tracker::new(params());
    t.track(&txid);
    assert!(t.restore(State::measured(&raw)).unwrap().is_some());
    assert_eq!(t.get(&txid).unwrap().tx_len(), Some(191));

    // Not tracked: held as it is.
    let mut t = Tracker::new(params());
    assert_eq!(t.restore(unmeasured_row(&txid, true)), Ok(None));
    assert_eq!(t.get(&txid).unwrap().tx_len(), None);
}

/// The witness: BRC-237's own test transaction of 64 bytes, `T64`, with its
/// BRC-74 path at height 950008, whose root is the merkle root of the
/// vector's header at that height. Never taken as proven, by either road.
#[test]
fn brc237_t64_is_never_proven() {
    let v: Value = serde_json::from_str(T64).unwrap();
    let raw = unhex(v["raw_tx"].as_str().unwrap());
    let txid = v["txid"].as_str().unwrap();
    let height = u32::try_from(v["height"].as_u64().unwrap()).unwrap();
    let root = v["merkle_root"].as_str().unwrap();
    let header = unhex(v["header"].as_str().unwrap());
    let path = MerklePath::from_hex(v["path"].as_str().unwrap()).unwrap();

    // The vector holds together: 64 bytes, whose hash is the txid, whose path
    // folds to the root in bytes 36 to 67 of the header at its height.
    assert_eq!(raw.len() as u64, v["tx_len"].as_u64().unwrap());
    assert_eq!(raw.len() as u64, AMBIGUOUS_TX_LEN);
    assert_eq!(txid_of(&raw), txid);
    assert_eq!(path.block_height, height);
    assert_eq!(path.compute_root(Some(txid)).unwrap(), root);
    let mut header_root = header[36..68].to_vec();
    header_root.reverse();
    assert_eq!(unhex(root), header_root);

    let mut h = TestHeaders {
        tip: height,
        ..TestHeaders::default()
    };
    h.headers.insert(
        height,
        Header {
            hash: v["block_hash"].as_str().unwrap().into(),
            merkle_root: root.into(),
        },
    );

    // By either constructor: no proof.
    assert_eq!(
        Proof::new(&raw, path.clone()),
        Err(CheckError::AmbiguousLength)
    );
    assert_eq!(
        Proof::with_stated_len(txid, AMBIGUOUS_TX_LEN, path.clone()),
        Err(CheckError::AmbiguousLength)
    );
    // A state measured from the bytes refuses a proof stated at another
    // length, under headers that carry its root, and is left whole.
    let mut s = State::measured(&raw);
    let before = s.clone();
    let stated = Proof::with_stated_len(txid, 63, path.clone()).unwrap();
    assert_eq!(offer(&mut s, &h, stated), Err(CheckError::AmbiguousLength));
    assert_eq!(s, before);
    assert_eq!(s.word(), &Word::Unknown);
    // Through a tracker, the same; no index moves.
    let mut t = Tracker::new(params());
    t.track_bytes(&raw).unwrap();
    let stated = Proof::with_stated_len(txid, 65, path).unwrap();
    assert_eq!(
        t.apply(txid, &h, Input::Evidence(Evidence::Proof(stated))),
        Err(CheckError::AmbiguousLength)
    );
    assert_eq!(t.get(txid).unwrap().word(), &Word::Unknown);
    assert!(t.on_chain(&h, reorg(height)).is_empty());
}
