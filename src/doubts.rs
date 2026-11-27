//! A host's doubts: the rivals the header service's feed has shown it, the
//! open splits and the feed's tip, folded one row at a time.
//!
//! The module mirrors `lean/Headers/Doubts.lean` of bsv-stack-lean at
//! `7213ef4a`, under the Lean's names: [`Rival`] (`Rival`, with `covers` and
//! `merge`), [`Doubts`] (`Doubts`, with `cold` and `held`), [`Bounds`]
//! (`Bounds`, and `B400`), [`raw`], [`prune`], [`step`] and [`run`]. The
//! private helpers `add`, `rekey`, `rekey_all`, `absorb`, `top_height`,
//! `low_tip` and `keep` are the Lean's `add`, `rekey`, `rekeyAll`, `absorb`,
//! `topHeight`, `lowTip` and `keep`.
//!
//! The doubts are a pure function of the feed's rows. No function here reads
//! a header, a clock or the network, and none reads a hash, a root, a time or
//! a chain work of an event: a `tip` row gives its height, a `fork` row its
//! height and its competitor's height, a `reorg` row its fork height, the
//! highest removed height and its new tip's height, a `split` row its fork
//! height and its lower witness tip's height, a `splitResolved` row its fork
//! height.
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
//! All four fields are required. `gone` is `null` or a rival; a text without
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
//! feed's tip, the open splits' fork heights in list order, each kept rival
//! as `fork:tip` in list order, and the summary as `fork:tip` or `-`. The
//! stored field `gone` is printed as `summary=`.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{ChainEvent, EventHeader, Height, SplitTip};

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

/// A host's doubts: the rivals it has been shown, the fork heights of the
/// open splits, the feed's tip at the host's cursor, and the summary of what
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
    splits: Vec<Height>,
    tip: Height,
    // Required on read: with no `default`, a text that leaves the field out
    // is refused where serde's derive would read an absent option as `None`.
    #[serde(deserialize_with = "required_gone")]
    gone: Option<Rival>,
}

fn required_gone<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Rival>, D::Error> {
    Option::<Rival>::deserialize(deserializer)
}

impl Doubts {
    /// The doubts holding exactly these parts: the kept rivals and the open
    /// splits' fork heights in list order, the feed's tip, and the summary of
    /// what was dropped. No invariant is asked of them: any value is a legal
    /// start for [`run`].
    #[must_use]
    pub const fn new(
        rivals: Vec<Rival>,
        splits: Vec<Height>,
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

    /// The fork heights of the open splits, in list order: the newest is at
    /// the front.
    #[must_use]
    pub fn splits(&self) -> &[Height] {
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
        for (i, fork) in self.splits.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{fork}")?;
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
/// - `split`: the row's fork height is put at the front of the open splits,
///   and the lower of the two witness tips is added as a rival at it.
/// - `splitResolved`: every open split at the row's fork height closes. The
///   row's key is not read.
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
            fork_height, tips, ..
        } => {
            let mut splits = Vec::with_capacity(ds.splits.len() + 1);
            splits.push(*fork_height);
            splits.extend_from_slice(&ds.splits);
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
        ChainEvent::SplitResolved { fork_height, .. } => Doubts {
            splits: ds
                .splits
                .iter()
                .copied()
                .filter(|fork| fork != fork_height)
                .collect(),
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
