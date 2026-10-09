//! Local tracker projections of the seven corpus witnesses at stack pin 24f22f8.
//! Actual wallet calls, caches and feed delivery are integration lane work.
mod common;
use bsv_tracker::*;
use common::*;

#[test]
fn mined_orphaned_remined_broadcaster_latch() {
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    announce(&mut s, &h);
    let old = proof(&txid, 100, 2);
    h.insert(&old, 100);
    step(&mut s, &h, hint(HintStatus::Mined { height: 100 }, 2));
    assert_eq!(s.word(), &Word::Seen);
    assert!(matches!(s.reask(), Some(Reask::Hint(_))));
    step(&mut s, &h, Input::Evidence(Evidence::Proof(old.clone())));
    mined(&s, 100);
    assert_eq!(s.reask(), None);
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Chain(ChainEvent::Fork {
            height: 100,
            competing_tips: vec![hash(100), hash(101)],
            depth: 1,
        })),
    );
    mined(&s, 100);
    assert!(s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Fork { height: 100 }));
    h.insert(&proof(&hash(3), 100, 4), 101);
    step(&mut s, &h, Input::Evidence(Evidence::Chain(reorg(100))));
    assert_eq!(s.word(), &Word::Stale);
    assert!(!s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Reorg { fork_height: 100 }));
    let retained = s.evidence().cloned();
    step(&mut s, &h, hint(HintStatus::StaleBlock, 3));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.evidence(), retained.as_ref());
    step(&mut s, &h, hint(HintStatus::Mined { height: 100 }, 4));
    assert_eq!(s.word(), &Word::Stale);
    assert!(matches!(
        s.step(&params(), &h, Input::Evidence(Evidence::Proof(old))),
        Err(CheckError::RootMismatch(100))
    ));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.reask(), Some(&Reask::ProofFailed { height: 100 }));
    step(&mut s, &h, hint(HintStatus::Seen, 5));
    assert_eq!(s.word(), &Word::Stale);
    step(&mut s, &h, Input::Host(HostAction::Tick(1_000)));
    assert_eq!(s.word(), &Word::Stale);
    assert!(matches!(s.reask(), Some(Reask::Hint(_))));
    let new = proof(&txid, 102, 5);
    h.insert(&new, 102);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(new)));
    mined(&s, 102);
    assert_eq!(s.reask(), None);
    step(&mut s, &h, hint(HintStatus::StaleBlock, 1_001));
    mined(&s, 102);
    step(&mut s, &h, Input::Host(HostAction::SpendAttempt));
    mined(&s, 102);
    assert_eq!(s.reask(), Some(&Reask::Spend));
    step(&mut s, &h, Input::Evidence(Evidence::Recheck));
    mined(&s, 102);
    // Tracker.recheck preserves the existing reask on success (Lean line 324).
    assert_eq!(s.reask(), Some(&Reask::Spend));
}

#[test]
fn reorg_announced_nobody_hears_go_server_ts_client_shape() {
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    announce(&mut s, &h);
    let old = proof(&txid, 100, 2);
    h.insert(&old, 100);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(old)));
    mined(&s, 100);
    h.insert(&proof(&hash(3), 100, 4), 101);
    mined(&s, 100);
    let mut no_feed = s.clone();
    let fault = decode_envelope(br#"{"v":1,"kind":"reorg","orphanedHashes":[],"commonAncestor":"x","newTip":"y","depth":1}"#);
    assert!(matches!(
        fault,
        Err(DecodeError::UnknownShape {
            version: Some(1),
            ..
        })
    ));
    mined(&s, 100);
    let event = decode_envelope(
        br#"{"v":1,"kind":"reorg","forkHeight":100,"depth":1,"deactivated":[],"newTip":"new"}"#,
    )
    .unwrap();
    step(&mut s, &h, Input::Evidence(Evidence::Chain(event)));
    assert_eq!(s.word(), &Word::Stale);
    step(&mut no_feed, &h, Input::Evidence(Evidence::Recheck));
    assert_eq!(no_feed.word(), &Word::Stale);
    step(&mut s, &h, Input::Host(HostAction::Tick(1_000)));
    assert_eq!(s.word(), &Word::Stale);
    step(&mut s, &h, hint(HintStatus::Seen, 1_001));
    assert_eq!(s.word(), &Word::Stale);
    step(&mut s, &h, Input::Host(HostAction::SpendAttempt));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.reask(), Some(&Reask::Spend));
    step(&mut s, &h, Input::Evidence(Evidence::Recheck));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.reask(), Some(&Reask::Spend));
}

#[test]
fn production_proofs_orphaned_healed_without_a_hand() {
    let mut h = TestHeaders::default();
    for n in [1, 2] {
        let txid = hash(n);
        let mut s = State::new(&txid);
        let old = proof(&txid, 100, n + 10);
        h.insert(&old, 100);
        step(&mut s, &h, Input::Evidence(Evidence::Proof(old.clone())));
        mined(&s, 100);
        h.insert(&proof(&hash(3), 100, 20), 101);
        step(&mut s, &h, Input::Evidence(Evidence::Chain(reorg(100))));
        assert_eq!(s.word(), &Word::Stale);
        assert!(s
            .step(&params(), &h, Input::Evidence(Evidence::Proof(old)))
            .is_err());
        assert_eq!(s.word(), &Word::Stale);
        let new = proof(&txid, 102, n + 30);
        h.insert(&new, 102);
        step(&mut s, &h, Input::Evidence(Evidence::Proof(new)));
        mined(&s, 102);
        step(&mut s, &h, Input::Evidence(Evidence::Recheck));
        mined(&s, 102);
        assert_eq!(
            s.evidence().unwrap().header().merkle_root,
            h.headers[&102].merkle_root
        );
    }
}

#[test]
fn contract_1_versioned_envelope_fails_loud() {
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    let p = proof(&txid, 100, 2);
    h.insert(&p, 100);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    mined(&s, 100);
    let tip = decode_envelope(br#"{"v":1,"kind":"tip","height":101,"hash":"tip"}"#).unwrap();
    step(&mut s, &h, Input::Evidence(Evidence::Chain(tip)));
    mined(&s, 100);
    let mut faults = vec![];
    for body in [
        br#"{"kind":"future","v":2}"#.as_slice(),
        br#"{"v":1,"kind":"reorg","forkHeight":100,"depth":1,"orphanedHashes":[],"newTip":"b"}"#,
    ] {
        faults.push(decode_envelope(body).unwrap_err());
        mined(&s, 100);
    }
    assert_eq!(faults[0], DecodeError::UnknownVersion(2));
    assert!(matches!(
        faults[1],
        DecodeError::UnknownShape {
            version: Some(1),
            ..
        }
    ));
    step(&mut s, &h, Input::Evidence(Evidence::Chain(reorg(100))));
    assert_eq!(s.word(), &Word::Stale);
}

#[test]
fn contract_2_pending_wallet_call_tracker_projection() {
    // The approval machine owns deadline enforcement. Only its successful
    // T1 enters the tracker; the other calls produce no tracker input.
    let h = TestHeaders::default();
    let mut t1 = State::new(hash(1));
    step(&mut t1, &h, Input::Host(HostAction::Build));
    assert_eq!(t1.word(), &Word::Built);
    step(&mut t1, &h, hint(HintStatus::Accepted, 5));
    assert_eq!(t1.word(), &Word::Announced);
    let mut tracked = vec![t1];
    for _deadline_outcome in ["expired", "cancelled", "expired"] {
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].word(), &Word::Announced);
    }
    // If a wallet broadcasts anyway, the lost call cannot hide its hints.
    let mut gap = State::new(hash(3));
    step(&mut gap, &h, Input::Host(HostAction::Build));
    assert_eq!(gap.word(), &Word::Built);
    step(&mut gap, &h, hint(HintStatus::Accepted, 90));
    assert_eq!(gap.word(), &Word::Announced);
    tracked.push(gap);
    assert_eq!(tracked.len(), 2);
}

#[test]
fn contract_3_tracker_word_only_chain_word() {
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    step(&mut s, &h, Input::Host(HostAction::Build));
    assert_eq!(s.word(), &Word::Built);
    step(&mut s, &h, hint(HintStatus::Accepted, 1));
    assert_eq!(s.word(), &Word::Announced);
    step(
        &mut s,
        &h,
        hint(
            HintStatus::Rejected {
                reason: "fee".into(),
            },
            2,
        ),
    );
    assert_eq!(s.word(), &Word::Announced);
    step(&mut s, &h, hint(HintStatus::Seen, 3));
    assert_eq!(s.word(), &Word::Seen);
    let p = proof(&txid, 100, 2);
    h.insert(&p, 100);
    step(&mut s, &h, hint(HintStatus::Mined { height: 100 }, 4));
    assert_eq!(s.word(), &Word::Seen);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    mined(&s, 100);
    step(&mut s, &h, Input::Evidence(Evidence::Chain(reorg(100))));
    assert_eq!(s.word(), &Word::Stale);
    step(&mut s, &h, hint(HintStatus::Mined { height: 100 }, 5));
    assert_eq!(s.word(), &Word::Stale);
    step(&mut s, &h, hint(HintStatus::OrphanMempool, 6));
    assert_eq!(s.word(), &Word::Stale);
    let new = proof(&txid, 102, 5);
    h.insert(&new, 102);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(new)));
    mined(&s, 102);
    assert_eq!(s.hints().len(), 6);
}

#[test]
fn contract_4_served_proof_tracker_projection() {
    // Only the tracker's evidence and word are replayed. The app's cache,
    // count attestation, TTL and ETag remain integration obligations.
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    let old = proof(&txid, 100, 2);
    h.insert(&old, 100);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(old)));
    mined(&s, 100);
    let old_header = s.evidence().unwrap().header().clone();
    step(&mut s, &h, Input::Evidence(Evidence::Chain(reorg(100))));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.evidence().unwrap().header(), &old_header);
    assert_eq!(s.word().height(), None);
    let new = proof(&txid, 102, 5);
    h.insert(&new, 102);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(new)));
    mined(&s, 102);
    assert_ne!(s.evidence().unwrap().header(), &old_header);
    step(&mut s, &h, Input::Host(HostAction::Tick(1_000)));
    mined(&s, 102);
}
