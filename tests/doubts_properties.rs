//! The doubts' theorems as randomized statements, each test named for the
//! Lean theorem it mirrors: `lean/Headers/Doubts.lean` and
//! `lean/Headers/Host.lean` of bsv-stack-lean at `08ce467d`, the statements as
//! they stand there (`docs/VERIFICATION.md`, "The doubts' theorems as
//! properties", is the table). 256 reproducible cases per test, using an
//! explicit ChaCha seed and no failure persistence, network, clock or
//! storage, as `tests/properties.rs`. No case is a journal row.
//!
//! The Lean's `runUnpruned`, `dropped`, `effDepthFeedTip`, `Tracker.spendable`
//! and `Tracker.rootChecks` are not items of the crate; each is mirrored here
//! as a local function under its name, and so is the host's page loop
//! (`readC`), which is the host's.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134). A test carries its Lean theorem's name, case and all.
#![allow(
    non_snake_case,
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
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use std::cell::Cell;

fn runner() -> TestRunner {
    TestRunner::new_with_rng(
        Config {
            cases: 256,
            failure_persistence: None,
            ..Config::default()
        },
        TestRng::deterministic_rng(RngAlgorithm::ChaCha),
    )
}

// ---------------------------------------------------------------------------
// The draw
// ---------------------------------------------------------------------------

/// The keys a split row names. `KA` is `ka` in another case: the row's key
/// is lower-cased where it is read, so the two are one key.
const KEYS: [&str; 4] = ["ka", "kb", "KA", "kc"];

/// Every height a test asks at: the draw's heights are at most 600, a
/// headers' tip at most 640.
const XS: std::ops::RangeInclusive<u32> = 0..=645;

/// The settled lines a case is drawn under: two short ones, where the prune
/// drops, and the feed's own.
const SETTLED: [u32; 3] = [2, 5, 400];

/// One row's draw: its kind, three heights and a key.
type RowParts = (u8, u32, u32, u32, usize);

fn row_parts() -> impl Strategy<Value = RowParts> {
    (
        0u8..10,
        0u32..=600,
        0u32..=600,
        0u32..=600,
        0usize..KEYS.len(),
    )
}

/// A row from its draw, its heights mapped by `h`: a `tip` row half the
/// time, a `fork`, a well-formed `reorg`, now and then the hand-built `reorg`
/// that removes no header, a `split` and a `splitResolved`.
fn row(h: &impl Fn(u32) -> u32, (kind, a, b, c, k): RowParts) -> ChainEvent {
    let (a, b, c) = (h(a), h(b), h(c));
    match kind {
        0..=4 => tip_row(a),
        5 => fork_row(a, b, c),
        6 => reorg_row(a, a + b % 24, c),
        7 => keyed_split_row(KEYS[k], a, b, c),
        8 => keyed_resolved_row(KEYS[k], a),
        _ if b % 4 == 0 => reorg_none(a, c),
        _ => fork_row(a, c, b),
    }
}

/// A case: a settled line, a start (any value the type holds), rows, more
/// rows after them, a headers' tip, and the map of the draw's heights.
#[derive(Clone, Debug)]
struct World {
    settled: u32,
    start: Doubts,
    rows: Vec<ChainEvent>,
    more: Vec<ChainEvent>,
    hd_tip: u32,
    base: u32,
    wide: bool,
}

impl World {
    fn h(&self, v: u32) -> u32 {
        heights(self.base, self.wide)(v)
    }
    fn bounds(&self) -> Bounds {
        Bounds::new(self.settled)
    }
}

/// Wide: a height anywhere in 0 to 600. Narrow: within 40 of `base`, where
/// rivals, splits and tips meet.
fn heights(base: u32, wide: bool) -> impl Fn(u32) -> u32 {
    move |v| if wide { v } else { base + v % 41 }
}

type StartParts = (Vec<(u32, u32)>, Vec<(usize, u32)>, u32, Option<(u32, u32)>);

fn start_parts() -> impl Strategy<Value = StartParts> {
    (
        prop::collection::vec((0u32..=600, 0u32..=600), 0..4),
        prop::collection::vec((0usize..3, 0u32..=600), 0..3),
        0u32..=600,
        prop::option::of((0u32..=600, 0u32..=600)),
    )
}

/// Any value the type holds: no invariant is asked of a start. The keys are
/// distinct, as the fold and the stored form hold them.
fn start(h: &impl Fn(u32) -> u32, (rivals, splits, tip, gone): StartParts) -> Doubts {
    let mut keys: Vec<(String, u32)> = Vec::new();
    for (k, fork) in splits {
        let key = ["ka", "kb", "kc"][k].to_owned();
        if !keys.iter().any(|(held, _)| *held == key) {
            keys.push((key, h(fork)));
        }
    }
    Doubts::new(
        rivals
            .into_iter()
            .map(|(f, t)| Rival::new(h(f), h(t)))
            .collect(),
        keys,
        h(tip),
        gone.map(|(f, t)| Rival::new(h(f), h(t))),
    )
}

fn world() -> impl Strategy<Value = World> {
    (
        0usize..SETTLED.len(),
        0u32..=560,
        any::<bool>(),
        start_parts(),
        prop::collection::vec(row_parts(), 0..40),
        prop::collection::vec(row_parts(), 0..20),
        0u32..=640,
    )
        .prop_map(|(s, base, wide, st, rows, more, hd)| {
            let h = heights(base, wide);
            World {
                settled: SETTLED[s],
                start: start(&h, st),
                rows: rows.iter().map(|p| row(&h, *p)).collect(),
                more: more.iter().map(|p| row(&h, *p)).collect(),
                hd_tip: if wide { hd } else { base + hd % 45 },
                base,
                wide,
            }
        })
}

// ---------------------------------------------------------------------------
// The local mirrors
// ---------------------------------------------------------------------------

/// The Lean's `hdAt t`: a tip and no header.
fn hd_at(tip: u32) -> TestHeaders {
    TestHeaders {
        tip,
        ..TestHeaders::default()
    }
}

fn eff(headers: &TestHeaders, ds: &Doubts, x: u32) -> u64 {
    doubts::eff_depth(headers, ds, x).unwrap()
}

/// The Lean's `Tracker.depth`.
fn depth(tip: u32, x: u32) -> u64 {
    (u64::from(tip) + 1).saturating_sub(u64::from(x))
}

/// The Lean's `runUnpruned`: `raw` folded, no prune.
fn unpruned(from: &Doubts, rows: &[ChainEvent]) -> Doubts {
    rows.iter().fold(from.clone(), |ds, e| doubts::raw(e, &ds))
}

/// The Lean's `keep`.
fn keep(b: &Bounds, tip: u32, r: &Rival) -> bool {
    u64::from(tip) <= u64::from(r.fork) + u64::from(b.settled)
}

/// The Lean's `dropped`: every entry the pruned host drops along a run, with
/// the tip at which it dropped it.
fn dropped(b: &Bounds, from: &Doubts, rows: &[ChainEvent]) -> Vec<(Rival, u32)> {
    let mut out = Vec::new();
    let mut ds = from.clone();
    for e in rows {
        let d = doubts::raw(e, &ds);
        for r in d.rivals() {
            if !keep(b, d.tip(), r) {
                out.push((r.clone(), d.tip()));
            }
        }
        ds = doubts::prune(b, &d);
    }
    out
}

/// The Lean's `splitBelow`.
fn split_below(ds: &Doubts, x: u32) -> bool {
    ds.splits().iter().any(|(_, f)| *f <= x)
}

/// The Lean's `leadBound`.
fn lead_bound(d: u64, t: u32, x: u32, ds: &Doubts) -> u64 {
    ds.held()
        .filter(|r| r.fork <= x)
        .map(|r| u64::from(t.saturating_sub(r.tip)))
        .fold(d, u64::min)
}

/// The Lean's `effDepthFeedTip`, cut 1d's effective depth: the lead under
/// the feed's tip alone.
fn eff_feed_tip(headers: &TestHeaders, ds: &Doubts, x: u32) -> u64 {
    if split_below(ds, x) {
        0
    } else {
        lead_bound(depth(headers.tip, x), ds.tip(), x, ds)
    }
}

/// The Lean's `Tracker.stillConfirmed`.
fn still_confirmed(headers: &TestHeaders, s: &State) -> bool {
    match s.word() {
        Word::Mined(m) => headers
            .headers
            .get(&m.height())
            .is_some_and(|hdr| hdr.merkle_root == m.checked().root()),
        _ => false,
    }
}

/// The Lean's `Tracker.spendable`, today's guard: the depth under the
/// headers alone.
fn spendable_today(
    params: &Params,
    headers: &TestHeaders,
    rule: Option<&AppRule>,
    s: &State,
) -> bool {
    match s.word() {
        Word::Mined(m) => {
            still_confirmed(headers, s) && params.spend_depth <= depth(headers.tip, m.height())
        }
        w @ (Word::Unknown | Word::Built | Word::Announced | Word::Seen | Word::Abandoned) => {
            rule.is_some_and(|r| (r.allows)(w))
        }
        Word::Stale | Word::Rejected { .. } | Word::Conflicted { .. } => false,
    }
}

/// The Lean's `Tracker.rootChecks`.
fn root_checks(headers: &TestHeaders, p: &Proof) -> bool {
    let root = p.root().unwrap();
    headers
        .headers
        .get(&p.height())
        .is_some_and(|hdr| hdr.merkle_root == root)
}

/// The Lean's `Cov l l'`: every entry of `l` covered by one of `l'`.
fn cov<'a>(l: impl Iterator<Item = &'a Rival>, by: &Doubts) -> bool {
    let by: Vec<&Rival> = by.held().collect();
    l.into_iter().all(|q| by.iter().any(|c| c.covers(q)))
}

/// The Lean's `SplitCov U P`: each key of `U` open in `P` at its height or
/// lower.
fn split_cov(u: &[(String, u32)], p: &[(String, u32)]) -> bool {
    u.iter()
        .all(|(k, f)| p.iter().any(|(pk, pf)| pk == k && pf <= f))
}

fn is_open(key: &str, ds: &Doubts) -> bool {
    ds.splits().iter().any(|(k, _)| k == key)
}

fn sorted_rivals(ds: &Doubts) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = ds.rivals().iter().map(|r| (r.fork, r.tip)).collect();
    v.sort_unstable();
    v
}

fn sorted_splits(ds: &Doubts) -> Vec<(String, u32)> {
    let mut v = ds.splits().to_vec();
    v.sort();
    v
}

/// Every doubts a case reads: its start, the run of its rows, and the run
/// that kept every row.
fn three(w: &World) -> [Doubts; 3] {
    [
        w.start.clone(),
        doubts::run(&w.bounds(), &w.start, &w.rows),
        unpruned(&w.start, &w.rows),
    ]
}

// ---------------------------------------------------------------------------
// The coins: words and paths under headers that carry real roots
// ---------------------------------------------------------------------------

fn allow_all(_: &Word) -> bool {
    true
}
fn allow_heard(w: &Word) -> bool {
    matches!(w, Word::Announced | Word::Seen)
}

fn rule(n: u8) -> Option<AppRule> {
    match n % 3 {
        0 => None,
        1 => Some(AppRule::new("all", allow_all)),
        _ => Some(AppRule::new("heard", allow_heard)),
    }
}

/// Headers and the states and paths over them: mined words whose root the
/// headers carry, mined words whose root has since moved, paths at heights
/// the headers do not carry, and each word that claims nothing of the chain.
struct Coins {
    headers: TestHeaders,
    states: Vec<State>,
    proofs: Vec<Proof>,
}

fn coin_parts() -> impl Strategy<Value = Vec<(u8, u32)>> {
    prop::collection::vec((0u8..8, 0u32..=600), 0..10)
}

fn coins(w: &World, parts: &[(u8, u32)]) -> Coins {
    let mut headers = hd_at(w.hd_tip);
    let mut states = Vec::new();
    let mut proofs = Vec::new();
    let mut moved = Vec::new();
    for (n, (kind, height)) in parts.iter().enumerate() {
        let txid = n as u64 + 10;
        let height = w.h(*height).max(1);
        let mut s = State::new(hash(txid));
        match kind {
            0..=2 => {
                let p = proof(&hash(txid), height, txid + 7);
                headers.insert(&p, u64::from(height) + 900);
                step(
                    &mut s,
                    &headers,
                    Input::Evidence(Evidence::Proof(p.clone())),
                );
                if *kind == 2 {
                    moved.push(height);
                }
                proofs.push(p);
            }
            3 => {
                step(&mut s, &headers, Input::Host(HostAction::Build));
            }
            4 => announce(&mut s, &headers),
            5 => {
                let _ = s.step(
                    &params(),
                    &headers,
                    Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
                        reason: "node".into(),
                    })),
                );
            }
            6 => {
                let _ = s.step(
                    &params(),
                    &headers,
                    Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
                        competitors: vec![hash(txid + 500)],
                    })),
                );
            }
            _ => {
                // A path at a height the headers may not carry.
                proofs.push(proof(&hash(txid), height + 1_000, txid + 7));
            }
        }
        states.push(s);
    }
    for height in moved {
        headers.move_root(height, u64::from(height) + 3_000);
    }
    Coins {
        headers,
        states,
        proofs,
    }
}

// ---------------------------------------------------------------------------
// Appendix C: the theorems of `lean/Headers/Doubts.lean`
// ---------------------------------------------------------------------------

/// `effDepth_le_depth`, `safe_either_lag` and `effDepth_le_lead_headers`:
/// whichever tip is ahead, the effective depth is at most the depth under
/// the host's headers, at most the lead under the feed's tip over every held
/// entry forking at or below `x`, and at most the lead under the headers'
/// tip over the same entries.
#[test]
fn safe_either_lag() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            for ds in three(&w) {
                for x in XS {
                    let e = eff(&hd, &ds, x);
                    prop_assert!(e <= depth(hd.tip, x), "{ds} at {x}");
                    for r in ds.held().filter(|r| r.fork <= x) {
                        prop_assert!(e <= u64::from(ds.tip().saturating_sub(r.tip)));
                        prop_assert!(e <= u64::from(hd.tip.saturating_sub(r.tip)));
                    }
                }
            }
            Ok(())
        })
        .unwrap();
}

/// `no_feed_is_today` and `live_no_feed`: the cold doubts at any tip, 0
/// included, and any doubts with no rival, no split and nothing dropped,
/// answer today's depth, today's guard (`Tracker.spendable`, mirrored) and
/// today's builder's test ("the root checks and two deep").
#[test]
fn no_feed_is_today() {
    runner()
        .run(
            &(world(), coin_parts(), 0u64..4, any::<u8>(), any::<bool>()),
            |(w, parts, d, r, zero)| {
                let c = coins(&w, &parts);
                let p = Params::new(60, d);
                let rule = rule(r);
                let t = if zero { 0 } else { w.start.tip() };
                for ds in [Doubts::cold(t), Doubts::new(vec![], vec![], t, None)] {
                    for x in XS {
                        prop_assert_eq!(eff(&c.headers, &ds, x), depth(c.headers.tip, x));
                    }
                    for s in &c.states {
                        prop_assert_eq!(
                            doubts::spendable_eff(&p, &c.headers, &ds, rule.as_ref(), s).unwrap(),
                            spendable_today(&p, &c.headers, rule.as_ref(), s),
                            "{:?}",
                            s.word()
                        );
                    }
                    for pr in &c.proofs {
                        prop_assert_eq!(
                            doubts::may_enter(&c.headers, &ds, pr).unwrap(),
                            root_checks(&c.headers, pr) && 2 <= depth(c.headers.tip, pr.height())
                        );
                    }
                }
                Ok(())
            },
        )
        .unwrap();
}

/// `run_safe`: from a cold start, after any rows, under any headers, at
/// every height, the pruned host's effective depth is at most the host's
/// that kept every row: what the pruned host passes, that host passes.
#[test]
fn run_safe() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            let cold = Doubts::cold(w.start.tip());
            let pruned = doubts::run(&w.bounds(), &cold, &w.rows);
            let kept = unpruned(&cold, &w.rows);
            for x in XS {
                prop_assert!(eff(&hd, &pruned, x) <= eff(&hd, &kept, x), "at {x}");
            }
            Ok(())
        })
        .unwrap();
}

/// `run_safe_from`: the same from any start the type holds.
#[test]
fn run_safe_from() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            let [_, pruned, kept] = three(&w);
            for x in XS {
                prop_assert!(eff(&hd, &pruned, x) <= eff(&hd, &kept, x), "at {x}");
            }
            Ok(())
        })
        .unwrap();
}

/// `run_exact`: from a cold start, for an act number `N` from 1 to 8, when
/// every entry the pruned host dropped (the Lean's `dropped`, mirrored)
/// stands `N` below the lesser of the run's tip and the headers' tip, the
/// pruned host passes `N` at a height exactly when the host that kept every
/// row does. Cut 1e reads the hypothesis under the lesser tip.
#[test]
fn run_exact() {
    let met = Cell::new(0u32);
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            let b = w.bounds();
            let cold = Doubts::cold(w.start.tip());
            let pruned = doubts::run(&b, &cold, &w.rows);
            let kept = unpruned(&cold, &w.rows);
            let gone = dropped(&b, &cold, &w.rows);
            let lesser = u64::from(pruned.tip().min(hd.tip));
            for n in 1u64..=8 {
                if !gone.iter().all(|(r, _)| u64::from(r.tip) + n <= lesser) {
                    continue;
                }
                if !gone.is_empty() {
                    met.set(met.get() + 1);
                }
                for x in XS {
                    prop_assert_eq!(
                        n <= eff(&hd, &pruned, x),
                        n <= eff(&hd, &kept, x),
                        "N {} at {}",
                        n,
                        x
                    );
                }
            }
            Ok(())
        })
        .unwrap();
    // The hypothesis held with something dropped: the property is not vacuous.
    assert!(met.get() > 0, "run_exact's hypothesis never met a drop");
}

/// `state_bound`: from a cold start, after every prefix of the rows, the
/// held entries forking at or below the tip number at most the settled line
/// plus two (the kept rivals plus one, the summary one more).
#[test]
fn state_bound() {
    runner()
        .run(&world(), |w| {
            let b = w.bounds();
            let mut ds = Doubts::cold(w.start.tip());
            for e in &w.rows {
                ds = doubts::step(&b, e, &ds);
                let n = ds.held().filter(|r| r.fork <= ds.tip()).count();
                prop_assert!(n as u64 <= u64::from(w.settled) + 2, "{n} held: {ds}");
            }
            Ok(())
        })
        .unwrap();
}

/// `reorg_holds_until_headers`: after a `reorg` row at fork height `fh`
/// whose removed branch tops at `m`, from any start and after any rows that
/// follow, while the headers' tip is at most `m`, every height at or above
/// `fh` has an effective depth of nothing.
#[test]
fn reorg_holds_until_headers() {
    runner()
        .run(&(world(), row_parts()), |(w, (_, a, b, c, _))| {
            let (fh, m) = (w.h(a), w.h(a) + w.h(b) % 24);
            let reorg = reorg_row(fh, m, w.h(c));
            let bounds = w.bounds();
            let after = doubts::step(&bounds, &reorg, &w.start);
            let ds = doubts::run(&bounds, &after, &w.rows);
            let hd = hd_at(w.hd_tip % (m + 1));
            for x in fh..=*XS.end() {
                prop_assert_eq!(eff(&hd, &ds, x), 0, "at {} under {}: {}", x, hd.tip, ds);
            }
            Ok(())
        })
        .unwrap();
}

/// `stale_feed_only_unseen`: two hosts over one set of headers, the stale
/// one's feed tip at most the fresh one's (here the run and the run of more
/// rows after it, ordered by their tips). Where the stale host passes `N` at
/// `x` and the fresh one refuses, the fresh host holds an open split at or
/// below `x` the stale one does not, or an entry forking at or below `x`,
/// fewer than `N` below the lesser of its tip and the headers', that no
/// entry of the stale host covers. The conclusion only weakens as `N` rises,
/// so the least such `N`, one more than the fresh host's depth, is asked.
#[test]
fn stale_feed_only_unseen() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            let b = w.bounds();
            let a = doubts::run(&b, &w.start, &w.rows);
            let z = doubts::run(&b, &a, &w.more);
            let (stale, fresh) = if a.tip() <= z.tip() { (a, z) } else { (z, a) };
            for x in XS {
                let (es, ef) = (eff(&hd, &stale, x), eff(&hd, &fresh, x));
                if ef >= es {
                    continue;
                }
                let n = ef + 1;
                let unseen_split = fresh
                    .splits()
                    .iter()
                    .any(|p| p.1 <= x && !stale.splits().contains(p));
                let lesser = u64::from(fresh.tip().min(hd.tip));
                let unseen_rival = fresh.held().any(|r| {
                    r.fork <= x
                        && lesser < u64::from(r.tip) + n
                        && stale.held().all(|q| !q.covers(r))
                });
                prop_assert!(
                    unseen_split || unseen_rival,
                    "stale {} passes {} at {} where fresh {} refuses",
                    stale,
                    n,
                    x,
                    fresh
                );
            }
            Ok(())
        })
        .unwrap();
}

/// `lesser_cost_bound`, `effDepth_le_feedTip` and `effDepth_eq_feedTip`,
/// with cut 1d's `effDepthFeedTip` mirrored: the effective depth is never
/// above cut 1d's, cut 1d's exceeds it by at most the distance the headers
/// stand behind the feed, and the two are equal when the feed is not ahead.
#[test]
fn lesser_cost_bound() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            for ds in three(&w) {
                let lag = u64::from(ds.tip().saturating_sub(hd.tip));
                for x in XS {
                    let (e, ft) = (eff(&hd, &ds, x), eff_feed_tip(&hd, &ds, x));
                    prop_assert!(e <= ft, "{ds} at {x}");
                    prop_assert!(ft <= e + lag, "{ds} at {x}");
                    if ds.tip() <= hd.tip {
                        prop_assert_eq!(e, ft);
                    }
                }
            }
            Ok(())
        })
        .unwrap();
}

/// `safety` and `spendableEff_imp_spendable`: under any doubts, the
/// effective guard spends nothing today's guard (`Tracker.spendable`,
/// mirrored) would not, and a path the builder's test admits has its root
/// active at its height and is two deep under the host's headers.
#[test]
fn safety() {
    runner()
        .run(
            &(world(), coin_parts(), 0u64..4, any::<u8>()),
            |(w, parts, d, r)| {
                let c = coins(&w, &parts);
                let p = Params::new(60, d);
                let rule = rule(r);
                for ds in three(&w) {
                    for s in &c.states {
                        if doubts::spendable_eff(&p, &c.headers, &ds, rule.as_ref(), s).unwrap() {
                            prop_assert!(
                                spendable_today(&p, &c.headers, rule.as_ref(), s),
                                "{:?} under {}",
                                s.word(),
                                ds
                            );
                        }
                    }
                    for pr in &c.proofs {
                        if doubts::may_enter(&c.headers, &ds, pr).unwrap() {
                            prop_assert!(root_checks(&c.headers, pr));
                            prop_assert!(2 <= depth(c.headers.tip, pr.height()));
                        }
                    }
                }
                Ok(())
            },
        )
        .unwrap();
}

// ---------------------------------------------------------------------------
// The start from a snapshot: `lean/Headers/Host.lean` sections 6 and 12
// ---------------------------------------------------------------------------

/// `coldFrom_safe` with `least_covers`: `U` the doubts of a host that kept
/// every row from any start; the least snapshot of `U` covers it, and a host
/// started from that snapshot passes nothing, after any later rows, that the
/// host that kept every row refuses.
#[test]
fn coldFrom_safe() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            let u = unpruned(&w.start, &w.rows);
            let s = Snapshot::least(&u);
            prop_assert_eq!(s.tip, u.tip());
            prop_assert!(u.splits().iter().all(|p| s.splits.contains(p)));
            prop_assert!(cov(u.held(), &s.doubts()));
            let started = doubts::run(&w.bounds(), &s.doubts(), &w.more);
            let kept = unpruned(&u, &w.more);
            for x in XS {
                prop_assert!(eff(&hd, &started, x) <= eff(&hd, &kept, x), "at {x}");
            }
            Ok(())
        })
        .unwrap();
}

/// `least_enough`: a host started from the least snapshot of the host that
/// kept every row from cold up to a point is never more permissive, over any
/// later rows, than that host continued.
#[test]
fn least_enough() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            let cold = Doubts::cold(w.start.tip());
            let s = Snapshot::least(&unpruned(&cold, &w.rows));
            let started = doubts::run(&w.bounds(), &s.doubts(), &w.more);
            let all: Vec<ChainEvent> = w.rows.iter().chain(&w.more).cloned().collect();
            let kept = unpruned(&cold, &all);
            for x in XS {
                prop_assert!(eff(&hd, &started, x) <= eff(&hd, &kept, x), "at {x}");
            }
            Ok(())
        })
        .unwrap();
}

/// `least_fold_tip_splits` and `least_fold_coversK` (cut 1g, D): the
/// service's fold of the least snapshot has the tip and the open splits of
/// the least snapshot of the run, exactly, and covers by key the run of the
/// host it was taken from (one tip, each open key at its height or lower,
/// each held entry covered).
#[test]
fn least_fold_coversK() {
    runner()
        .run(&world(), |w| {
            let b = w.bounds();
            let u = unpruned(&w.start, &w.rows);
            let folded = Snapshot::least(&u).fold(&b, &w.more);
            let later = unpruned(&u, &w.more);
            let least = Snapshot::least(&later);
            prop_assert_eq!(folded.tip, least.tip);
            prop_assert_eq!(&folded.splits, &least.splits);
            let fd = folded.doubts();
            prop_assert_eq!(fd.tip(), later.tip());
            prop_assert!(split_cov(later.splits(), fd.splits()));
            prop_assert!(cov(later.held(), &fd));
            Ok(())
        })
        .unwrap();
}

// ---------------------------------------------------------------------------
// The keyed split (cut 1f): `lean/Headers/Doubts.lean` section 7d
// ---------------------------------------------------------------------------

/// A drawn split row and its key as the fold holds it.
fn split_of(w: &World, (_, a, b, c, k): RowParts) -> (ChainEvent, String, u32, u32) {
    let (fh, lo, hi) = (w.h(a), w.h(b), w.h(c));
    (
        keyed_split_row(KEYS[k], fh, lo, hi),
        KEYS[k].to_ascii_lowercase(),
        fh,
        lo.min(hi),
    )
}

/// `split_row_holds`: after any `split` row for any key, open or not, from
/// any doubts, every height at or above its fork height has an effective
/// depth of nothing, under any headers: under headers that fault too, since
/// the hold is answered before a header is read.
#[test]
fn split_row_holds() {
    runner()
        .run(&(world(), row_parts()), |(w, parts)| {
            let (e, _, fh, _) = split_of(&w, parts);
            let faulty = TestHeaders {
                fault: true,
                ..hd_at(w.hd_tip)
            };
            for ds in three(&w) {
                let after = doubts::step(&w.bounds(), &e, &ds);
                for x in fh..=*XS.end() {
                    prop_assert_eq!(eff(&hd_at(w.hd_tip), &after, x), 0);
                    prop_assert_eq!(doubts::eff_depth(&faulty, &after, x), Ok(0));
                }
            }
            Ok(())
        })
        .unwrap();
}

/// `split_new_key`: a `split` row for a key not open puts the key at the
/// front with the row's fork height.
#[test]
fn split_new_key() {
    let met = Cell::new(0u32);
    runner()
        .run(&(world(), row_parts()), |(w, parts)| {
            let (e, key, fh, _) = split_of(&w, parts);
            for ds in three(&w).iter().filter(|ds| !is_open(&key, ds)) {
                met.set(met.get() + 1);
                let after = doubts::step(&w.bounds(), &e, ds);
                let mut want = vec![(key.clone(), fh)];
                want.extend_from_slice(ds.splits());
                prop_assert_eq!(after.splits(), &want[..]);
            }
            Ok(())
        })
        .unwrap();
    assert!(met.get() > 0);
}

/// `split_open_key`: a `split` row for a key already open keeps its place,
/// and its stored height becomes the lower of the stored height and the
/// row's; every other entry is as it was.
#[test]
fn split_open_key() {
    let met = Cell::new(0u32);
    runner()
        .run(&(world(), row_parts()), |(w, parts)| {
            let (e, key, fh, _) = split_of(&w, parts);
            for ds in three(&w).iter().filter(|ds| is_open(&key, ds)) {
                met.set(met.get() + 1);
                let after = doubts::step(&w.bounds(), &e, ds);
                let want: Vec<(String, u32)> = ds
                    .splits()
                    .iter()
                    .map(|(k, f)| (k.clone(), if *k == key { (*f).min(fh) } else { *f }))
                    .collect();
                prop_assert_eq!(after.splits(), &want[..]);
            }
            Ok(())
        })
        .unwrap();
    assert!(met.get() > 0);
}

/// `split_adds_rival`: a `split` row adds its lower witness tip as a rival
/// at its fork height, key open or not, as a `fork` row with that competitor
/// adds it; the tip and the summary stay.
#[test]
fn split_adds_rival() {
    runner()
        .run(&(world(), row_parts()), |(w, parts)| {
            let (e, _, fh, low) = split_of(&w, parts);
            for ds in three(&w) {
                let after = doubts::raw(&e, &ds);
                let as_fork = doubts::raw(&fork_row(fh, fh, low), &ds);
                prop_assert_eq!(after.rivals(), as_fork.rivals());
                prop_assert_eq!(after.tip(), ds.tip());
                prop_assert_eq!(after.gone(), ds.gone());
            }
            Ok(())
        })
        .unwrap();
}

/// `splitResolved_mem` and `splitResolved_ends_key`: after a `splitResolved`
/// row for a key, an entry is open exactly when it was open and carries
/// another key, so the row's key is not open.
#[test]
fn splitResolved_mem() {
    runner()
        .run(&(world(), row_parts()), |(w, (_, a, _, _, k))| {
            let e = keyed_resolved_row(KEYS[k], w.h(a));
            let key = KEYS[k].to_ascii_lowercase();
            for ds in three(&w) {
                let after = doubts::step(&w.bounds(), &e, &ds);
                let want: Vec<&(String, u32)> =
                    ds.splits().iter().filter(|(kk, _)| *kk != key).collect();
                let got: Vec<&(String, u32)> = after.splits().iter().collect();
                prop_assert_eq!(got, want);
                prop_assert!(!is_open(&key, &after));
            }
            Ok(())
        })
        .unwrap();
}

/// `splitResolved_unknown`: a `splitResolved` row for a key not open changes
/// nothing.
#[test]
fn splitResolved_unknown() {
    let met = Cell::new(0u32);
    runner()
        .run(&(world(), row_parts()), |(w, (_, a, _, _, k))| {
            let e = keyed_resolved_row(KEYS[k], w.h(a));
            let key = KEYS[k].to_ascii_lowercase();
            for ds in three(&w).iter().filter(|ds| !is_open(&key, ds)) {
                met.set(met.get() + 1);
                prop_assert_eq!(&doubts::raw(&e, ds), ds);
            }
            Ok(())
        })
        .unwrap();
    assert!(met.get() > 0);
}

/// `splitResolved_reads_no_height`: a `splitResolved` row's fork height is
/// not read.
#[test]
fn splitResolved_reads_no_height() {
    runner()
        .run(&(world(), row_parts()), |(w, (_, a, b, _, k))| {
            let one = keyed_resolved_row(KEYS[k], w.h(a));
            let other = keyed_resolved_row(KEYS[k], w.h(b));
            for ds in three(&w) {
                prop_assert_eq!(doubts::raw(&one, &ds), doubts::raw(&other, &ds));
            }
            Ok(())
        })
        .unwrap();
}

/// `splitResolved_keeps_other`: after a `splitResolved` row for a key, every
/// open entry of another key holds every height at or above its own.
#[test]
fn splitResolved_keeps_other() {
    runner()
        .run(&(world(), row_parts()), |(w, (_, a, _, _, k))| {
            let e = keyed_resolved_row(KEYS[k], w.h(a));
            let key = KEYS[k].to_ascii_lowercase();
            let hd = hd_at(w.hd_tip);
            for ds in three(&w) {
                let after = doubts::step(&w.bounds(), &e, &ds);
                for (kk, f) in ds.splits().iter().filter(|(kk, _)| *kk != key) {
                    for x in *f..=*XS.end() {
                        prop_assert_eq!(eff(&hd, &after, x), 0, "{} at {}", kk, x);
                    }
                }
            }
            Ok(())
        })
        .unwrap();
}

/// `split_holds_until_resolved` and `run_keeps_hold`: an open split holds
/// through any rows that carry no `splitResolved` for its key: the key stays
/// open at its height or lower, and every height at or above its height has
/// an effective depth of nothing. No tip, reorg, settled line or other key
/// ends it.
#[test]
fn split_holds_until_resolved() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            let ds = doubts::run(&w.bounds(), &w.start, &w.rows);
            for (key, f) in ds.splits() {
                let rows: Vec<ChainEvent> = w
                    .more
                    .iter()
                    .filter(|e| match e {
                        ChainEvent::SplitResolved { key: k, .. } => {
                            k.strip_prefix("split:").unwrap().to_ascii_lowercase() != *key
                        }
                        _ => true,
                    })
                    .cloned()
                    .collect();
                let after = doubts::run(&w.bounds(), &ds, &rows);
                prop_assert!(after.splits().iter().any(|(k, g)| k == key && g <= f));
                for x in *f..=*XS.end() {
                    prop_assert_eq!(eff(&hd, &after, x), 0);
                }
            }
            Ok(())
        })
        .unwrap();
}

/// `splits_never_shrink`: no row but a `splitResolved` lowers the count of
/// open splits, at every step of a run from any start.
#[test]
fn splits_never_shrink() {
    runner()
        .run(&world(), |w| {
            let b = w.bounds();
            let mut ds = w.start.clone();
            for e in &w.rows {
                let after = doubts::step(&b, e, &ds);
                if !matches!(e, ChainEvent::SplitResolved { .. }) {
                    prop_assert!(ds.splits().len() <= after.splits().len());
                }
                ds = after;
            }
            Ok(())
        })
        .unwrap();
}

/// `run_perm` (K5 whole): two hosts alike but for the order of their kept
/// rivals and of their open splits stay so after any rows (the same rivals
/// and splits as sets, one tip, one summary), and give one effective depth
/// at every height.
#[test]
fn run_perm() {
    runner()
        .run(&(world(), any::<usize>()), |(w, turn)| {
            let hd = hd_at(w.hd_tip);
            let s = &w.start;
            let mut rivals = s.rivals().to_vec();
            rivals.reverse();
            let mut splits = s.splits().to_vec();
            if !splits.is_empty() {
                let n = turn % splits.len();
                splits.rotate_left(n);
            }
            let other = Doubts::new(rivals, splits, s.tip(), s.gone().cloned());
            let b = w.bounds();
            let (one, two) = (
                doubts::run(&b, s, &w.rows),
                doubts::run(&b, &other, &w.rows),
            );
            prop_assert_eq!(sorted_rivals(&one), sorted_rivals(&two));
            prop_assert_eq!(sorted_splits(&one), sorted_splits(&two));
            prop_assert_eq!(one.tip(), two.tip());
            prop_assert_eq!(one.gone(), two.gone());
            for x in XS {
                prop_assert_eq!(eff(&hd, &one, x), eff(&hd, &two, x));
            }
            Ok(())
        })
        .unwrap();
}

// ---------------------------------------------------------------------------
// The bound and the out host (cut 1g): `lean/Headers/Host.lean` section 12
// ---------------------------------------------------------------------------

/// `trips_iff`, with `within_never_trips` and `run_splits`: under a named
/// number `n`, the checked run is out exactly when some row leaves more than
/// `n` keys open in the run that kept every row; the check is made after
/// every row and drops nothing, so its doubts are `run`'s either way.
#[test]
fn trips_iff() {
    let outs = Cell::new(0u32);
    runner()
        .run(&(world(), 0u32..4), |(w, n)| {
            let b = w.bounds();
            let p = Params::new(60, 1).with_bound(Some(n));
            let checked = doubts::run_checked(&b, &p, &w.start, &w.rows);
            let want = (0..w.rows.len())
                .any(|k| unpruned(&w.start, &w.rows[..=k]).splits().len() > n as usize);
            prop_assert_eq!(checked.is_out(), want);
            if want {
                outs.set(outs.get() + 1);
            }
            let ran = doubts::run(&b, &w.start, &w.rows);
            prop_assert_eq!(checked.doubts(), &ran);
            let kept = unpruned(&w.start, &w.rows);
            prop_assert_eq!(ran.splits(), kept.splits());
            Ok(())
        })
        .unwrap();
    assert!(outs.get() > 0);
}

/// `trips_none` and `absent_is_today`: with the number absent no run is
/// out, and the doubts are `run`'s.
#[test]
fn trips_none() {
    runner()
        .run(&world(), |w| {
            let b = w.bounds();
            let checked = doubts::run_checked(&b, &Params::new(60, 1), &w.start, &w.rows);
            prop_assert_eq!(checked, Checked::Within(doubts::run(&b, &w.start, &w.rows)));
            Ok(())
        })
        .unwrap();
}

/// `out_passes_nothing`: an out host's effective depth is nothing at every
/// height, and no `mined` word is spendable under a spend depth of 1 or
/// more; a word with no height reads no feed and is today's guard, out or
/// not.
#[test]
fn out_passes_nothing() {
    runner()
        .run(
            &(world(), coin_parts(), 0u64..4, any::<u8>()),
            |(w, parts, d, r)| {
                let c = coins(&w, &parts);
                let p = Params::new(60, d);
                let rule = rule(r);
                for ds in three(&w) {
                    let out = Checked::Out(ds);
                    for x in XS {
                        prop_assert_eq!(doubts::eff_c(&c.headers, &out, x), Ok(0));
                    }
                    for s in &c.states {
                        let got =
                            doubts::spendable_c(&p, &c.headers, &out, rule.as_ref(), s).unwrap();
                        match s.word() {
                            Word::Mined(_) if d >= 1 => prop_assert!(!got),
                            Word::Mined(_) => {}
                            _ => prop_assert_eq!(
                                got,
                                spendable_today(&p, &c.headers, rule.as_ref(), s)
                            ),
                        }
                    }
                }
                Ok(())
            },
        )
        .unwrap();
}

/// `effC_le`: the checked host is never more permissive than its host; in
/// its contract it is its host.
#[test]
fn effC_le() {
    runner()
        .run(&world(), |w| {
            let hd = hd_at(w.hd_tip);
            for ds in three(&w) {
                for x in XS {
                    let plain = eff(&hd, &ds, x);
                    let within = Checked::Within(ds.clone());
                    prop_assert_eq!(doubts::eff_c(&hd, &within, x), Ok(plain));
                    let out = Checked::Out(ds.clone());
                    prop_assert!(doubts::eff_c(&hd, &out, x).unwrap() <= plain);
                }
            }
            Ok(())
        })
        .unwrap();
}

/// The headers of a start: at each height up to 40 but those `missing`
/// names, the header `h<height>`.
fn start_headers(missing: u64) -> TestHeaders {
    let mut headers = hd_at(40);
    for h in 0..=40u32 {
        if missing >> (h % 64) & 1 == 0 {
            headers.headers.insert(
                h,
                Header {
                    hash: format!("h{h}"),
                    merkle_root: format!("r{h}"),
                },
            );
        }
    }
    headers
}

type ServedParts = (u64, u64, u64, u64, usize, u32, bool);

fn served_parts() -> impl Strategy<Value = ServedParts> {
    (
        0u64..8,
        0u64..8,
        0u64..4,
        0u64..8,
        0usize..4,
        0u32..=44,
        any::<bool>(),
    )
}

fn served(
    (_, cursor, version, readable, open, tip, own): ServedParts,
    rival: Option<Rival>,
) -> Served {
    let splits = (0..open).map(|i| (format!("k{i}"), tip)).collect();
    let hash = if own {
        format!("h{tip}")
    } else {
        format!("x{tip}")
    };
    Served::new(
        Snapshot::new(rival, splits, tip),
        cursor,
        version,
        readable,
        hash,
    )
}

/// `accepts_iff`, with `refuse_version`, `refuse_unreadable`,
/// `overfull_refused` and `refuse_tip`: a host takes a served snapshot
/// exactly when it is at the host's cursor or past it, of a version the host
/// reads, readable through its own cursor, within the named number, and its
/// tip is the host's own verified header by height and hash; a refused
/// snapshot builds no doubts, and a taken one gives the snapshot's doubts and
/// cursor. The crate names the earliest condition that fails, in the Lean's
/// order.
#[test]
fn accepts_iff() {
    let taken = Cell::new(0u32);
    runner()
        .run(
            &(
                served_parts(),
                prop::option::of(0u32..4),
                0u8..4,
                any::<u64>(),
            ),
            |(parts, bound, vs, missing)| {
                let host = parts.0;
                let sv = served(parts, None);
                let versions: Vec<u64> = (1..=2).filter(|v| vs >> (v - 1) & 1 == 1).collect();
                let p = Params::new(60, 1)
                    .with_bound(bound)
                    .with_versions(versions.clone());
                let hd = start_headers(missing);
                let behind = sv.cursor < host;
                let version = !versions.contains(&sv.version);
                let unreadable = sv.readable < sv.cursor;
                let over = bound.is_some_and(|n| sv.snap.splits.len() > n as usize);
                let header = hd.headers.get(&sv.snap.tip);
                let tip_ok = header.is_some_and(|h| h.hash == sv.tip_hash);
                let want = !behind && !version && !unreadable && !over && tip_ok;
                let got = doubts::accepts(&p, &hd, host, &sv);
                prop_assert_eq!(got.is_ok(), want, "{:?}", got);
                let started = doubts::start_from(&p, &hd, host, &sv);
                if want {
                    taken.set(taken.get() + 1);
                    prop_assert_eq!(started, Ok((sv.snap.doubts(), sv.cursor)));
                } else {
                    let earliest = if behind {
                        Refused::Behind {
                            cursor: sv.cursor,
                            host,
                        }
                    } else if version {
                        Refused::Version(sv.version)
                    } else if unreadable {
                        Refused::Unreadable {
                            readable: sv.readable,
                            cursor: sv.cursor,
                        }
                    } else if over {
                        Refused::Overfull {
                            open: sv.snap.splits.len(),
                            bound: bound.unwrap(),
                        }
                    } else if header.is_none() {
                        Refused::TipAbsent(sv.snap.tip)
                    } else {
                        Refused::TipOtherHash(sv.snap.tip)
                    };
                    prop_assert_eq!(got, Err(earliest.clone()));
                    prop_assert_eq!(started, Err(earliest));
                }
                Ok(())
            },
        )
        .unwrap();
    assert!(taken.get() > 0);
}

/// The Lean's `HostC` over a host with no word: the doubts, within or out,
/// the cursor and the two counters.
#[derive(Clone, Debug)]
struct HostC {
    checked: Checked,
    cursor: u64,
    reads: u32,
    starts: u32,
}

/// The Lean's `readC` under the policy `skip`, the host's page loop: a host
/// in its contract reads the page its cursor answers (`run_checked` over its
/// rows); an out host reads no row and makes one ask, `sv`, taken when
/// `start_from` takes it.
fn read_c(
    b: &Bounds,
    p: &Params,
    hd: &TestHeaders,
    j: &[(u64, ChainEvent)],
    limit: usize,
    sv: Option<&Served>,
    c: HostC,
) -> HostC {
    if c.checked.is_out() {
        let (checked, cursor) = match sv.and_then(|s| doubts::start_from(p, hd, c.cursor, s).ok()) {
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
    let page: Vec<&(u64, ChainEvent)> = j
        .iter()
        .filter(|(k, _)| *k > c.cursor)
        .take(limit)
        .collect();
    let rows: Vec<ChainEvent> = page.iter().map(|(_, e)| e.clone()).collect();
    HostC {
        checked: doubts::run_checked(b, p, c.checked.doubts(), &rows),
        cursor: page.last().map_or(c.cursor, |(k, _)| *k),
        reads: c.reads + 1,
        starts: c.starts,
    }
}

/// `starts_le_reads`, with `readC_counts`, `in_never_starts`, `leaves_out`
/// and `refused_stays`, over the host's page loop (`readC`, mirrored): every
/// page read counts one read and at most one start (one start a page read at
/// most); a host in its contract makes no start; an out host that takes a
/// snapshot is in, at the snapshot's doubts and cursor; one that is refused,
/// or has no answer, stays as it was; and over every run of page reads the
/// starts are at most the reads.
#[test]
fn starts_le_reads() {
    let (began, refused) = (Cell::new(0u32), Cell::new(0u32));
    let reads = prop::collection::vec(
        (1usize..4, prop::option::of(served_parts()), any::<u8>()),
        1..12,
    );
    runner()
        .run(
            &(world(), reads, 0u32..3, any::<u64>()),
            |(w, reads, n, missing)| {
                let b = w.bounds();
                let p = Params::new(60, 1).with_bound(Some(n));
                let hd = start_headers(missing & 0xff);
                let j: Vec<(u64, ChainEvent)> = (1..).zip(w.rows.iter().cloned()).collect();
                let mut c = HostC {
                    checked: Checked::Within(w.start.clone()),
                    cursor: 0,
                    reads: 0,
                    starts: 0,
                };
                for (limit, sv, r) in reads {
                    let sv = sv.map(|parts| {
                        let kept =
                            unpruned(&w.start, &w.rows[..w.rows.len().min(parts.1 as usize)]);
                        let rival = Snapshot::least(&kept).rival;
                        served(parts, if r % 2 == 0 { rival } else { None })
                    });
                    let before = c.clone();
                    c = read_c(&b, &p, &hd, &j, limit, sv.as_ref(), c);
                    prop_assert_eq!(c.reads, before.reads + 1);
                    prop_assert!(c.starts <= before.starts + 1);
                    if !before.checked.is_out() {
                        prop_assert_eq!(c.starts, before.starts);
                        continue;
                    }
                    match sv
                        .as_ref()
                        .filter(|s| doubts::accepts(&p, &hd, before.cursor, s).is_ok())
                    {
                        Some(s) => {
                            began.set(began.get() + 1);
                            prop_assert_eq!(&c.checked, &Checked::Within(s.snap.doubts()));
                            prop_assert_eq!(c.cursor, s.cursor);
                        }
                        None => {
                            refused.set(refused.get() + 1);
                            prop_assert_eq!(&c.checked, &before.checked);
                            prop_assert_eq!(c.cursor, before.cursor);
                        }
                    }
                    prop_assert!(c.starts <= c.reads);
                }
                prop_assert!(c.starts <= c.reads);
                Ok(())
            },
        )
        .unwrap();
    assert!(began.get() > 0 && refused.get() > 0);
}

// ---------------------------------------------------------------------------
// `registerAncestor` (`lean/Headers/Doubts.lean` section 6)
// ---------------------------------------------------------------------------

/// `registerAncestor_is_onProof`: the Lean's `registerAncestor` is no new
/// item here. It is the tracker's own proof step on a new state: a proven
/// ancestor is `State::new(txid)` stepped with its path, and every theorem
/// of the tracker about a proven word holds of it. The word is `mined` at the
/// path's height, with the path and the header it was checked against as its
/// evidence, nothing suspect and no re-ask, exactly when the path is the
/// ancestor's and the headers carry its root at its height; otherwise no
/// word is written. (The crate's check also asks the headers' tip at or
/// above the height; every header drawn here is at or below the tip.)
#[test]
fn registerAncestor_is_onProof() {
    runner()
        .run(
            &(1u32..600, 0u8..4, any::<bool>(), 0u32..700),
            |(height, case, own, tip)| {
                let txid = hash(7);
                let leaf = if own { hash(7) } else { hash(8) };
                let p = proof(&leaf, height, 99);
                let mut hd = hd_at(tip);
                match case {
                    0 | 1 => hd.insert(&p, u64::from(height)),
                    2 => hd.move_root(height, u64::from(height)),
                    _ => {}
                }
                let registered = own && root_checks(&hd, &p) && hd.tip >= height;
                let mut s = State::new(txid.clone());
                let result = s.step(&params(), &hd, Input::Evidence(Evidence::Proof(p.clone())));
                if registered {
                    prop_assert!(result.is_ok());
                    prop_assert_eq!(s.word().height(), Some(height));
                    let checked = s.evidence().unwrap();
                    prop_assert_eq!(checked.proof(), &p);
                    prop_assert_eq!(checked.header(), hd.headers.get(&height).unwrap());
                    prop_assert!(!s.suspect());
                    prop_assert_eq!(s.reask(), None);
                } else {
                    prop_assert!(result.is_err());
                    prop_assert_eq!(s.word(), &Word::Unknown);
                    prop_assert!(s.evidence().is_none());
                }
                Ok(())
            },
        )
        .unwrap();
}
