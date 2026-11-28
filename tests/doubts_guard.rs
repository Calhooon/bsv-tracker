//! The doubts' reads that need a real root: the builder's test, the spend
//! guard, a headers fault, the snapshot of `lean/Headers/Host.lean` section 6,
//! and the out host and the served snapshot of section 12 (cut 1g), each test
//! named for its row of record in bsv-stack-lean at `7e616ab3`
//! (`lean/Headers/Doubts.lean`, `lean/Headers/Host.lean`,
//! `lean/Headers/Builder.lean`; the table of
//! `docs/p0/reorg-proof-cut2-2026-11-18.md`, "Cut 1g"). No case is a journal
//! row.
//!
//! The Lean's `hdAt t` is `TestHeaders` at tip `t` carrying no header; its
//! `hdRoots t` is [`hd_roots`]. The page loop of the checked host (the Lean's
//! `readC` and `readsC`) is the host's and is mirrored here, over the crate's
//! `run_checked`, `start_from` and `Snapshot`.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod common;
use bsv_tracker::doubts::{
    self, AppRule, Bounds, Checked, Doubts, Refused, Rival, Served, Snapshot,
};
use bsv_tracker::*;
use common::*;

const B400: Bounds = Bounds::B400;

/// The Lean's `hdAt t`: a tip and no header.
fn hd_at(tip: u32) -> TestHeaders {
    TestHeaders {
        tip,
        ..TestHeaders::default()
    }
}

/// The Lean's `hdRoots t`: at every height up to `t` the header `h<height>`
/// with the root `r<height>`.
fn hd_roots(tip: u32) -> TestHeaders {
    let mut headers = hd_at(tip);
    for h in 0..=tip {
        headers.headers.insert(
            h,
            Header {
                hash: format!("h{h}"),
                merkle_root: format!("r{h}"),
            },
        );
    }
    headers
}

/// The Lean's `runUnpruned`: `raw` folded, no prune.
fn unpruned(from: &Doubts, rows: &[ChainEvent]) -> Doubts {
    rows.iter().fold(from.clone(), |ds, e| doubts::raw(e, &ds))
}

/// A served snapshot with one part changed (`Served` is `#[non_exhaustive]`:
/// a host outside the crate assigns a field, it never writes a literal).
fn with(served: &Served, change: impl FnOnce(&mut Served)) -> Served {
    let mut served = served.clone();
    change(&mut served);
    served
}

fn eff(headers: &TestHeaders, ds: &Doubts, x: u32) -> u64 {
    doubts::eff_depth(headers, ds, x).unwrap()
}

/// A word mined at `height`, its root carried by `headers` (block `block`).
fn mined_at(headers: &mut TestHeaders, txid: u64, height: u32, block: u64) -> (State, Proof) {
    let p = proof(&hash(txid), height, txid + 7);
    headers.insert(&p, block);
    let mut s = State::new(hash(txid));
    step(&mut s, headers, Input::Evidence(Evidence::Proof(p.clone())));
    mined(&s, height);
    (s, p)
}

fn depth(d: u64) -> Params {
    Params::new(60, d)
}

// ---------------------------------------------------------------------------
// Appendix B: the builder's test, the guard, a fault, the snapshot (section 6)
// ---------------------------------------------------------------------------

/// `oneBehind`: the feed has read `tip 100`; the headers stand at 101. The
/// path at 100 is two deep under the headers and is admitted.
#[test]
fn may_enter_one_behind() {
    let ds = doubts::run(&B400, &Doubts::cold(99), &[tip_row(100)]);
    let mut headers = hd_at(101);
    let p = proof(&hash(1), 100, 2);
    headers.insert(&p, 100);
    assert_eq!(headers.tip, 101);
    assert_eq!(doubts::eff_depth(&headers, &ds, 101), Ok(1));
    assert_eq!(doubts::may_enter(&headers, &ds, &p), Ok(true));
}

fn feed_ahead() -> Doubts {
    let mut rows: Vec<ChainEvent> = (101..=105).map(tip_row).collect();
    rows.push(reorg_row(101, 105, 112));
    doubts::run(&B400, &Doubts::cold(100), &rows)
}

/// Cut 1e's row after `feedAhead`: the headers still hold A to 105, the
/// feed has removed 101 to 105; A's path at 103 is refused.
#[test]
fn may_enter_feed_ahead_refuses_the_removed_branch() {
    let fa = feed_ahead();
    assert_eq!(
        fa.to_string(),
        "tip=112 splits=[] rivals=[101:105] summary=-"
    );
    let mut headers = hd_at(105);
    let a = proof(&hash(1), 103, 2);
    headers.insert(&a, 103);
    assert_eq!(doubts::may_enter(&headers, &fa, &a), Ok(false));
}

/// `feedAhead` at the true tip: the headers at 112 hold B at 103. A's path
/// is refused by its root, B's is admitted.
#[test]
fn may_enter_feed_ahead_at_the_true_tip() {
    let fa = feed_ahead();
    let mut headers = hd_at(112);
    headers.move_root(103, 103);
    let a = proof(&hash(1), 103, 2);
    assert_eq!(doubts::eff_depth(&headers, &fa, 103), Ok(7));
    assert_eq!(doubts::may_enter(&headers, &fa, &a), Ok(false));
    let b = proof(&hash(3), 103, 4);
    headers.insert(&b, 203);
    assert_eq!(doubts::may_enter(&headers, &fa, &b), Ok(true));
}

/// `feedAhead` with the feed's tip held back to 105: the removed branch
/// still binds, and A's path is refused though its root is carried.
#[test]
fn may_enter_feed_ahead_with_the_tip_held_back() {
    let fa = feed_ahead();
    let held = Doubts::new(
        fa.rivals().to_vec(),
        fa.splits().to_vec(),
        105,
        fa.gone().cloned(),
    );
    let mut headers = hd_at(105);
    let a = proof(&hash(1), 103, 2);
    headers.insert(&a, 103);
    assert_eq!(doubts::may_enter(&headers, &held, &a), Ok(false));
}

/// `Builder.lean`'s `tieDs`: a one-block tie at the tip. The path into the
/// tip block is left out; the path below it enters.
#[test]
fn may_enter_tie_at_the_tip() {
    let ds = doubts::run(
        &B400,
        &Doubts::cold(800_001),
        &[fork_row(800_001, 800_001, 800_001)],
    );
    let mut headers = hd_at(800_001);
    let at_tip = proof(&hash(1), 800_001, 2);
    let below = proof(&hash(3), 800_000, 4);
    headers.insert(&at_tip, 1);
    headers.insert(&below, 0);
    assert_eq!(doubts::may_enter(&headers, &ds, &at_tip), Ok(false));
    assert_eq!(doubts::may_enter(&headers, &ds, &below), Ok(true));
}

/// `Builder.lean`'s `rivalTwo`: a rival two behind admits the path at its
/// fork height.
#[test]
fn may_enter_a_rival_two_behind() {
    let ds = doubts::run(
        &B400,
        &Doubts::cold(800_000),
        &[fork_row(800_000, 800_000, 800_000), tip_row(800_002)],
    );
    let mut headers = hd_at(800_002);
    let p = proof(&hash(1), 800_000, 2);
    headers.insert(&p, 0);
    assert_eq!(headers.tip, 800_002);
    assert_eq!(doubts::eff_depth(&headers, &ds, 800_000), Ok(2));
    assert_eq!(doubts::may_enter(&headers, &ds, &p), Ok(true));
}

/// `Host.lean`'s `gapNoHeader`: headers at 10 that carry nothing. The word
/// is six deep, and the guard spends nothing on it: no header is `false`,
/// never a pass and never a fault.
#[test]
fn spendable_eff_no_header() {
    let mut carried = TestHeaders::default();
    let (s, _) = mined_at(&mut carried, 1, 5, 5);
    let empty = hd_at(10);
    let ds = Doubts::cold(10);
    assert_eq!(doubts::eff_depth(&empty, &ds, 5), Ok(6));
    assert_eq!(
        doubts::spendable_eff(&depth(6), &empty, &ds, None, &s),
        Ok(false)
    );
    // Under the headers that carry its root it is spent at 6.
    let mut both = hd_at(10);
    both.headers = carried.headers.clone();
    assert_eq!(
        doubts::spendable_eff(&depth(6), &both, &ds, None, &s),
        Ok(true)
    );
}

/// `spendableEff_today`: with no rival, no split and nothing dropped the
/// guard is the depth under the headers.
#[test]
fn spendable_eff_is_todays_with_no_doubt() {
    let mut headers = TestHeaders::default();
    let (s, _) = mined_at(&mut headers, 1, 100, 100);
    let ds = Doubts::cold(100);
    assert_eq!(
        doubts::spendable_eff(&depth(1), &headers, &ds, None, &s),
        Ok(true)
    );
    assert_eq!(
        doubts::spendable_eff(&depth(2), &headers, &ds, None, &s),
        Ok(false)
    );
    // The depth is the parameters' own: a later field moves nothing.
    let p = Params::new(60, 1).with_bound(Some(3));
    assert_eq!(doubts::spendable_eff(&p, &headers, &ds, None, &s), Ok(true));
}

/// `split_holds_coin` and `level_rival_holds_coin`: under a split at or
/// below the word's height, and under a rival level with the feed's tip
/// forking at or below it, nothing is spent at a depth of 1. A depth of 0
/// has no floor here, as the Lean's guard has none.
#[test]
fn spendable_eff_under_a_split_and_a_level_rival() {
    let mut headers = hd_at(105);
    let (s, _) = mined_at(&mut headers, 1, 100, 100);
    assert_eq!(headers.tip, 105);
    let clear = Doubts::cold(105);
    assert_eq!(
        doubts::spendable_eff(&depth(6), &headers, &clear, None, &s),
        Ok(true)
    );
    let split = Doubts::new(vec![], vec![("k95".into(), 95)], 105, None);
    assert_eq!(
        doubts::spendable_eff(&depth(1), &headers, &split, None, &s),
        Ok(false)
    );
    let at = Doubts::new(vec![], vec![("k100".into(), 100)], 105, None);
    assert_eq!(
        doubts::spendable_eff(&depth(1), &headers, &at, None, &s),
        Ok(false)
    );
    let above = Doubts::new(vec![], vec![("k101".into(), 101)], 105, None);
    assert_eq!(
        doubts::spendable_eff(&depth(1), &headers, &above, None, &s),
        Ok(true)
    );
    let level = Doubts::new(vec![Rival::new(100, 105)], vec![], 105, None);
    assert_eq!(
        doubts::spendable_eff(&depth(1), &headers, &level, None, &s),
        Ok(false)
    );
    let lower = Doubts::new(vec![Rival::new(99, 105)], vec![], 105, None);
    assert_eq!(
        doubts::spendable_eff(&depth(1), &headers, &lower, None, &s),
        Ok(false)
    );
    assert_eq!(
        doubts::spendable_eff(&depth(0), &headers, &split, None, &s),
        Ok(true)
    );
}

fn seen_alone(word: &Word) -> bool {
    matches!(word, Word::Seen)
}

fn every_word(_: &Word) -> bool {
    true
}

/// The guard's two lower arms: a word that claims nothing of the chain is
/// spent only under a rule that allows it; `Stale`, `Rejected` and
/// `Conflicted` never. No header is read: under headers that fault each
/// answer is `Ok`.
#[test]
fn spendable_eff_the_other_words() {
    let ok = TestHeaders::default();
    let mut unknown = State::new(hash(1));
    let mut built = unknown.clone();
    step(&mut built, &ok, Input::Host(HostAction::Build));
    let mut announced = built.clone();
    step(&mut announced, &ok, hint(HintStatus::Accepted, 0));
    let mut seen = announced.clone();
    step(&mut seen, &ok, hint(HintStatus::Seen, 1));
    let mut abandoned = built.clone();
    step(&mut abandoned, &ok, Input::Host(HostAction::Abandon));
    let claimless = [
        (&mut unknown, Word::Unknown),
        (&mut built, Word::Built),
        (&mut announced, Word::Announced),
        (&mut seen, Word::Seen),
        (&mut abandoned, Word::Abandoned),
    ];

    let mut carried = TestHeaders::default();
    let (mut stale, _) = mined_at(&mut carried, 3, 10, 10);
    carried.move_root(10, 50);
    step(
        &mut stale,
        &carried,
        Input::Evidence(Evidence::Chain(reorg(10))),
    );
    assert_eq!(stale.word(), &Word::Stale);
    let mut rejected = State::new(hash(5));
    step(
        &mut rejected,
        &ok,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
            reason: "no".into(),
        })),
    );
    let mut conflicted = State::new(hash(6));
    step(
        &mut conflicted,
        &ok,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(8)],
        })),
    );

    let fault = TestHeaders {
        fault: true,
        ..TestHeaders::default()
    };
    let ds = Doubts::cold(10);
    let seen_rule = AppRule::new("seen alone", seen_alone);
    let all = AppRule::new("every word", every_word);
    assert_eq!(all.name, "every word");
    for (state, word) in claimless {
        assert_eq!(state.word(), &word);
        assert_eq!(
            doubts::spendable_eff(&depth(1), &fault, &ds, None, state),
            Ok(false)
        );
        assert_eq!(
            doubts::spendable_eff(&depth(1), &fault, &ds, Some(&seen_rule), state),
            Ok(word == Word::Seen),
            "{word:?} under a rule that allows seen alone"
        );
    }
    for state in [&stale, &rejected, &conflicted] {
        assert_eq!(
            doubts::spendable_eff(&depth(1), &fault, &ds, Some(&all), state),
            Ok(false),
            "{:?} is never spent",
            state.word()
        );
    }
}

fn is_fault<T>(answer: &Result<T, CheckError>) -> bool {
    matches!(answer, Err(CheckError::Headers(_)))
}

/// K1: the crate's headers can fail. Each function that reads a header
/// answers its fault as an `Err`, never a pass; under an open split at or
/// below the height the depth is 0 before any header is read.
#[test]
fn a_headers_fault_is_an_error() {
    let mut carried = TestHeaders::default();
    let (s, p) = mined_at(&mut carried, 1, 5, 5);
    let fault = TestHeaders {
        fault: true,
        ..carried
    };
    let ds = Doubts::cold(10);
    assert!(is_fault(&doubts::eff_depth(&fault, &ds, 5)));
    assert!(is_fault(&doubts::lead_tip(&fault, &ds)));
    assert!(is_fault(&doubts::spendable_eff(
        &depth(1),
        &fault,
        &ds,
        None,
        &s
    )));
    assert!(is_fault(&doubts::may_enter(&fault, &ds, &p)));
    // A depth of 0 still reads the header for the root: a fault, not a pass.
    assert!(is_fault(&doubts::spendable_eff(
        &depth(0),
        &fault,
        &ds,
        None,
        &s
    )));
    let within = Checked::Within(ds.clone());
    assert!(is_fault(&doubts::eff_c(&fault, &within, 5)));
    assert!(is_fault(&doubts::spendable_c(
        &depth(1),
        &fault,
        &within,
        None,
        &s
    )));
    let split = Doubts::new(vec![], vec![("k5".into(), 5)], 10, None);
    assert_eq!(doubts::eff_depth(&fault, &split, 5), Ok(0));
    assert_eq!(doubts::may_enter(&fault, &split, &p), Ok(false));
    assert_eq!(
        doubts::spendable_eff(&depth(1), &fault, &split, None, &s),
        Ok(false)
    );
}

/// `Host.lean`'s `emptyFork`: a rival level at 100 holds 100; the empty
/// snapshot passes it; the least snapshot holds it.
#[test]
fn snapshot_empty_is_blind_to_a_fork() {
    let headers = hd_at(100);
    let u = unpruned(&Doubts::cold(99), &[tip_row(100), fork_row(100, 100, 100)]);
    assert_eq!(eff(&headers, &u, 100), 0);
    assert_eq!(eff(&headers, &Snapshot::empty(100).doubts(), 100), 1);
    assert_eq!(eff(&headers, &Snapshot::least(&u).doubts(), 100), 0);
}

/// `emptySplit`: the split at 95 holds 98; the empty snapshot passes 3.
#[test]
fn snapshot_empty_is_blind_to_a_split() {
    let headers = hd_at(100);
    let u = unpruned(
        &Doubts::cold(99),
        &[tip_row(100), keyed_split_row("k95", 95, 99, 100)],
    );
    assert_eq!(eff(&headers, &u, 98), 0);
    assert_eq!(eff(&headers, &Snapshot::empty(100).doubts(), 98), 3);
    assert_eq!(eff(&headers, &Snapshot::least(&u).doubts(), 98), 0);
}

fn two_splits() -> Doubts {
    unpruned(
        &Doubts::cold(10),
        &[
            keyed_split_row("k5", 5, 9, 10),
            keyed_split_row("k8", 8, 9, 10),
        ],
    )
}

/// `lowestSplitOnly`: the pair and the lowest split alone answer as the
/// host at every height, until the split at 5 resolves.
#[test]
fn snapshot_lowest_split_is_not_enough() {
    let headers = hd_at(10);
    let two = two_splits();
    let lowest = Snapshot::new(Snapshot::least(&two).rival, vec![("k5".into(), 5)], 10);
    for x in 0..12 {
        assert_eq!(
            eff(&headers, &lowest.doubts(), x),
            eff(&headers, &two, x),
            "at {x}"
        );
    }
    let resolved = [keyed_resolved_row("k5", 5)];
    assert_eq!(eff(&headers, &unpruned(&two, &resolved), 9), 0);
    assert_eq!(
        eff(
            &headers,
            &doubts::run(&B400, &lowest.doubts(), &resolved),
            9
        ),
        1
    );
    let least = Snapshot::least(&two).doubts();
    assert_eq!(eff(&headers, &doubts::run(&B400, &least, &resolved), 9), 0);
}

fn two_keys() -> Vec<ChainEvent> {
    vec![
        tip_row(60),
        keyed_split_row("ka", 50, 52, 53),
        keyed_split_row("kb", 50, 54, 55),
    ]
}

/// `oneKeyOfTwo` (cut 1f): one entry for a fork height is not enough; every
/// key is needed.
#[test]
fn snapshot_one_key_of_two_is_not_enough() {
    let headers = hd_at(60);
    let u = unpruned(&Doubts::cold(0), &two_keys());
    let one = Snapshot::new(Snapshot::least(&u).rival, vec![("ka".into(), 50)], 60);
    for x in 0..62 {
        assert_eq!(
            eff(&headers, &one.doubts(), x),
            eff(&headers, &u, x),
            "at {x}"
        );
    }
    let resolved = [keyed_resolved_row("ka", 50)];
    assert_eq!(eff(&headers, &unpruned(&u, &resolved), 50), 0);
    assert_eq!(
        eff(&headers, &doubts::run(&B400, &one.doubts(), &resolved), 50),
        6
    );
    let least = Snapshot::least(&u).doubts();
    assert_eq!(eff(&headers, &doubts::run(&B400, &least, &resolved), 50), 0);
}

/// `settledCut`: every rival, not only those above the settled line.
#[test]
fn snapshot_needs_every_rival() {
    let headers = hd_at(6);
    let three = [tip_row(5), fork_row(5, 5, 5), tip_row(406)];
    let back = [reorg_row(405, 406, 6)];
    let mut fallen = three.to_vec();
    fallen.extend_from_slice(&back);
    assert_eq!(eff(&headers, &unpruned(&Doubts::cold(4), &fallen), 5), 1);
    let empty = Snapshot::empty(406).doubts();
    assert_eq!(eff(&headers, &doubts::run(&B400, &empty, &back), 5), 2);
    let least = Snapshot::least(&unpruned(&Doubts::cold(4), &three)).doubts();
    assert_eq!(eff(&headers, &doubts::run(&B400, &least, &back), 5), 1);
}

/// `Snapshot.doubts`: no kept rival, the splits, the tip, the pair held as
/// the summary. A key read into a snapshot is lower-cased.
#[test]
fn snapshot_doubts_holds_the_pair_as_the_summary() {
    let snap = Snapshot::new(Some(Rival::new(0, 5)), vec![("k7".into(), 7)], 10);
    assert_eq!(
        snap.doubts().to_string(),
        "tip=10 splits=[k7:7] rivals=[] summary=0:5"
    );
    assert_eq!(Snapshot::empty(10).doubts(), Doubts::cold(10));
    let upper = Snapshot::new(None, vec![("AB12".into(), 7)], 10);
    assert_eq!(upper.splits, [("ab12".to_owned(), 7)]);
    // The pair is the merge of the summary and every kept rival.
    let ds = Doubts::new(
        vec![Rival::new(405, 406), Rival::new(9, 12)],
        vec![],
        406,
        Some(Rival::new(5, 5)),
    );
    assert_eq!(Snapshot::least(&ds).rival, Some(Rival::new(5, 406)));
}

// ---------------------------------------------------------------------------
// Cut 1g: the out host and the served snapshot (Host.lean section 12)
// ---------------------------------------------------------------------------

/// A delivery of the journal: a decoded row, or an `unreadable` one.
// A test's journal of a few rows: the variant's size costs nothing here.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
enum Delivered {
    Row(ChainEvent),
    Unreadable,
}

type Journal = Vec<(u64, Delivered)>;

/// The Lean's `journalFrom c`: the rows at cursors `c`, `c + 1`, ...
fn journal_from(c: u64, rows: Vec<ChainEvent>) -> Journal {
    (c..).zip(rows.into_iter().map(Delivered::Row)).collect()
}

/// The Lean's `HostC` over a `Host` with no word: the doubts, within or
/// out, the cursor, and the two counters.
#[derive(Clone, Debug)]
struct HostC {
    checked: Checked,
    cursor: u64,
    reads: u32,
    starts: u32,
}

/// The Lean's `hostIn`: a host in its contract at a cursor.
fn host_in(ds: Doubts, cursor: u64) -> HostC {
    HostC {
        checked: Checked::Within(ds),
        cursor,
        reads: 0,
        starts: 0,
    }
}

fn host_out(ds: Doubts, cursor: u64) -> HostC {
    HostC {
        checked: Checked::Out(ds),
        ..host_in(Doubts::cold(0), cursor)
    }
}

/// The Lean's `readC` under the policy `skip`: a host in its contract reads
/// the page its cursor answers (`run_checked` over its readable rows, the
/// cursor to the page's last row); an out host reads no row and makes one
/// ask, `sv`, taken when `start_from` takes it.
fn read_c(
    params: &Params,
    headers: &TestHeaders,
    j: &Journal,
    limit: usize,
    sv: Option<&Served>,
    c: HostC,
) -> HostC {
    if c.checked.is_out() {
        let started = sv.and_then(|s| doubts::start_from(params, headers, c.cursor, s).ok());
        let (checked, cursor) = match started {
            Some((ds, cursor)) => (Checked::Within(ds), cursor),
            None => (c.checked, c.cursor),
        };
        return HostC {
            checked,
            cursor,
            reads: c.reads + 1,
            starts: c.starts + 1,
        };
    }
    let page: Vec<&(u64, Delivered)> = j
        .iter()
        .filter(|(k, _)| *k > c.cursor)
        .take(limit)
        .collect();
    let rows: Vec<ChainEvent> = page
        .iter()
        .filter_map(|(_, d)| match d {
            Delivered::Row(e) => Some(e.clone()),
            Delivered::Unreadable => None,
        })
        .collect();
    let checked = doubts::run_checked(&B400, params, c.checked.doubts(), &rows);
    HostC {
        checked,
        cursor: page.last().map_or(c.cursor, |(k, _)| *k),
        reads: c.reads + 1,
        starts: c.starts,
    }
}

/// The Lean's `readsC`.
fn reads_c(
    params: &Params,
    headers: &TestHeaders,
    j: &Journal,
    reads: &[(usize, Option<&Served>)],
    c: HostC,
) -> HostC {
    reads.iter().fold(c, |c, (limit, sv)| {
        read_c(params, headers, j, *limit, *sv, c)
    })
}

/// The Lean's `keptAt j t c`: the host that kept every decoded row at or
/// below `c`, from cold at `t`.
fn kept_at(j: &Journal, t: u32, c: u64) -> Doubts {
    let rows: Vec<ChainEvent> = j
        .iter()
        .filter(|(k, _)| *k <= c)
        .filter_map(|(_, d)| match d {
            Delivered::Row(e) => Some(e.clone()),
            Delivered::Unreadable => None,
        })
        .collect();
    unpruned(&Doubts::cold(t), &rows)
}

/// The Lean's `honestAt`: the least snapshot of the kept host at `c`, folded
/// by a version-2 decoder that read every row, its tip's hash `h`.
fn honest_at(j: &Journal, t: u32, c: u64, h: &str) -> Served {
    Served::new(Snapshot::least(&kept_at(j, t, c)), c, 2, c, h)
}

/// The Lean's `S1`: the number 1, versions 1 and 2, and the Lean's `P6`'s
/// depth of 6.
fn s1() -> Params {
    Params::new(3600, 6).with_bound(Some(1))
}

fn j1() -> Journal {
    journal_from(
        1,
        vec![
            tip_row(100),
            tip_row(101),
            fork_row(100, 101, 102),
            tip_row(102),
            tip_row(103),
        ],
    )
}

/// A: `absentToday`. Under no number the checked host is today's host.
#[test]
fn absent_today() {
    let params = Params::new(3600, 6);
    assert_eq!(params.bound, None);
    let c = read_c(
        &params,
        &hd_at(103),
        &j1(),
        100,
        None,
        host_in(Doubts::cold(99), 0),
    );
    let events: Vec<ChainEvent> = j1()
        .into_iter()
        .map(|(_, d)| match d {
            Delivered::Row(e) => e,
            Delivered::Unreadable => unreachable!(),
        })
        .collect();
    assert!(!c.checked.is_out());
    assert_eq!(
        c.checked.doubts(),
        &doubts::run(&B400, &Doubts::cold(99), &events)
    );
    assert_eq!(c.cursor, 5);
}

fn j_ob() -> Journal {
    journal_from(
        1,
        vec![
            tip_row(60),
            keyed_split_row("ka", 50, 52, 53),
            keyed_split_row("kb", 51, 54, 55),
            keyed_resolved_row("ka", 50),
        ],
    )
}

/// B: `overThenBack`. Under 1 the row at cursor 3 leaves two keys open and
/// the page ends with one: the host is out and passes nothing at 58. Under 2
/// it is in.
#[test]
fn over_then_back() {
    let headers = hd_roots(60);
    let c = read_c(
        &s1(),
        &headers,
        &j_ob(),
        100,
        None,
        host_in(Doubts::cold(59), 0),
    );
    assert!(c.checked.is_out());
    assert_eq!(c.checked.doubts().splits().len(), 1);
    assert_eq!(c.cursor, 4);
    assert_eq!(doubts::eff_c(&headers, &c.checked, 58), Ok(0));
    let two = Params::new(3600, 6).with_bound(Some(2));
    let c = read_c(
        &two,
        &headers,
        &j_ob(),
        100,
        None,
        host_in(Doubts::cold(59), 0),
    );
    assert!(!c.checked.is_out());
    // Below the split at 51 the host in its contract answers its depth; the
    // out host answers nothing there.
    assert_eq!(doubts::eff_c(&headers, &c.checked, 50), Ok(8));
    let out = read_c(
        &s1(),
        &headers,
        &j_ob(),
        100,
        None,
        host_in(Doubts::cold(59), 0),
    );
    assert_eq!(doubts::eff_c(&headers, &out.checked, 50), Ok(0));
}

/// `extraKeyTrips`: a started host holding a key the kept host does not
/// trips on one new key; the kept host is within the number.
#[test]
fn extra_key_trips() {
    let row = [keyed_split_row("ka", 50, 52, 53)];
    let started = Snapshot::new(None, vec![("kx".into(), 40)], 60).doubts();
    assert!(doubts::run_checked(&B400, &s1(), &started, &row).is_out());
    assert_eq!(unpruned(&Doubts::cold(60), &row).splits().len(), 1);
    assert!(!doubts::run_checked(&B400, &s1(), &Doubts::cold(60), &row).is_out());
}

/// `outZero`: an out host under a spend depth of 0 passes a `mined` word
/// whose root its headers carry; under 6 it refuses; its depth is nothing.
#[test]
fn out_zero() {
    let mut headers = hd_roots(10);
    let (word5, _) = mined_at(&mut headers, 5, 5, 5);
    let out = Checked::Out(Doubts::cold(10));
    assert_eq!(
        doubts::spendable_c(&Params::new(3600, 0), &headers, &out, None, &word5),
        Ok(true)
    );
    assert_eq!(
        doubts::spendable_c(&depth(6), &headers, &out, None, &word5),
        Ok(false)
    );
    assert_eq!(
        doubts::spendable_c(&depth(1), &headers, &out, None, &word5),
        Ok(false)
    );
    assert_eq!(doubts::eff_c(&headers, &out, 5), Ok(0));
    // The same host within its contract passes it at 6.
    let within = Checked::Within(Doubts::cold(10));
    assert_eq!(
        doubts::spendable_c(&depth(6), &headers, &within, None, &word5),
        Ok(true)
    );
    // An out host reads no header for its depth, and refuses with none.
    let fault = TestHeaders {
        fault: true,
        ..headers
    };
    assert_eq!(doubts::eff_c(&fault, &out, 5), Ok(0));
    assert_eq!(
        doubts::spendable_c(&depth(1), &fault, &out, None, &word5),
        Ok(false)
    );
    // A word with no height reads no feed: today's guard, out or not.
    let seen_rule = AppRule::new("seen alone", seen_alone);
    let mut seen = State::new(hash(9));
    let ok = TestHeaders::default();
    step(&mut seen, &ok, Input::Host(HostAction::Build));
    step(&mut seen, &ok, hint(HintStatus::Accepted, 0));
    step(&mut seen, &ok, hint(HintStatus::Seen, 1));
    assert_eq!(
        doubts::spendable_c(&depth(6), &fault, &out, Some(&seen_rule), &seen),
        Ok(true)
    );
}

fn j_l() -> Journal {
    journal_from(
        1,
        vec![
            tip_row(60),
            keyed_split_row("ka", 50, 52, 53),
            keyed_split_row("kb", 50, 54, 55),
            keyed_resolved_row("ka", 50),
            keyed_split_row("kc", 50, 56, 57),
            keyed_resolved_row("kb", 50),
        ],
    )
}

/// `stuckAtCursor`: the host trips at cursor 3 and reads no row; the honest
/// snapshot at 3 holds the two keys and is refused; the one at 4 is taken.
#[test]
fn stuck_at_cursor() {
    let headers = hd_roots(60);
    let c = read_c(
        &s1(),
        &headers,
        &j_l(),
        3,
        None,
        host_in(Doubts::cold(59), 0),
    );
    assert!(c.checked.is_out());
    assert_eq!(c.cursor, 3);
    assert_eq!(
        doubts::accepts(&s1(), &headers, 3, &honest_at(&j_l(), 59, 3, "h60")),
        Err(Refused::Overfull { open: 2, bound: 1 })
    );
    assert_eq!(
        doubts::accepts(&s1(), &headers, 3, &honest_at(&j_l(), 59, 4, "h60")),
        Ok(())
    );
}

/// `startTripLoop`: start, trip, start: two starts over four page reads; one
/// start a page read at most, never a start that causes a start.
#[test]
fn start_trip_loop() {
    let headers = hd_roots(60);
    let (at4, at6) = (
        honest_at(&j_l(), 59, 4, "h60"),
        honest_at(&j_l(), 59, 6, "h60"),
    );
    let c = reads_c(
        &s1(),
        &headers,
        &j_l(),
        &[(3, None), (100, Some(&at4)), (100, None), (100, Some(&at6))],
        host_in(Doubts::cold(59), 0),
    );
    assert!(!c.checked.is_out());
    assert_eq!((c.starts, c.reads, c.cursor), (2, 4, 6));
    assert_eq!(c.checked.doubts().splits(), [("kc".to_owned(), 50)]);
    let c = reads_c(
        &s1(),
        &headers,
        &j_l(),
        &[(3, None), (100, Some(&at4)), (100, None)],
        host_in(Doubts::cold(59), 0),
    );
    assert!(c.checked.is_out());
    assert_eq!((c.starts, c.reads), (1, 3));
}

/// C: `overfullForever`. Served the over-full snapshot at every page read,
/// the host holds: its doubts and cursor as they were, a start a read.
#[test]
fn overfull_forever() {
    let headers = hd_roots(60);
    let at3 = honest_at(&j_l(), 59, 3, "h60");
    let mut reads = vec![(3, None)];
    reads.extend(std::iter::repeat_n((100, Some(&at3)), 5));
    let tripped = read_c(
        &s1(),
        &headers,
        &j_l(),
        3,
        None,
        host_in(Doubts::cold(59), 0),
    );
    let c = reads_c(
        &s1(),
        &headers,
        &j_l(),
        &reads,
        host_in(Doubts::cold(59), 0),
    );
    assert!(c.checked.is_out());
    assert_eq!((c.starts, c.reads, c.cursor), (5, 6, 3));
    assert_eq!(c.checked.doubts(), tripped.checked.doubts());
    assert_eq!(doubts::eff_c(&headers, &c.checked, 50), Ok(0));
}

/// `coversNotKept`: after a `split` row for a key the snapshot holds lower,
/// the kept entry is not an entry of the fold; the cover by key holds.
#[test]
fn covers_not_kept() {
    let row = [keyed_split_row("kx", 50, 52, 53)];
    let u = unpruned(&Doubts::cold(60), &row);
    let f = Snapshot::new(None, vec![("kx".into(), 40)], 60).fold(&B400, &row);
    assert!(!u.splits().iter().all(|p| f.splits.contains(p)));
    assert!(u
        .splits()
        .iter()
        .all(|(k, h)| f.splits.iter().any(|(q, g)| q == k && g <= h)));
}

/// D: `resolveUnknown`. A `splitResolved` for a key the fold does not hold
/// changes nothing.
#[test]
fn resolve_unknown() {
    let s = Snapshot::new(None, vec![("ka".into(), 50)], 60);
    assert_eq!(s.fold(&B400, &[keyed_resolved_row("kz", 50)]), s);
}

/// D: `leastCommutes`, its three rows: the service's fold of the least
/// snapshot is the least snapshot of the kept host.
#[test]
fn least_commutes() {
    let merge_rows = [
        tip_row(100),
        fork_row(0, 100, 100),
        tip_row(500),
        fork_row(150, 500, 500),
    ];
    let cases: [(Doubts, Vec<ChainEvent>); 3] = [
        (
            unpruned(&Doubts::cold(0), &merge_rows),
            vec![tip_row(551), reorg_row(500, 551, 520)],
        ),
        (
            unpruned(
                &Doubts::cold(4),
                &[tip_row(5), fork_row(5, 5, 5), tip_row(406)],
            ),
            vec![reorg_row(405, 406, 6)],
        ),
        (
            unpruned(&Doubts::cold(0), &two_keys()),
            vec![keyed_resolved_row("ka", 50), fork_row(40, 60, 61)],
        ),
    ];
    for (i, (u, rows)) in cases.iter().enumerate() {
        assert_eq!(
            Snapshot::least(u).fold(&B400, rows),
            Snapshot::least(&unpruned(u, rows)),
            "the row {i}"
        );
    }
}

/// E1: `skippedSplit`. A version-1 decoder skipped the `split` row; its
/// snapshot passes 11 at 50 where the kept host holds it. Refused by its
/// readable cursor; a version the host does not read is refused; the honest
/// version-2 snapshot is taken.
#[test]
fn skipped_split() {
    let headers = hd_roots(60);
    let j_v: Journal = vec![
        (1, Delivered::Row(tip_row(60))),
        (2, Delivered::Row(keyed_split_row("ka", 50, 52, 53))),
    ];
    let sv_v1 = Served::new(
        Snapshot::least(&unpruned(&Doubts::cold(59), &[tip_row(60)])),
        2,
        1,
        1,
        "h60",
    );
    assert_eq!(eff(&headers, &kept_at(&j_v, 59, 2), 50), 0);
    assert_eq!(eff(&headers, &sv_v1.snap.doubts(), 50), 11);
    assert_eq!(
        doubts::accepts(&s1(), &headers, 2, &sv_v1),
        Err(Refused::Unreadable {
            readable: 1,
            cursor: 2
        })
    );
    let v3 = with(&sv_v1, |s| {
        s.version = 3;
        s.readable = 2;
    });
    assert_eq!(
        doubts::accepts(&s1(), &headers, 2, &v3),
        Err(Refused::Version(3))
    );
    assert_eq!(
        doubts::accepts(&s1(), &headers, 2, &honest_at(&j_v, 59, 2, "h60")),
        Ok(())
    );
}

/// `afterStart`: a host started at 1 skips a `split` row of version 3 at 2
/// and passes 11 at 50, where a host that read the row holds it. The
/// snapshot's version covers the rows to its cursor; every later row's
/// version is the decoder's to check, and it refuses version 3.
#[test]
fn after_start() {
    let headers = hd_roots(60);
    let j_a: Journal = vec![(1, Delivered::Row(tip_row(60))), (2, Delivered::Unreadable)];
    let start = honest_at(&j_a, 59, 1, "h60");
    let c = reads_c(
        &s1(),
        &headers,
        &j_a,
        &[(100, Some(&start)), (100, None)],
        host_out(Doubts::cold(59), 1),
    );
    assert!(!c.checked.is_out());
    assert_eq!(c.cursor, 2);
    assert_eq!(doubts::eff_c(&headers, &c.checked, 50), Ok(11));
    let read = unpruned(
        &Doubts::cold(59),
        &[tip_row(60), keyed_split_row("kb", 50, 54, 55)],
    );
    assert_eq!(eff(&headers, &read, 50), 0);
    // The row at 2 is unreadable because the decoder checks its version.
    let v3 = include_str!("vectors/chain_events/split.json").replacen("\"v\":2", "\"v\":3", 1);
    assert_eq!(
        decode_envelope(v3.as_bytes()),
        Err(DecodeError::UnknownVersion(3))
    );
}

fn j_f() -> Journal {
    journal_from(1, vec![tip_row(60), fork_row(58, 60, 61)])
}

fn sv_b() -> Served {
    Served::new(Snapshot::empty(60), 2, 2, 2, "x60")
}

/// E2: `otherBranch`. A snapshot at 60 of another branch, with no rival,
/// passes 3 at 58 where the kept host refuses 1; its tip's hash is not the
/// host's header at 60, and it is refused.
#[test]
fn other_branch() {
    let headers = hd_roots(60);
    assert_eq!(eff(&headers, &kept_at(&j_f(), 59, 2), 58), 0);
    assert_eq!(eff(&headers, &sv_b().snap.doubts(), 58), 3);
    assert_eq!(
        doubts::accepts(&s1(), &headers, 2, &sv_b()),
        Err(Refused::TipOtherHash(60))
    );
}

/// `hashNotContent`: the same doubts under the host's own hash at 60 are
/// taken and pass 3 at 58. The hash names the tip and nothing else.
#[test]
fn hash_not_content() {
    let headers = hd_roots(60);
    let own = with(&sv_b(), |s| s.tip_hash = "h60".into());
    assert_eq!(doubts::accepts(&s1(), &headers, 2, &own), Ok(()));
    let c = read_c(
        &s1(),
        &headers,
        &j_f(),
        100,
        Some(&own),
        host_out(Doubts::cold(59), 2),
    );
    assert!(!c.checked.is_out());
    assert_eq!(doubts::eff_c(&headers, &c.checked, 58), Ok(3));
}

/// Each condition of `accepts` is refused by a value of its own, read in the
/// Lean's order; a snapshot refused on its own parts reads no header; a
/// headers fault is a refusal, never a start; a taken snapshot gives its
/// doubts and its cursor.
#[test]
fn each_refusal_is_a_value_of_its_own() {
    let headers = hd_roots(60);
    let fault = TestHeaders {
        fault: true,
        ..headers.clone()
    };
    let snap = Snapshot::new(Some(Rival::new(50, 52)), vec![("ka".into(), 50)], 60);
    let good = Served::new(snap.clone(), 5, 2, 5, "h60");
    assert_eq!(
        doubts::start_from(&s1(), &headers, 4, &good),
        Ok((snap.doubts(), 5))
    );
    assert_eq!(
        doubts::accepts(&s1(), &headers, 5, &good),
        Ok(()),
        "at the host's cursor"
    );
    // An upper-case hash names the same block.
    let upper = with(&good, |s| s.tip_hash = "H60".into());
    assert_eq!(doubts::accepts(&s1(), &headers, 5, &upper), Ok(()));
    let refusals = [
        (6, good.clone(), Refused::Behind { cursor: 5, host: 6 }),
        (5, with(&good, |s| s.version = 7), Refused::Version(7)),
        (
            5,
            with(&good, |s| s.readable = 4),
            Refused::Unreadable {
                readable: 4,
                cursor: 5,
            },
        ),
        (
            5,
            with(&good, |s| {
                s.snap = Snapshot::new(None, vec![("ka".into(), 50), ("kb".into(), 51)], 60);
            }),
            Refused::Overfull { open: 2, bound: 1 },
        ),
        (
            5,
            with(&good, |s| s.snap = Snapshot::empty(61)),
            Refused::TipAbsent(61),
        ),
        (
            5,
            with(&good, |s| s.tip_hash = "x60".into()),
            Refused::TipOtherHash(60),
        ),
    ];
    for (cursor, served, refused) in refusals {
        assert_eq!(
            doubts::accepts(&s1(), &headers, cursor, &served),
            Err(refused.clone())
        );
        assert_eq!(
            doubts::start_from(&s1(), &headers, cursor, &served),
            Err(refused.clone())
        );
        assert!(!refused.to_string().is_empty());
        // The four refusals on the snapshot's own parts read no header.
        if !matches!(refused, Refused::TipAbsent(_) | Refused::TipOtherHash(_)) {
            assert_eq!(
                doubts::accepts(&s1(), &fault, cursor, &served),
                Err(refused)
            );
        }
    }
    assert!(matches!(
        doubts::accepts(&s1(), &fault, 5, &good),
        Err(Refused::Headers(_))
    ));
    // Under no number no snapshot is over-full; a version the host names is read.
    let none = Params::new(3600, 6);
    let wide = with(&good, |s| {
        s.snap = Snapshot::new(None, vec![("ka".into(), 50), ("kb".into(), 51)], 60);
    });
    assert_eq!(doubts::accepts(&none, &headers, 5, &wide), Ok(()));
    let only_one = Params::new(3600, 6).with_versions(vec![1]);
    assert_eq!(
        doubts::accepts(&only_one, &headers, 5, &good),
        Err(Refused::Version(2))
    );
    assert_eq!(Params::new(3600, 6).versions, Params::VERSIONS);
}
