//! The test of record for `bsv_tracker::doubts` (for #51): the doubts after
//! every row of the header service's two journals are the states the stack's
//! journal driver printed for them.
//!
//! The journals and the states files are private to the estate and nothing
//! of them is committed: no row, no states line, no cursor, hash or height.
//! The test reads them in place from two folders named by the environment:
//!
//! - `BSV_TRACKER_JOURNALS_DIR`: `production.events.jsonl` and
//!   `beta.events.jsonl`, one `{"cursor": "<digits>", "event": {..}}` object a
//!   line;
//! - `BSV_TRACKER_JOURNAL_STATES_DIR`: `production.states.txt` and
//!   `beta.states.txt`, one line a folded row, in the form of
//!   `tools/journal/JournalRun.lean` of bsv-stack-lean at `7213ef4a`:
//!   `<cursor> tip=<t> splits=[..] rivals=[..] summary=<fork:tip or ->`.
//!
//! With `BSV_TRACKER_JOURNALS_DIR` unset (CI's run) the test prints a
//! `SKIPPED` line with its reason and passes. With it set, every other
//! fault fails: the states folder unset, a file missing, a row of another
//! shape, a refusal of the crate's decoder, a kind the driver writes no line
//! for, a states line too many or too few, a total other than the stack's.
//!
//! Each journal is folded from `Doubts::cold(0)` under `Bounds::new(400)`
//! (the feed's settled line), a row at a time with `doubts::step`, as the
//! driver folds it; a delivered `unreadable` row is not an envelope, has no
//! line, and is skipped as the driver skips it. After each folded row the
//! state is compared with the next states line two ways: the ruled
//! comparison (the cursor, the tip, the splits in order, the rivals as a
//! set, the summary) and the line byte for byte (the order of the rivals).
//!
//! A failure names the journal, the line's number and the NAMES of the
//! fields that differ (`cursor`, `tip`, `splits`, `rivals`, `summary`,
//! `line`), or the decoder's variant; it never prints a value of a journal.
//! No `assert_eq!` here touches journal data, since it would print both
//! sides. On a pass the test prints one tally line a journal, counts only.
//!
//! The run of record, on the host that mounts both folders:
//!
//! ```text
//! BSV_TRACKER_JOURNALS_DIR=<journals> BSV_TRACKER_JOURNAL_STATES_DIR=<states> \
//!   cargo test --locked --test journal_states -- --nocapture
//! ```
// The lint bar binds the library; a test may unwrap, and its faults fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use bsv_tracker::doubts::{self, Bounds, Doubts, Rival};
use bsv_tracker::{decode_envelope, ChainEvent, DecodeError};
use serde_json::Value;

const JOURNALS: &str = "BSV_TRACKER_JOURNALS_DIR";
const STATES: &str = "BSV_TRACKER_JOURNAL_STATES_DIR";

/// The most failing lines named before the test fails.
const NAMED: usize = 5;

/// One journal: its name and the totals the stack's public page states
/// (`docs/DECISIONS-2026-10-19.md` 28.22) or that follow from them.
struct Journal {
    name: &'static str,
    rows: usize,
    unreadable: usize,
}

const JOURNAL_TOTALS: [Journal; 2] = [
    Journal {
        name: "production",
        rows: 6254,
        unreadable: 0,
    },
    Journal {
        name: "beta",
        rows: 39686,
        unreadable: 11,
    },
];

/// A direct write to the process's stderr, which the test harness does not
/// capture, so the line prints under a plain `cargo test`.
fn say(line: &str) {
    let mut err = std::io::stderr().lock();
    let _ = writeln!(err, "{line}");
}

/// A states line read into its parts, by the driver's form.
struct StatesLine {
    cursor: u64,
    tip: u32,
    splits: Vec<(String, u32)>,
    rivals: Vec<(u32, u32)>,
    summary: Option<(u32, u32)>,
}

fn number<T: std::str::FromStr>(text: &str) -> Option<T> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn pair(text: &str) -> Option<(u32, u32)> {
    let (fork, tip) = text.split_once(':')?;
    Some((number(fork)?, number(tip)?))
}

/// An open split as printed, `key:height` (the keyed form, cut 1f).
fn split_pair(text: &str) -> Option<(String, u32)> {
    let (key, fork) = text.rsplit_once(':')?;
    if key.is_empty() {
        return None;
    }
    Some((key.to_owned(), number(fork)?))
}

fn list<T>(text: &str, item: impl Fn(&str) -> Option<T>) -> Option<Vec<T>> {
    if text.is_empty() {
        return Some(Vec::new());
    }
    text.split(' ').map(item).collect()
}

/// Reads a states line by four `split_once` calls, then by spaces and `:`;
/// `None` for a line of another shape.
fn parse_line(line: &str) -> Option<StatesLine> {
    let (cursor, rest) = line.split_once(" tip=")?;
    let (tip, rest) = rest.split_once(" splits=[")?;
    let (splits, rest) = rest.split_once("] rivals=[")?;
    let (rivals, summary) = rest.split_once("] summary=")?;
    Some(StatesLine {
        cursor: number(cursor)?,
        tip: number(tip)?,
        splits: list(splits, split_pair)?,
        rivals: list(rivals, pair)?,
        summary: if summary == "-" {
            None
        } else {
            Some(pair(summary)?)
        },
    })
}

/// A list's pairs as a set, `None` when a pair is held twice.
fn as_set(pairs: impl Iterator<Item = (u32, u32)>) -> Option<HashSet<(u32, u32)>> {
    let mut set = HashSet::new();
    for p in pairs {
        if !set.insert(p) {
            return None;
        }
    }
    Some(set)
}

fn rival_pair(r: &Rival) -> (u32, u32) {
    (r.fork, r.tip)
}

/// The names of the fields in which `line` differs from the row's `cursor`
/// and the doubts `ds` after it, `line` among them when the bytes differ;
/// empty when the line is equal both ways. A line of another shape is
/// `line` alone.
fn differing(line: &str, cursor: u64, ds: &Doubts) -> Vec<&'static str> {
    let mut names = Vec::new();
    if let Some(parsed) = parse_line(line) {
        if parsed.cursor != cursor {
            names.push("cursor");
        }
        if parsed.tip != ds.tip() {
            names.push("tip");
        }
        if parsed.splits.as_slice() != ds.splits() {
            names.push("splits");
        }
        let theirs = as_set(parsed.rivals.iter().copied());
        let ours = as_set(ds.rivals().iter().map(rival_pair));
        if theirs.is_none() || ours.is_none() || theirs != ours {
            names.push("rivals");
        }
        if parsed.summary != ds.gone().map(rival_pair) {
            names.push("summary");
        }
    }
    if line != format!("{cursor} {ds}") {
        names.push("line");
    }
    names
}

fn kind_name(event: &ChainEvent) -> &'static str {
    match event {
        ChainEvent::Tip { .. } => "Tip",
        ChainEvent::Fork { .. } => "Fork",
        ChainEvent::Reorg { .. } => "Reorg",
        ChainEvent::TipAge { .. } => "TipAge",
        ChainEvent::Invalidated { .. } => "Invalidated",
        ChainEvent::Frozen { .. } => "Frozen",
        ChainEvent::Split { .. } => "Split",
        ChainEvent::SplitResolved { .. } => "SplitResolved",
    }
}

fn decode_name(error: &DecodeError) -> &'static str {
    match error {
        DecodeError::UnknownShape { .. } => "UnknownShape",
        DecodeError::UnknownVersion(_) => "UnknownVersion",
        DecodeError::UnknownKind(_) => "UnknownKind",
    }
}

fn open(dir: &Path, file: &str, variable: &str) -> BufReader<File> {
    match File::open(dir.join(file)) {
        Ok(f) => BufReader::new(f),
        Err(e) => panic!("{file} under {variable}: {:?}", e.kind()),
    }
}

/// One journal's run: its counts on a pass, the failure's text otherwise.
struct Tally {
    rows: usize,
    unreadable: usize,
    folded: usize,
    compared: usize,
    equal: usize,
    byte_equal: usize,
    failures: Vec<String>,
}

fn replay(journal: &Journal, journals: &Path, states: &Path) -> Tally {
    let name = journal.name;
    let events = open(journals, &format!("{name}.events.jsonl"), JOURNALS);
    let mut states_lines = open(states, &format!("{name}.states.txt"), STATES).lines();
    let bounds = Bounds::new(400);
    let mut ds = Doubts::cold(0);
    let mut tally = Tally {
        rows: 0,
        unreadable: 0,
        folded: 0,
        compared: 0,
        equal: 0,
        byte_equal: 0,
        failures: Vec::new(),
    };
    for (index, text) in events.lines().enumerate() {
        let at = index + 1;
        let Ok(text) = text else {
            panic!("{name} journal line {at}: not read as UTF-8 text");
        };
        tally.rows += 1;
        let Ok(Value::Object(row)) = serde_json::from_str::<Value>(&text) else {
            panic!("{name} journal line {at}: not a JSON object");
        };
        if row.len() != 2 {
            panic!("{name} journal line {at}: not exactly the fields cursor and event");
        }
        let Some(cursor) = row
            .get("cursor")
            .and_then(Value::as_str)
            .and_then(number_u64)
        else {
            panic!("{name} journal line {at}: cursor is not a string of digits");
        };
        let Some(event) = row.get("event").filter(|e| e.is_object()) else {
            panic!("{name} journal line {at}: event is not an object");
        };
        if event.get("kind").and_then(Value::as_str) == Some("unreadable") {
            tally.unreadable += 1;
            continue;
        }
        let bytes = serde_json::to_vec(event).unwrap();
        let event = match decode_envelope(&bytes) {
            Ok(event) => event,
            Err(e) => panic!(
                "{name} journal line {at}: the decoder refuses it ({})",
                decode_name(&e)
            ),
        };
        if !matches!(
            event,
            ChainEvent::Tip { .. }
                | ChainEvent::Fork { .. }
                | ChainEvent::Reorg { .. }
                | ChainEvent::TipAge { .. }
        ) {
            panic!(
                "{name} journal line {at}: a {} row, for which the driver writes no line",
                kind_name(&event)
            );
        }
        ds = doubts::step(&bounds, &event, &ds);
        tally.folded += 1;
        let Some(line) = states_lines.next() else {
            panic!("{name} journal line {at}: no states line left");
        };
        let Ok(line) = line else {
            panic!(
                "{name} states line {}: not read as UTF-8 text",
                tally.compared + 1
            );
        };
        tally.compared += 1;
        let names = differing(&line, cursor, &ds);
        let ruled = names.iter().all(|n| *n == "line");
        if ruled {
            tally.equal += 1;
        }
        if !names.contains(&"line") {
            tally.byte_equal += 1;
        }
        if !names.is_empty() {
            tally.failures.push(format!(
                "{name} journal line {at} (states line {}): differs in {}",
                tally.compared,
                names.join(", ")
            ));
            if tally.failures.len() >= NAMED {
                return tally;
            }
        }
    }
    let left = states_lines.count();
    if left > 0 {
        tally
            .failures
            .push(format!("{name}: {left} states lines left over"));
    }
    if tally.rows != journal.rows || tally.unreadable != journal.unreadable {
        tally.failures.push(format!(
            "{name}: the totals are not the stack's (rows {} against {}, unreadable {} against {})",
            tally.rows, journal.rows, tally.unreadable, journal.unreadable
        ));
    }
    tally
}

fn number_u64(text: &str) -> Option<u64> {
    number(text)
}

#[test]
fn journals_reproduce_the_states_after_every_row() {
    let Some(journals) = env::var_os(JOURNALS).map(PathBuf::from) else {
        say(&format!(
            "SKIPPED journals_reproduce_the_states_after_every_row: {JOURNALS} is unset \
             (the header service's journals are private to the estate; nothing of them is committed)"
        ));
        return;
    };
    let Some(states) = env::var_os(STATES).map(PathBuf::from) else {
        panic!("{JOURNALS} is set and {STATES} is unset: a half-named run is not a pass");
    };
    let mut failures = Vec::new();
    for journal in &JOURNAL_TOTALS {
        let t = replay(journal, &journals, &states);
        say(&format!(
            "JOURNAL STATES {}: rows {}, unreadable skipped {}, folded {}, states lines compared {}, \
             equal {} (the rivals as a set), byte-equal {}",
            journal.name, t.rows, t.unreadable, t.folded, t.compared, t.equal, t.byte_equal
        ));
        failures.extend(t.failures);
    }
    for f in &failures {
        say(&format!("FAILED {f}"));
    }
    assert!(
        failures.is_empty(),
        "{} failures, named above by field",
        failures.len()
    );
}

/// The witness that a wrong states line fails, with nothing of a journal:
/// a hand-built `Doubts` and eight hand-written lines, each named by the
/// fields it differs in.
#[test]
fn a_wrong_line_is_named_by_its_fields() {
    let ds = Doubts::new(
        vec![Rival::new(10, 12), Rival::new(5, 7)],
        vec![("k9".into(), 9), ("k4".into(), 4)],
        20,
        Some(Rival::new(1, 3)),
    );
    let cases: [(&str, u64, &[&str]); 8] = [
        (
            "77 tip=20 splits=[k9:9 k4:4] rivals=[10:12 5:7] summary=1:3",
            77,
            &[],
        ),
        (
            "78 tip=20 splits=[k9:9 k4:4] rivals=[10:12 5:7] summary=1:3",
            77,
            &["cursor", "line"],
        ),
        (
            "77 tip=21 splits=[k9:9 k4:4] rivals=[10:12 5:7] summary=1:3",
            77,
            &["tip", "line"],
        ),
        (
            "77 tip=20 splits=[k9:9] rivals=[10:12 5:7] summary=1:3",
            77,
            &["splits", "line"],
        ),
        (
            "77 tip=20 splits=[k9:9 k4:4] rivals=[5:7 10:12] summary=1:3",
            77,
            &["line"],
        ),
        (
            "77 tip=20 splits=[k9:9 k4:4] rivals=[10:12] summary=1:3",
            77,
            &["rivals", "line"],
        ),
        (
            "77 tip=20 splits=[k9:9 k4:4] rivals=[10:12 5:7] summary=-",
            77,
            &["summary", "line"],
        ),
        ("77 tip=20 splits=k9:9,k4:4 rivals=10:12,5:7", 77, &["line"]),
    ];
    for (line, cursor, expected) in cases {
        assert_eq!(differing(line, cursor, &ds), expected, "{line}");
    }
    // A pair held twice is not a set, on either side.
    assert_eq!(
        differing(
            "77 tip=20 splits=[k9:9 k4:4] rivals=[10:12 5:7 5:7] summary=1:3",
            77,
            &ds
        ),
        ["rivals", "line"]
    );
    let twice = Doubts::new(vec![Rival::new(5, 7), Rival::new(5, 7)], vec![], 20, None);
    assert_eq!(
        differing("77 tip=20 splits=[] rivals=[5:7] summary=-", 77, &twice),
        ["rivals", "line"]
    );
}
