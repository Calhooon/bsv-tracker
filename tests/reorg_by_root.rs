//! The reorg rule by the root: the ruling (5) of 2026-10-20 19:4xZ
//! (bsv-stack-lean docs/RULINGS-2026-10-12.md). A `mined` word is `stale`
//! when, and only when, its stored proof's root is not the active header's
//! root at its height in the host's headers. A reorg or an invalidation names
//! the words to recheck; the recheck is a header read, never a fetch.
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

/// One mined word per height, each checked against `h`.
fn tracker_at(h: &mut TestHeaders, heights: &[u32]) -> Tracker {
    let mut tracker = Tracker::new(params());
    for &height in heights {
        let txid = hash(u64::from(height));
        let p = proof(&txid, height, 7);
        h.insert(&p, u64::from(height));
        tracker
            .apply(&txid, h, Input::Evidence(Evidence::Proof(p)))
            .unwrap();
    }
    tracker
}

#[test]
fn a_reorg_whose_new_branch_moves_the_root_makes_the_word_stale() {
    let mut h = TestHeaders::default();
    let mut tracker = tracker_at(&mut h, &[100]);
    let saved = tracker.get(&hash(100)).unwrap().evidence().cloned();
    h.move_root(100, 300);
    let updates = tracker.on_chain(&h, reorg(99));
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].result, Ok(()));
    assert_eq!(updates[0].reask, Some(Reask::Reorg { fork_height: 99 }));
    let s = tracker.get(&hash(100)).unwrap();
    assert_eq!(s.word(), &Word::Stale);
    assert!(!s.suspect());
    assert_eq!(s.evidence(), saved.as_ref());
}

#[test]
fn a_reorg_whose_active_header_still_carries_the_root_moves_nothing() {
    // The repair row's shape (bsv-stack-lean DECISIONS-2026-10-19 item 5): a
    // reorg at or below the words whose headers still carry their roots.
    for fork_height in [90, 100] {
        let mut h = TestHeaders::default();
        let mut tracker = tracker_at(&mut h, &[100, 101, 105]);
        let before: Vec<State> = [100, 101, 105]
            .iter()
            .map(|&n| tracker.get(&hash(n)).unwrap().clone())
            .collect();
        let updates = tracker.on_chain(&h, reorg(fork_height));
        assert_eq!(updates.len(), 3);
        for (update, old) in updates.iter().zip(&before) {
            assert_eq!(update.result, Ok(()));
            assert_eq!(update.reask, None);
            let s = tracker.get(&update.txid).unwrap();
            assert_eq!(s, old);
            assert!(!s.suspect());
            mined(s, old.word().height().unwrap());
        }
        // Nothing went stale, so the same event selects the same words again.
        assert_eq!(tracker.on_chain(&h, reorg(fork_height)).len(), 3);
    }
}

#[test]
fn a_reorg_clears_a_standing_fork_mark_when_the_root_holds() {
    let mut h = TestHeaders::default();
    let mut tracker = tracker_at(&mut h, &[100]);
    tracker.on_chain(&h, fork(100));
    assert!(tracker.get(&hash(100)).unwrap().suspect());
    let updates = tracker.on_chain(&h, reorg(100));
    assert_eq!(updates[0].reask, None);
    let s = tracker.get(&hash(100)).unwrap();
    assert!(!s.suspect());
    mined(s, 100);
}

#[test]
fn a_reorg_with_no_header_at_the_height_keeps_the_word_and_fails_closed() {
    let mut h = TestHeaders::default();
    let mut tracker = tracker_at(&mut h, &[100]);
    let kept = h.headers.remove(&100).unwrap();
    let updates = tracker.on_chain(&h, reorg(100));
    assert_eq!(updates[0].result, Err(CheckError::Unavailable(100)));
    assert_eq!(updates[0].reask, Some(Reask::Recheck));
    let s = tracker.get(&hash(100)).unwrap();
    mined(s, 100);
    // Suspect until the headers answer: not served, and the next tip rechecks it.
    assert!(s.suspect());
    h.headers.insert(100, kept);
    let updates = tracker.on_chain(&h, tip(101, 101));
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].reask, None);
    let s = tracker.get(&hash(100)).unwrap();
    assert!(!s.suspect());
    mined(s, 100);
}

#[test]
fn a_word_below_the_fork_is_untouched() {
    let mut h = TestHeaders::default();
    let mut tracker = tracker_at(&mut h, &[98, 100]);
    let below = tracker.get(&hash(98)).unwrap().clone();
    // Even a moved root below the fork is not the reorg's to recheck.
    h.move_root(98, 298);
    h.move_root(100, 300);
    let updates = tracker.on_chain(&h, reorg(99));
    assert_eq!(
        updates.iter().map(|u| u.txid.clone()).collect::<Vec<_>>(),
        [hash(100)]
    );
    assert_eq!(tracker.get(&hash(98)).unwrap(), &below);

    // One state alone: the arm's own bound.
    let mut s = below.clone();
    step(&mut s, &h, Input::Evidence(Evidence::Chain(reorg(99))));
    assert_eq!(s, below);
}

#[test]
fn an_invalidation_of_the_checked_header_rechecks_by_the_root() {
    // The root moved: stale, re-asked at the word's height.
    let mut h = TestHeaders::default();
    let mut tracker = tracker_at(&mut h, &[100]);
    h.move_root(100, 300);
    let updates = tracker.on_chain(
        &h,
        ChainEvent::Invalidated {
            block_hash: hash(100),
        },
    );
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].reask, Some(Reask::Reorg { fork_height: 100 }));
    assert_eq!(tracker.get(&hash(100)).unwrap().word(), &Word::Stale);

    // The same root still active at the height: the word stands.
    let mut h = TestHeaders::default();
    let mut tracker = tracker_at(&mut h, &[100]);
    let before = tracker.get(&hash(100)).unwrap().clone();
    let updates = tracker.on_chain(
        &h,
        ChainEvent::Invalidated {
            block_hash: hash(100),
        },
    );
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].result, Ok(()));
    assert_eq!(updates[0].reask, None);
    assert_eq!(tracker.get(&hash(100)).unwrap(), &before);

    // Another block's invalidation does not select the word.
    let mut s = before.clone();
    h.move_root(100, 300);
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Chain(ChainEvent::Invalidated {
            block_hash: hash(1),
        })),
    );
    assert_eq!(s, before);
}

#[test]
fn a_mempool_presence_over_built_and_over_announced_reads_announced() {
    // The ruling (2) of 2026-10-20 19:4xZ (bsv-stack-lean docs/RULINGS-2026-10-12.md).
    let h = TestHeaders::default();
    let present = hint(
        HintStatus::MempoolPresent {
            source: "courier".into(),
        },
        5,
    );
    let mut built = State::new(hash(1));
    step(&mut built, &h, Input::Host(HostAction::Build));
    step(&mut built, &h, present.clone());
    assert_eq!(built.word(), &Word::Announced);
    assert_eq!(built.reask(), None);

    let mut announced = State::new(hash(1));
    step(&mut announced, &h, Input::Host(HostAction::Build));
    step(&mut announced, &h, hint(HintStatus::Accepted, 0));
    step(&mut announced, &h, present);
    assert_eq!(announced.word(), &Word::Announced);
    assert_eq!(announced.reask(), None);
    // The same word either way.
    assert_eq!(built.word(), announced.word());
}
