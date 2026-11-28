//! A host's doubts: the rivals the header service's feed has shown it, the
//! open splits and the feed's tip, folded one row at a time.
//!
//! The module mirrors `lean/Headers/Doubts.lean` of bsv-stack-lean at
//! `656c502a`, under the Lean's names: [`Rival`] (`Rival`, with `covers` and
//! `merge`), [`Doubts`] (`Doubts`, with `cold` and `held`), [`Bounds`]
//! (`Bounds`, and `B400`), [`raw`], [`prune`], [`step`] and [`run`]. The
//! private helpers `add`, `rekey`, `rekey_all`, `absorb`, `top_height`,
//! `low_tip`, `is_open`, `open_split`, `close_split` and `keep` are the
//! Lean's `add`, `rekey`, `rekeyAll`, `absorb`, `topHeight`, `lowTip`,
//! `isOpen`, `openSplit`, `closeSplit` and `keep`.
//!
//! # The open splits, keyed (cut 1f)
//!
//! An open split is a pair, its key and its fork height, and nothing more
//! (the Lean's section 7d; the stack's `docs/DECISIONS-2026-10-19.md` 30.7).
//! The key is the hash of the two branches' common ancestor: the row's
//! `key` with its `split:` prefix taken off (a key without the prefix, which
//! the decoder refuses, is read whole). Two keys can be open at one height.
//! A `split` row for a key not open puts its pair at the front; a `split`
//! row for a key already open keeps its place, and the stored height
//! becomes the lower of the stored one and the row's. A `splitResolved` row
//! removes the pair of its key and reads no height; a key not open changes
//! nothing.
//!
//! # The feed's named number (cut 1g)
//!
//! A feed may name the most keys it can have open at once; the host holds
//! that number as [`Params::bound`], never as anything a snapshot or a row
//! carries. [`step_checked`] and [`run_checked`] count the open keys after
//! every row, not at a page's end, and answer [`Checked::Out`] when some row
//! leaves more open than the number: the feed is out of its contract. The
//! check drops no split and merges no key: the doubts in the answer are
//! [`step`]'s and [`run`]'s, every row applied. With the number absent the
//! answer is always [`Checked::Within`] and its doubts are [`step`]'s and
//! [`run`]'s. What an out host then does (it reads no row and starts again
//! from a snapshot at its cursor or past it) is the host's. The stack's
//! `lean/Headers/Host.lean` section 12: [`over`] is its `over`, and
//! [`run_checked`] its `trips` beside `run`.
//!
//! The doubts are a pure function of the feed's rows. No function of the fold
//! ([`raw`], [`prune`], [`step`], [`run`] and the checked two) reads a header,
//! a clock or the network, and none reads a root, a time or a chain work of an
//! event: a `tip` row gives its height, a `fork` row its height
//! and its competitor's height, a `reorg` row its fork height, the highest
//! removed height and its new tip's height, a `split` row its key, its fork
//! height and its lower witness tip's height, a `splitResolved` row its key.
//!
//! # The depth, the guard and the builder's test
//!
//! The headers enter in the reads alone, never in the fold. [`lead_tip`],
//! [`eff_depth`], [`spendable_eff`] and [`may_enter`] are the Lean's
//! `leadTip`, `effDepth`, `spendableEff` and `mayEnter` (`lean/Headers/
//! Doubts.lean` sections 2, 4 and 5 of bsv-stack-lean at `7e616ab3`), with
//! [`AppRule`] (`Tracker.AppRule`) and the private mirrors `depth`,
//! `still_confirmed`, `root_checks` and `rule_allows` of `lean/Tracker.lean`'s
//! `depth`, `stillConfirmed`, `rootChecks` and `ruleAllows`. The Lean's headers
//! are total; the crate's [`Headers`] can fail, and a fault is an `Err` in
//! each function that reads a header, never a pass. The spend depth is the
//! host's own, [`Params::spend_depth`], set when it builds its parameters.
//!
//! # The out host and the snapshot (cut 1g)
//!
//! A host whose feed is out of its contract ([`Checked::Out`]) reads no row,
//! reads its depths through [`eff_c`] (nothing at any height) and its guard
//! through [`spendable_c`], which passes no `Mined` word under a spend depth
//! of 1 or more, and makes one ask for a snapshot at each page read: one
//! start a page read at most, never a start that causes a start. Those reads
//! are the Lean's `effC` and `spendableC` (`lean/Headers/Host.lean` section
//! 12); the page loop is the host's. A snapshot is a [`Snapshot`] (section 6:
//! `Snapshot`, `empty`, `least`, `doubts`, and section 12's `fold`), served as
//! a [`Served`] (its cursor, the folding decoder's row version, its readable
//! cursor and its tip's hash), and taken through [`accepts`] or
//! [`start_from`] alone, refused by a [`Refused`] value of its own for each
//! condition: behind the host's cursor, a version [`Params::versions`] does
//! not name, a row before its cursor unreadable to its decoder, more open
//! splits than [`Params::bound`], and a tip that is not the host's own
//! verified header by height and hash.
//!
//! The doubts sit beside the tracker's reducer and never inside it: a host
//! that reads one feed row calls [`step`] and then
//! [`Tracker::on_chain`](crate::Tracker::on_chain), which takes the event by
//! value.
//!
//! # The stored form
//!
//! [`Doubts`] has one stored form, its serde text, with the fields in the
//! Lean's order and under the Lean's names:
//!
//! ```json
//! {"rivals":[{"fork":101,"tip":101}],"splits":[],"tip":103,"gone":null}
//! ```
//!
//! An open split is stored as a pair, `["<key>", <fork height>]`:
//! `"splits":[["k95",95]]`, in list order. All four fields are required. `gone` is `null` or a rival; a text without
//! it is refused and never read as "nothing dropped", since forgetting the
//! summary is the permissive direction. A field's name never changes, a later
//! minor only adds a field, and this reader refuses a text holding a field it
//! does not know. A host that cannot read its stored doubts reports a fault;
//! it does not start cold in silence. The reader checks shape and no
//! invariant: any value the type holds is a legal start for [`run`].
//!
//! # The printed line
//!
//! `Display` prints `tip=103 splits=[] rivals=[101:101] summary=-`: the
//! feed's tip, each open split as `key:height` in list order (the newest key
//! at the front, the key in full), each kept rival
//! as `fork:tip` in list order, and the summary as `fork:tip` or `-`. The
//! stored field `gone` is printed as `summary=`.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    ChainEvent, CheckError, EventHeader, Hash, HeaderError, Headers, Height, Params, Proof,
    SplitTip, State, Word,
};

/// A rival branch the host has been shown: the height at which it forked
/// from the host's chain (one above the last common block, the feed's
/// `fork.height` and `reorg.forkHeight`) and the height of its highest
/// header the host has seen. The Lean's `Rival`.
///
/// `#[non_exhaustive]`: build one with [`Rival::new`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Rival {
    /// The height at which the rival forked from the host's chain.
    pub fork: Height,
    /// The height of the rival's highest header the host has seen.
    pub tip: Height,
}

impl Rival {
    /// A rival that forked at `fork` and whose highest seen header is at `tip`.
    #[must_use]
    pub const fn new(fork: Height, tip: Height) -> Self {
        Self { fork, tip }
    }

    /// Whether this rival covers `other`: it forked at or below `other` and
    /// its tip is at least `other`'s, so wherever `other` binds an act this
    /// one binds it at least as hard. The Lean's `Rival.covers`.
    #[must_use]
    pub fn covers(&self, other: &Self) -> bool {
        self.fork <= other.fork && other.tip <= self.tip
    }

    /// The pair that covers both: the lower fork height and the higher tip.
    /// The Lean's `Rival.merge`.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.fork.min(other.fork), self.tip.max(other.tip))
    }
}

/// A host's doubts: the rivals it has been shown, the open splits (each its
/// key and its fork height), the feed's tip at the host's cursor, and the summary of what
/// the prune has dropped. The Lean's `Doubts`.
///
/// The fields are private and read through [`Doubts::rivals`],
/// [`Doubts::splits`], [`Doubts::tip`] and [`Doubts::gone`]. The lists keep
/// the Lean's order, so two hosts on one run hold equal values and equal
/// stored texts. See the [module's page](self) for the stored form.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Doubts {
    rivals: Vec<Rival>,
    // A stored form that names one key twice is refused: the fold holds a key
    // once, and a reader that took either entry would drop the other's height.
    #[serde(deserialize_with = "distinct_keys")]
    splits: Vec<(Hash, Height)>,
    tip: Height,
    // Required on read: with no `default`, a text that leaves the field out
    // is refused where serde's derive would read an absent option as `None`.
    #[serde(deserialize_with = "required_gone")]
    gone: Option<Rival>,
}

fn distinct_keys<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<(Hash, Height)>, D::Error> {
    let splits = Vec::<(Hash, Height)>::deserialize(deserializer)?;
    for (i, (key, _)) in splits.iter().enumerate() {
        if splits[..i].iter().any(|(held, _)| held == key) {
            return Err(serde::de::Error::custom(format!(
                "the open split key {key} is named twice"
            )));
        }
    }
    Ok(splits)
}

fn required_gone<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Rival>, D::Error> {
    Option::<Rival>::deserialize(deserializer)
}

impl Doubts {
    /// The doubts holding exactly these parts: the kept rivals and the open
    /// splits (each its key and its fork height) in list order, the feed's tip, and the summary of
    /// what was dropped. No invariant is asked of them: any value is a legal
    /// start for [`run`].
    #[must_use]
    pub const fn new(
        rivals: Vec<Rival>,
        splits: Vec<(Hash, Height)>,
        tip: Height,
        gone: Option<Rival>,
    ) -> Self {
        Self {
            rivals,
            splits,
            tip,
            gone,
        }
    }

    /// A host with no rows read: no rival, no split, nothing dropped, the tip
    /// it starts from. The Lean's `Doubts.cold`.
    #[must_use]
    pub const fn cold(tip: Height) -> Self {
        Self::new(Vec::new(), Vec::new(), tip, None)
    }

    /// The kept rivals, in list order: the newest entry is at the front.
    #[must_use]
    pub fn rivals(&self) -> &[Rival] {
        &self.rivals
    }

    /// The open splits, each its key and its fork height, in list order: the
    /// newest key is at the front. A key is in the list at most once.
    #[must_use]
    pub fn splits(&self) -> &[(Hash, Height)] {
        &self.splits
    }

    /// The feed's tip at the host's cursor.
    #[must_use]
    pub const fn tip(&self) -> Height {
        self.tip
    }

    /// The summary of what the prune has dropped: the lowest fork height and
    /// the highest tip of every dropped entry, re-keyed by every `reorg` row
    /// since. `None` until something is dropped.
    #[must_use]
    pub const fn gone(&self) -> Option<&Rival> {
        self.gone.as_ref()
    }

    /// Every entry a depth reads: the summary of what was dropped, then the
    /// kept rivals. The Lean's `Doubts.held`.
    pub fn held(&self) -> impl Iterator<Item = &Rival> + '_ {
        self.gone.iter().chain(self.rivals.iter())
    }
}

/// The printed line, from `tip=` on: `tip=103 splits=[] rivals=[101:101]
/// summary=-`. See the [module's page](self).
impl fmt::Display for Doubts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tip={} splits=[", self.tip)?;
        for (i, (key, fork)) in self.splits.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{key}:{fork}")?;
        }
        f.write_str("] rivals=[")?;
        for (i, rival) in self.rivals.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{}:{}", rival.fork, rival.tip)?;
        }
        f.write_str("] summary=")?;
        match &self.gone {
            Some(gone) => write!(f, "{}:{}", gone.fork, gone.tip),
            None => f.write_str("-"),
        }
    }
}

/// The host's number for pruning: the settled line's depth, the depth below
/// which the feed names no reorg. The Lean's `Bounds`.
///
/// `#[non_exhaustive]`: build one with [`Bounds::new`], or take
/// [`Bounds::B400`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Bounds {
    /// The settled line's depth, in blocks.
    pub settled: u32,
}

impl Bounds {
    /// The feed's settled line, 400. The Lean's `B400`.
    pub const B400: Self = Self::new(400);

    /// The bounds with the settled line at `settled` blocks.
    #[must_use]
    pub const fn new(settled: u32) -> Self {
        Self { settled }
    }
}

/// The Lean's `add`: nothing when an entry already covers `rival`; otherwise
/// `rival` at the front, then every old entry it does not cover.
fn add(rival: Rival, list: Vec<Rival>) -> Vec<Rival> {
    if list.iter().any(|held| held.covers(&rival)) {
        return list;
    }
    let mut out = Vec::with_capacity(list.len() + 1);
    out.extend(list.into_iter().filter(|held| !rival.covers(held)));
    out.insert(0, rival);
    out
}

/// The Lean's `rekey`: a rival that forked above a reorg's fork height is
/// keyed at that fork height.
fn rekey(fork_height: Height, rival: &Rival) -> Rival {
    if fork_height < rival.fork {
        Rival::new(fork_height, rival.tip)
    } else {
        rival.clone()
    }
}

/// The Lean's `rekeyAll`, a `foldr`: every rival re-keyed, from the back of
/// the list, the list rebuilt through `add`.
fn rekey_all(fork_height: Height, list: &[Rival]) -> Vec<Rival> {
    list.iter()
        .rev()
        .fold(Vec::new(), |acc, rival| add(rekey(fork_height, rival), acc))
}

/// The Lean's `absorb`, a `foldr`: the summary `gone` with every entry of
/// `list` merged into it, from the back of the list.
fn absorb(gone: Option<Rival>, list: &[Rival]) -> Option<Rival> {
    list.iter().rev().fold(gone, |acc, rival| {
        Some(match acc {
            None => rival.clone(),
            Some(held) => rival.merge(&held),
        })
    })
}

/// The Lean's `topHeight`: the highest of the removed headers' heights, the
/// removed branch's tip in a `reorg` row; `None` when the row names none.
fn top_height(headers: &[EventHeader]) -> Option<Height> {
    headers.iter().map(|header| header.height).max()
}

/// The Lean's `lowTip`: the lower of a split's witness tips' heights.
fn low_tip(tips: &[SplitTip]) -> Option<Height> {
    tips.iter().map(|tip| tip.height).min()
}

/// The key of a split row: its `key` with the `split:` prefix taken off, the
/// common ancestor's hash, in lower case. A key without the prefix is read
/// whole. The decoder accepts a hash in either case and a key is compared as
/// text, so the key is lower-cased where it is read from the wire: a `split`
/// row and its `splitResolved` row name one key in whatever case each is
/// sent.
fn split_key(key: &str) -> String {
    key.strip_prefix("split:")
        .unwrap_or(key)
        .to_ascii_lowercase()
}

/// The Lean's `isOpen`: an entry of the open splits carries the key.
fn is_open(key: &str, list: &[(Hash, Height)]) -> bool {
    list.iter().any(|(held, _)| held == key)
}

/// The Lean's `openSplit`: a key not open goes to the front with the row's
/// fork height; a key open keeps its place and its stored height becomes the
/// lower of the stored height and the row's.
fn open_split(key: &str, fork_height: Height, list: &[(Hash, Height)]) -> Vec<(Hash, Height)> {
    if is_open(key, list) {
        list.iter()
            .map(|(held, fork)| {
                if held == key {
                    (held.clone(), (*fork).min(fork_height))
                } else {
                    (held.clone(), *fork)
                }
            })
            .collect()
    } else {
        let mut out = Vec::with_capacity(list.len() + 1);
        out.push((key.to_owned(), fork_height));
        out.extend_from_slice(list);
        out
    }
}

/// The Lean's `closeSplit`: the entry with the key is removed; no height is
/// read.
fn close_split(key: &str, list: &[(Hash, Height)]) -> Vec<(Hash, Height)> {
    list.iter()
        .filter(|(held, _)| held != key)
        .cloned()
        .collect()
}

/// The Lean's `keep`: an entry is kept while its fork height is not past the
/// settled line under the tip `tip`.
fn keep(bounds: &Bounds, tip: Height, rival: &Rival) -> bool {
    u64::from(tip) <= u64::from(rival.fork) + u64::from(bounds.settled)
}

/// The table of rows, one row at a time, with no prune. The Lean's `raw`.
///
/// - `tip`: the tip becomes the row's height.
/// - `fork`: the competitor (the second of the two tips; the other is the
///   served tip) is added as a rival at the row's fork height.
/// - `reorg`: the tip becomes the new tip's height; the summary and every
///   rival forking above the row's fork height are re-keyed at it; and, when
///   the row removes a header, the removed branch is added as a rival at the
///   fork height with the highest removed height as its tip.
/// - `split`: a key not open is put at the front of the open splits with the
///   row's fork height; a key open keeps its place and takes the lower of its
///   stored height and the row's. Either way the lower of the two witness
///   tips is added as a rival at the row's fork height.
/// - `splitResolved`: the open split of the row's key closes, and no other.
///   The row's fork height is not read; a key not open changes nothing.
/// - `invalidated`, `frozen`, `tipAge`: nothing.
///
/// Adding a rival keeps only entries no other entry covers: nothing is added
/// when a kept entry covers the new one, and the new one takes out every
/// kept entry it covers.
#[must_use]
pub fn raw(event: &ChainEvent, ds: &Doubts) -> Doubts {
    match event {
        ChainEvent::Tip { height, .. } => Doubts {
            tip: *height,
            ..ds.clone()
        },
        ChainEvent::Fork {
            height,
            competing_tips: [_, competitor],
            ..
        } => Doubts {
            rivals: add(Rival::new(*height, competitor.height), ds.rivals.clone()),
            ..ds.clone()
        },
        ChainEvent::Reorg {
            fork_height,
            deactivated_headers,
            new_tip,
            ..
        } => {
            let rekeyed = rekey_all(*fork_height, &ds.rivals);
            Doubts {
                rivals: match top_height(deactivated_headers) {
                    Some(top) => add(Rival::new(*fork_height, top), rekeyed),
                    None => rekeyed,
                },
                splits: ds.splits.clone(),
                tip: new_tip.height,
                gone: ds.gone.as_ref().map(|gone| rekey(*fork_height, gone)),
            }
        }
        ChainEvent::Split {
            key,
            fork_height,
            tips,
        } => {
            let splits = open_split(&split_key(key), *fork_height, &ds.splits);
            Doubts {
                rivals: match low_tip(tips) {
                    Some(low) => add(Rival::new(*fork_height, low), ds.rivals.clone()),
                    None => ds.rivals.clone(),
                },
                splits,
                tip: ds.tip,
                gone: ds.gone.clone(),
            }
        }
        ChainEvent::SplitResolved { key, .. } => Doubts {
            splits: close_split(&split_key(key), &ds.splits),
            ..ds.clone()
        },
        ChainEvent::Invalidated { .. } | ChainEvent::Frozen { .. } | ChainEvent::TipAge { .. } => {
            ds.clone()
        }
    }
}

/// The prune: the entries the settled line keeps stay, in order; every other
/// entry is merged into the summary of what was dropped. The Lean's `prune`.
///
/// An entry is kept while the tip is at most its fork height plus
/// `bounds.settled`.
#[must_use]
pub fn prune(bounds: &Bounds, ds: &Doubts) -> Doubts {
    let (kept, dropped): (Vec<Rival>, Vec<Rival>) = ds
        .rivals
        .iter()
        .cloned()
        .partition(|rival| keep(bounds, ds.tip, rival));
    Doubts {
        rivals: kept,
        splits: ds.splits.clone(),
        tip: ds.tip,
        gone: absorb(ds.gone.clone(), &dropped),
    }
}

/// The step per row: [`raw`], then [`prune`]. The Lean's `step`.
///
/// The prune runs on every row, so a run of `fork` rows with no `tip` row
/// between them is bounded too.
#[must_use]
pub fn step(bounds: &Bounds, event: &ChainEvent, ds: &Doubts) -> Doubts {
    prune(bounds, &raw(event, ds))
}

/// The doubts after every row of `rows`, in order, from `ds`: [`step`]
/// folded from the left. The Lean's `run`.
#[must_use]
pub fn run<'a, I>(bounds: &Bounds, ds: &Doubts, rows: I) -> Doubts
where
    I: IntoIterator<Item = &'a ChainEvent>,
{
    rows.into_iter()
        .fold(ds.clone(), |acc, event| step(bounds, event, &acc))
}

/// Whether the open splits of `ds` number more than the feed's named number,
/// [`Params::bound`]. With the number absent, never. The Lean's `over`.
#[must_use]
pub fn over(params: &Params, ds: &Doubts) -> bool {
    params
        .bound
        .is_some_and(|bound| u64::try_from(ds.splits.len()).map_or(true, |n| u64::from(bound) < n))
}

/// The answer of a checked step or run: the doubts after every row read,
/// and whether the feed stayed within its named number after each of them.
///
/// Either way the doubts are the unchecked fold's, every split kept.
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "an `Out` answer is the feed out of its contract"]
pub enum Checked {
    /// No row left more keys open than [`Params::bound`], or the number is
    /// absent.
    Within(Doubts),
    /// Some row left more keys open than [`Params::bound`]: the feed is out
    /// of its contract. The doubts after every row read, no split dropped.
    Out(Doubts),
}

impl Checked {
    /// The doubts after every row read, within or out.
    #[must_use]
    pub fn doubts(&self) -> &Doubts {
        match self {
            Self::Within(ds) | Self::Out(ds) => ds,
        }
    }

    /// The doubts after every row read, within or out.
    #[must_use]
    pub fn into_doubts(self) -> Doubts {
        match self {
            Self::Within(ds) | Self::Out(ds) => ds,
        }
    }

    /// Whether some row left more keys open than the named number.
    #[must_use]
    pub fn is_out(&self) -> bool {
        matches!(self, Self::Out(_))
    }
}

/// [`step`], then the count of open keys against [`Params::bound`]: one row
/// that leaves more open than the number answers [`Checked::Out`].
pub fn step_checked(bounds: &Bounds, params: &Params, event: &ChainEvent, ds: &Doubts) -> Checked {
    let after = step(bounds, event, ds);
    if over(params, &after) {
        Checked::Out(after)
    } else {
        Checked::Within(after)
    }
}

/// [`run`], with the count of open keys checked after every row: the answer
/// is [`Checked::Out`] when some row of `rows` left more open than
/// [`Params::bound`], even if a later row brings the count back within it.
/// Every row is applied either way. The start `ds` itself is not counted:
/// with no rows the answer is [`Checked::Within`]. The Lean's `trips`, beside
/// `run`.
pub fn run_checked<'a, I>(bounds: &Bounds, params: &Params, ds: &Doubts, rows: I) -> Checked
where
    I: IntoIterator<Item = &'a ChainEvent>,
{
    let (after, out) = rows
        .into_iter()
        .fold((ds.clone(), false), |(acc, out), event| {
            let next = step(bounds, event, &acc);
            let tripped = over(params, &next);
            (next, out || tripped)
        });
    if out {
        Checked::Out(after)
    } else {
        Checked::Within(after)
    }
}

// ---------------------------------------------------------------------------
// The effective depth, the guard and the builder's test (the Lean's sections 2,
// 4 and 5), the out host's two reads and the snapshot (`lean/Headers/Host.lean`
// sections 6 and 12).
// ---------------------------------------------------------------------------

/// The Lean's `Tracker.depth`: the block at `height` counts one, each height
/// above it to `tip` one more; zero above the tip.
fn depth(tip: Height, height: Height) -> u64 {
    (u64::from(tip) + 1).saturating_sub(u64::from(height))
}

/// The Lean's `leadBound`: the lesser of `depth` and the lead `tip - r.tip`
/// (stopped at zero) over every held entry forking at or below `x`.
fn lead_bound<'a>(
    depth: u64,
    tip: Height,
    x: Height,
    held: impl Iterator<Item = &'a Rival>,
) -> u64 {
    held.filter(|rival| rival.fork <= x)
        .map(|rival| u64::from(tip.saturating_sub(rival.tip)))
        .fold(depth, u64::min)
}

/// The Lean's `splitBelow`: an open split at or below `x`, whatever its key.
fn split_below(ds: &Doubts, x: Height) -> bool {
    ds.splits.iter().any(|(_, fork)| *fork <= x)
}

/// The tip the lead is read under: the lesser of the feed's tip at the
/// host's cursor and the host's own verified tip. When the headers are ahead
/// it is the feed's tip (never a fresher header read); when the feed is ahead
/// it is the headers' tip, never a tip the host has not verified. The Lean's
/// `leadTip`.
///
/// # Errors
///
/// [`CheckError::Headers`] when the headers cannot give their tip.
pub fn lead_tip<H: Headers + ?Sized>(headers: &H, ds: &Doubts) -> Result<Height, CheckError> {
    let tip = headers.tip_height().map_err(CheckError::Headers)?;
    Ok(ds.tip.min(tip))
}

/// The effective depth of the height `x`: nothing under an open split at or
/// below it; else the lesser of its depth under the host's own verified
/// headers and the lead under [`lead_tip`] over every held entry (the
/// summary and the kept rivals) that forked at or below it. An act with
/// number `N` passes at `x` exactly when `N` is at most this. The Lean's
/// `effDepth`.
///
/// An open split at or below `x` answers `Ok(0)` before any header is read,
/// so it answers under headers that fault as well.
///
/// # Errors
///
/// [`CheckError::Headers`] when the headers cannot give their tip; never a
/// depth.
pub fn eff_depth<H: Headers + ?Sized>(
    headers: &H,
    ds: &Doubts,
    x: Height,
) -> Result<u64, CheckError> {
    if split_below(ds, x) {
        return Ok(0);
    }
    let tip = headers.tip_height().map_err(CheckError::Headers)?;
    Ok(lead_bound(depth(tip, x), ds.tip.min(tip), x, ds.held()))
}

/// An explicit application rule that says otherwise: its name, and the words
/// it allows a spend under. The Lean's `Tracker.AppRule`.
///
/// `allows` is a pure function of the word, as the Lean's is: a host whose
/// rule reads its own state decides before the call and passes a rule or
/// none. A rule is consulted on the words that claim nothing of the chain
/// alone; it never overrides `Stale`, `Rejected` or `Conflicted`, and a
/// `Mined` word is read on its depth.
///
/// `#[non_exhaustive]`: build one with [`AppRule::new`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AppRule {
    /// The rule's name, for the host's records.
    pub name: String,
    /// The words the rule allows a spend under.
    pub allows: fn(&Word) -> bool,
}

impl AppRule {
    /// The rule named `name` that allows a spend under the words `allows`
    /// answers true for.
    #[must_use]
    pub fn new(name: impl Into<String>, allows: fn(&Word) -> bool) -> Self {
        Self {
            name: name.into(),
            allows,
        }
    }
}

/// The Lean's `Tracker.ruleAllows`: a rule's word, or no when there is none.
fn rule_allows(rule: Option<&AppRule>, word: &Word) -> bool {
    rule.is_some_and(|rule| (rule.allows)(word))
}

/// The Lean's `Tracker.stillConfirmed`: the header at a `mined` word's
/// height still carries its checked root. No header there is `false`.
fn still_confirmed<H: Headers + ?Sized>(headers: &H, state: &State) -> Result<bool, CheckError> {
    let Word::Mined(mined) = state.word() else {
        return Ok(false);
    };
    let header = headers
        .header_at(mined.height())
        .map_err(CheckError::Headers)?;
    Ok(header.is_some_and(|header| header.merkle_root == mined.checked().root()))
}

/// The Lean's `Tracker.rootChecks`: the header at a proof's height carries
/// the root the proof computes. No header there is `false`.
fn root_checks<H: Headers + ?Sized>(headers: &H, proof: &Proof) -> Result<bool, CheckError> {
    let root = proof.root()?;
    let header = headers
        .header_at(proof.height())
        .map_err(CheckError::Headers)?;
    Ok(header.is_some_and(|header| header.merkle_root == root))
}

/// The guard's shape, shared by [`spendable_eff`] and [`spendable_c`]: the
/// depth of a `mined` word is read before its header, so a word too shallow
/// is refused with no header read.
fn guard<H, F>(
    params: &Params,
    headers: &H,
    eff: F,
    rule: Option<&AppRule>,
    state: &State,
) -> Result<bool, CheckError>
where
    H: Headers + ?Sized,
    F: FnOnce(Height) -> Result<u64, CheckError>,
{
    match state.word() {
        Word::Mined(mined) => {
            if eff(mined.height())? < params.spend_depth {
                return Ok(false);
            }
            still_confirmed(headers, state)
        }
        word @ (Word::Unknown | Word::Built | Word::Announced | Word::Seen | Word::Abandoned) => {
            Ok(rule_allows(rule, word))
        }
        Word::Stale | Word::Rejected { .. } | Word::Conflicted { .. } => Ok(false),
    }
}

/// The spend guard read on the effective depth. A `Mined` word is spendable
/// when its height's [`eff_depth`] is at least [`Params::spend_depth`] and
/// the headers of the moment still carry its checked root; a word that
/// claims nothing of the chain (`Unknown`, `Built`, `Announced`, `Seen`,
/// `Abandoned`) is spendable only under a rule that allows it, and reads no
/// header; `Stale`, `Rejected` and `Conflicted` are never spendable. The
/// Lean's `spendableEff`.
///
/// The depth is the host's own, read from `params`, set once when it built
/// them: one host, one depth for an act. It is never weaker than the guard
/// on the depth under the headers alone (the Lean's
/// `spendableEff_imp_spendable`), and with no rival, no split and nothing
/// dropped it is that guard (`spendableEff_today`). The suspect mark is not
/// read; a host that keeps one reads it beside this.
///
/// # Errors
///
/// [`CheckError::Headers`] when a header the answer needs cannot be read;
/// never a pass.
pub fn spendable_eff<H: Headers + ?Sized>(
    params: &Params,
    headers: &H,
    ds: &Doubts,
    rule: Option<&AppRule>,
    state: &State,
) -> Result<bool, CheckError> {
    guard(params, headers, |x| eff_depth(headers, ds, x), rule, state)
}

/// The builder's test: a stored path may enter a BEEF when its root is the
/// active root at its height in the host's headers and that height's
/// [`eff_depth`] is at least 2. A predicate, not a builder. The Lean's
/// `mayEnter`.
///
/// The depth is read before the header, so no header is asked for at a
/// height the answer does not need; the answer is the Lean's.
///
/// # Errors
///
/// [`CheckError::Headers`] when a header the answer needs cannot be read,
/// and [`CheckError::InvalidProof`] when the path does not reduce; never a
/// pass.
pub fn may_enter<H: Headers + ?Sized>(
    headers: &H,
    ds: &Doubts,
    proof: &Proof,
) -> Result<bool, CheckError> {
    if eff_depth(headers, ds, proof.height())? < 2 {
        return Ok(false);
    }
    root_checks(headers, proof)
}

/// The effective depth of a checked host: nothing at any height while the
/// feed is out of its contract ([`Checked::Out`]), with no header read; else
/// [`eff_depth`] of its doubts. The Lean's `effC`.
///
/// # Errors
///
/// As [`eff_depth`].
pub fn eff_c<H: Headers + ?Sized>(
    headers: &H,
    checked: &Checked,
    x: Height,
) -> Result<u64, CheckError> {
    match checked {
        Checked::Out(_) => Ok(0),
        Checked::Within(ds) => eff_depth(headers, ds, x),
    }
}

/// The guard of a checked host: [`spendable_eff`] read on [`eff_c`]. An out
/// host passes no `Mined` word under a spend depth of 1 or more (the Lean's
/// `out_passes_nothing`); under a spend depth of 0 it passes one whose root
/// the headers carry (the row `outZero`). A word with no height reads no feed
/// and is the guard of today, out or not. The Lean's `spendableC`.
///
/// # Errors
///
/// As [`spendable_eff`].
pub fn spendable_c<H: Headers + ?Sized>(
    params: &Params,
    headers: &H,
    checked: &Checked,
    rule: Option<&AppRule>,
    state: &State,
) -> Result<bool, CheckError> {
    guard(params, headers, |x| eff_c(headers, checked, x), rule, state)
}

/// What a page response would give a host that starts: one pair for every
/// rival (the lowest fork height and the highest tip), the open splits (each
/// its key and its fork height) and the feed's tip. The Lean's
/// `Headers.Host.Snapshot`.
///
/// The snapshot's wire is the header service's; this type is the shape a
/// host reads it into. A key is held in lower case, as [`raw`] reads a row's
/// key. A host takes a snapshot through [`accepts`] alone, as a [`Served`].
///
/// `#[non_exhaustive]`: build one with [`Snapshot::new`],
/// [`Snapshot::empty`] or [`Snapshot::least`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Snapshot {
    /// The pair covering every entry the service holds, or none.
    pub rival: Option<Rival>,
    /// The open splits, each its key and its fork height, in list order.
    pub splits: Vec<(Hash, Height)>,
    /// The feed's tip at the snapshot's cursor.
    pub tip: Height,
}

impl Snapshot {
    /// The snapshot of these parts, each split's key lower-cased.
    #[must_use]
    pub fn new(rival: Option<Rival>, splits: Vec<(Hash, Height)>, tip: Height) -> Self {
        Self {
            rival,
            splits: splits
                .into_iter()
                .map(|(key, fork)| (key.to_ascii_lowercase(), fork))
                .collect(),
            tip,
        }
    }

    /// The empty snapshot at a tip: today's host, blind to every fork and
    /// split shown before it. The Lean's `Snapshot.empty`.
    #[must_use]
    pub const fn empty(tip: Height) -> Self {
        Self {
            rival: None,
            splits: Vec::new(),
            tip,
        }
    }

    /// The least snapshot that is enough for doubts `ds`: the merge of every
    /// entry held, every open split, the tip. The Lean's `Snapshot.least`.
    #[must_use]
    pub fn least(ds: &Doubts) -> Self {
        let held: Vec<Rival> = ds.held().cloned().collect();
        Self {
            rival: absorb(None, &held),
            splits: ds.splits.clone(),
            tip: ds.tip,
        }
    }

    /// The doubts a snapshot gives: no kept rival, its splits, its tip, and
    /// its pair held as the summary, so the prune never drops it and every
    /// `reorg` row re-keys it. The Lean's `Snapshot.doubts`.
    #[must_use]
    pub fn doubts(&self) -> Doubts {
        Doubts::new(
            Vec::new(),
            self.splits.clone(),
            self.tip,
            self.rival.clone(),
        )
    }

    /// The service's fold: the least snapshot of [`run`] over `rows` from this
    /// snapshot's doubts. The Lean's `Snapshot.fold`.
    #[must_use]
    pub fn fold<'a, I>(&self, bounds: &Bounds, rows: I) -> Self
    where
        I: IntoIterator<Item = &'a ChainEvent>,
    {
        Self::least(&run(bounds, &self.doubts(), rows))
    }
}

/// A snapshot as the header service serves it: the snapshot, the cursor it
/// is at, the row version of the decoder that folded it, the cursor through
/// which every row was readable to that decoder, and the hash of its tip
/// (the snapshot's `tip` is the height). The Lean's `Headers.Host.Served`.
///
/// The wire of a served snapshot is the header service's and this crate
/// reads none: a host reads it and builds this value.
///
/// `#[non_exhaustive]`: build one with [`Served::new`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Served {
    /// The snapshot.
    pub snap: Snapshot,
    /// The feed cursor the snapshot is at.
    pub cursor: u64,
    /// The row version of the decoder that folded it.
    pub version: u64,
    /// The cursor through which every row was readable to that decoder.
    pub readable: u64,
    /// The hash of the block at the snapshot's tip.
    pub tip_hash: Hash,
}

impl Served {
    /// The served snapshot of these parts.
    #[must_use]
    pub fn new(
        snap: Snapshot,
        cursor: u64,
        version: u64,
        readable: u64,
        tip_hash: impl Into<Hash>,
    ) -> Self {
        Self {
            snap,
            cursor,
            version,
            readable,
            tip_hash: tip_hash.into(),
        }
    }
}

/// Why a host refuses a served snapshot: one value for each condition of
/// [`accepts`], and the headers' fault.
///
/// A refused snapshot is never built into doubts. What a host does then is
/// the host's: an out host holds every act ([`spendable_c`] passes no `Mined`
/// word under a spend depth of 1 or more) and asks again at its next page
/// read, no faster than its page poll.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refused {
    /// The snapshot's cursor is below the host's.
    Behind {
        /// The snapshot's cursor.
        cursor: u64,
        /// The host's cursor.
        host: u64,
    },
    /// The decoder that folded the snapshot reads a row version the host's
    /// [`Params::versions`] does not name.
    Version(u64),
    /// Some row through the snapshot's cursor was not readable to the
    /// decoder that folded it: its readable cursor is below its cursor.
    Unreadable {
        /// The cursor through which every row was readable.
        readable: u64,
        /// The snapshot's cursor.
        cursor: u64,
    },
    /// The snapshot itself holds more open splits than the feed's named
    /// number, [`Params::bound`].
    Overfull {
        /// The open splits the snapshot holds.
        open: usize,
        /// The feed's named number.
        bound: u32,
    },
    /// The host's headers carry no header at the snapshot's tip.
    TipAbsent(Height),
    /// The host's header at the snapshot's tip has another hash.
    TipOtherHash(Height),
    /// The host's headers could not answer.
    Headers(HeaderError),
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Behind { cursor, host } => {
                write!(f, "snapshot at cursor {cursor} is behind the host's {host}")
            }
            Self::Version(v) => write!(f, "snapshot folded by a decoder of row version {v}"),
            Self::Unreadable { readable, cursor } => write!(
                f,
                "snapshot at cursor {cursor} was readable only through {readable}"
            ),
            Self::Overfull { open, bound } => write!(
                f,
                "snapshot holds {open} open splits, more than the feed's number {bound}"
            ),
            Self::TipAbsent(h) => write!(f, "no header of the host's at the snapshot's tip {h}"),
            Self::TipOtherHash(h) => {
                write!(
                    f,
                    "the host's header at the snapshot's tip {h} has another hash"
                )
            }
            Self::Headers(e) => write!(f, "headers: {e}"),
        }
    }
}

impl std::error::Error for Refused {}

/// Whether a host at feed cursor `cursor` takes the served snapshot `served`:
/// at its own cursor or past it; folded by a decoder of a row version
/// [`Params::versions`] names; every row through its cursor readable to that
/// decoder; holding at most [`Params::bound`] open splits; and its tip, by
/// height and by hash, the host's own verified header. The conditions are
/// read in that order and the earliest that fails is the answer; the headers
/// are read last, so a snapshot refused on its own parts reads no header.
/// The Lean's `accepts`.
///
/// A snapshot at the host's cursor alone is not asked for: an out host reads
/// no row, so its cursor stays where the check tripped, and an honest
/// snapshot there holds the count that tripped it (the row
/// `stuckAtCursor`); the service's fold covers the kept host at a later
/// cursor. The version covers the rows to the snapshot's cursor alone: every
/// later row's version is the decoder's to check, as before a start (the row
/// `afterStart`). The hash names the tip and says nothing of the rivals and
/// splits, which stay the service's word (the row `hashNotContent`).
///
/// # Errors
///
/// The [`Refused`] value of the earliest condition that fails, or
/// [`Refused::Headers`] when the headers cannot answer.
pub fn accepts<H: Headers + ?Sized>(
    params: &Params,
    headers: &H,
    cursor: u64,
    served: &Served,
) -> Result<(), Refused> {
    if served.cursor < cursor {
        return Err(Refused::Behind {
            cursor: served.cursor,
            host: cursor,
        });
    }
    if !params.versions.contains(&served.version) {
        return Err(Refused::Version(served.version));
    }
    if served.readable < served.cursor {
        return Err(Refused::Unreadable {
            readable: served.readable,
            cursor: served.cursor,
        });
    }
    if let Some(bound) = params.bound {
        let open = served.snap.splits.len();
        if u64::try_from(open).map_or(true, |n| u64::from(bound) < n) {
            return Err(Refused::Overfull { open, bound });
        }
    }
    let tip = served.snap.tip;
    match headers.header_at(tip).map_err(Refused::Headers)? {
        None => Err(Refused::TipAbsent(tip)),
        Some(header) if header.hash.eq_ignore_ascii_case(&served.tip_hash) => Ok(()),
        Some(_) => Err(Refused::TipOtherHash(tip)),
    }
}

/// A start: the doubts a host at feed cursor `cursor` takes from the served
/// snapshot, and the cursor it then stands at, when [`accepts`] takes it.
/// The host keeps its headers and its tracked words; its doubts become the
/// snapshot's and its cursor the snapshot's. The Lean's `startFrom` behind
/// `accepts`.
///
/// # Errors
///
/// As [`accepts`]: a refused snapshot builds no doubts.
pub fn start_from<H: Headers + ?Sized>(
    params: &Params,
    headers: &H,
    cursor: u64,
    served: &Served,
) -> Result<(Doubts, u64), Refused> {
    accepts(params, headers, cursor, served)?;
    Ok((served.snap.doubts(), served.cursor))
}
