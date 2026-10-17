//! The version-2 state rule: a split never writes a word; a resolved split
//! moves a word only through a checked proof. A split is an observation, not
//! a reorg (bsv-stack-lean docs/RULINGS-2026-10-12.md:323-329, the ruling of
//! 2026-10-16 19:3xZ): the service answers its most-work chain, depth is the
//! host's. 256 reproducible traces, with an explicit ChaCha seed.
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
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};

fn mined_at(height: u32) -> (State, TestHeaders) {
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    let p = proof(&txid, height, 2);
    h.insert(&p, u64::from(height));
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    mined(&s, height);
    (s, h)
}

fn chain(event: ChainEvent) -> Input {
    Input::Evidence(Evidence::Chain(event))
}

#[test]
fn split_marks_a_mined_word_suspect_and_never_stale() {
    for fork_height in [90, 100] {
        let (mut s, h) = mined_at(100);
        let saved = s.evidence().cloned();
        step(&mut s, &h, chain(split(fork_height)));
        mined(&s, 100);
        assert!(s.suspect());
        assert_eq!(
            s.reask(),
            Some(&Reask::Fork {
                height: fork_height
            })
        );
        assert_eq!(s.evidence().cloned(), saved);
        // A repeated split, even over an emptied snapshot, still writes nothing.
        let mut empty = h.clone();
        empty.headers.clear();
        step(&mut s, &empty, chain(split(fork_height)));
        mined(&s, 100);
        assert_ne!(s.word(), &Word::Stale);
    }
    let (mut s, h) = mined_at(100);
    let before = s.clone();
    step(&mut s, &h, chain(split(101)));
    assert_eq!(s, before, "a split above the word does not touch it");
}

#[test]
fn split_resolved_rechecks_through_the_snapshot() {
    // The active header still commits to the proof: the mark clears.
    let (mut s, h) = mined_at(100);
    step(&mut s, &h, chain(split(100)));
    step(&mut s, &h, chain(split_resolved(100)));
    mined(&s, 100);
    assert!(!s.suspect());
    assert_eq!(s.reask(), None);

    // The resolution left the word's block off the active chain: only the
    // checked snapshot says so, and the word goes stale through it.
    let (mut s, mut h) = mined_at(100);
    step(&mut s, &h, chain(split(99)));
    let other = proof(&hash(2), 100, 3);
    h.insert(&other, 500);
    step(&mut s, &h, chain(split_resolved(99)));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.reask(), Some(&Reask::Reorg { fork_height: 100 }));

    // An unavailable snapshot fails closed: the word and the mark stay.
    let (mut s, mut h) = mined_at(100);
    step(&mut s, &h, chain(split(100)));
    h.fault = true;
    let result = s.step(&params(), &h, chain(split_resolved(100)));
    assert!(matches!(result, Err(CheckError::Headers(_))));
    mined(&s, 100);
    assert!(s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Recheck));

    // A resolution touches no word it did not mark, nor one below it.
    let (mut s, h) = mined_at(100);
    let before = s.clone();
    step(&mut s, &h, chain(split_resolved(100)));
    assert_eq!(s, before);
    let (mut s, h) = mined_at(100);
    step(&mut s, &h, chain(split(100)));
    let before = s.clone();
    step(&mut s, &h, chain(split_resolved(101)));
    assert_eq!(s, before);
}

#[test]
fn tracker_routes_splits_to_the_mined_range_at_or_above() {
    let mut h = TestHeaders::default();
    let mut tracker = Tracker::new(params());
    for height in 100..=102 {
        let txid = hash(u64::from(height));
        let p = proof(&txid, height, 1);
        h.insert(&p, u64::from(height));
        tracker
            .apply(&txid, &h, Input::Evidence(Evidence::Proof(p)))
            .unwrap();
    }
    tracker.track(hash(7));
    let updates = tracker.on_chain(&h, split(101));
    assert_eq!(
        updates.iter().map(|u| u.txid.clone()).collect::<Vec<_>>(),
        [hash(101), hash(102)]
    );
    for update in &updates {
        assert_eq!(update.result, Ok(()));
        assert_eq!(update.reask, Some(Reask::Fork { height: 101 }));
    }
    assert!(!tracker.get(&hash(100)).unwrap().suspect());
    for height in 100..=102 {
        mined(tracker.get(&hash(u64::from(height))).unwrap(), height);
    }
    let updates = tracker.on_chain(&h, split_resolved(101));
    assert_eq!(updates.len(), 2);
    for height in 100..=102 {
        let state = tracker.get(&hash(u64::from(height))).unwrap();
        mined(state, height);
        assert!(!state.suspect());
    }
    assert_eq!(tracker.get(&hash(7)).unwrap().word(), &Word::Unknown);
}

#[test]
fn split_writes_no_word() {
    let runner = || {
        TestRunner::new_with_rng(
            Config {
                cases: 256,
                failure_persistence: None,
                ..Config::default()
            },
            TestRng::deterministic_rng(RngAlgorithm::ChaCha),
        )
    };
    let states = prop::collection::vec((1u32..20, any::<bool>()), 1..40);
    runner()
        .run(&(states, 1u32..22), |(states, fork_height)| {
            let mut tracker = Tracker::new(params());
            let mut h = TestHeaders::default();
            let mut before = vec![];
            for (n, (height, prove)) in states.into_iter().enumerate() {
                let txid = hash(n as u64 + 100);
                tracker.track(&txid);
                if prove {
                    let p = proof(&txid, height, 1_000);
                    h.insert(&p, u64::from(height));
                    let _ = tracker.apply(&txid, &h, Input::Evidence(Evidence::Proof(p)));
                }
                before.push(tracker.get(&txid).unwrap().clone());
            }
            // An empty snapshot: nothing a split does may depend on headers.
            h.headers.clear();
            let affected = tracker.on_chain(&h, split(fork_height));
            for old in before {
                let after = tracker.get(old.txid()).unwrap();
                prop_assert_eq!(after.word(), old.word());
                prop_assert_eq!(after.evidence(), old.evidence());
                let marked = old.word().height().is_some_and(|h| h >= fork_height);
                prop_assert_eq!(after.suspect(), old.suspect() || marked);
                prop_assert_eq!(
                    affected
                        .iter()
                        .any(|u| u.txid == old.txid() && u.result.is_ok()),
                    marked
                );
            }
            Ok(())
        })
        .unwrap();
}
