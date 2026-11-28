# The doubts: what a host's feed has shown it and not yet settled

A host reads the header service's feed one row at a time. Some rows say the chain is not yet settled: a fork names a competing branch, a reorg names the branch it removed, and a split says two of the service's sources follow different branches past a common ancestor. `bsv_tracker::doubts` folds those rows into a host's doubts (the rival branches it has been shown, the open splits, the feed's tip, and a summary of what the prune has dropped) and answers the depth a host may trust at a height while they stand: nothing under an open split at or below it, and otherwise no more than the lead its own chain holds over every rival that forked at or below it. A coin mined at that height is spendable only on that depth. The fold reads no header, clock or network; the reads take the host's own verified headers. The module sits beside the tracker's reducer and never inside it: a host that reads one feed row calls `doubts::step` and then `Tracker::on_chain` (or `State::step` with `Evidence::Chain`), and nothing a host already calls changes its answer.

Every Lean name on this page is read at bsv-stack-lean `08ce467d` (`08ce467d2251c14f8a32719deb5f6e904d869bc4`): `lean/Headers/Doubts.lean`, `lean/Headers/Host.lean` and `lean/Tracker.lean`, by file, section and name.

## The table

One row per public item of `bsv_tracker::doubts`, and the fields of `Params` that it reads.

| Rust item | Lean name | at | what it answers |
|---|---|---|---|
| `Rival` (`fork`, `tip`), `Rival::new` | `Doubts.lean` section 1, `Rival` | `08ce467d` | a rival branch: the height it forked at and the height of its highest header the host has seen |
| `Rival::covers` | `Rival.covers` | `08ce467d` | whether one rival forked at or below another and reaches at least its tip, so binds every act the other binds |
| `Rival::merge` | `Rival.merge` | `08ce467d` | the pair covering both: the lower fork height, the higher tip |
| `Doubts`, `Doubts::new` | section 1, `Doubts` (`rivals`, `splits`, `tip`, `gone`) | `08ce467d` | the state: the kept rivals, the open splits (each its key and its fork height), the feed's tip, the summary of what was dropped; `new` takes any parts, each key lower-cased |
| `Doubts::cold` | `Doubts.cold` | `08ce467d` | a host with no rows read: nothing held, the tip it starts from |
| `Doubts::rivals`, `splits`, `tip`, `gone` | the fields of `Doubts` | `08ce467d` | the parts, the lists in the Lean's order (the newest entry at the front) |
| `Doubts::held` | `Doubts.held` | `08ce467d` | every entry a depth reads: the summary, then the kept rivals |
| `Display for Doubts` | the line of `tools/journal/JournalRun.lean` | `08ce467d` | the printed line (below) |
| `Serialize`, `Deserialize` for `Doubts` | none: the crate's own | | the stored form (below) |
| `Bounds` (`settled`), `Bounds::new` | section 1, `Bounds` | `08ce467d` | the settled line's depth, below which the feed names no reorg |
| `Bounds::B400` | section 8, `B400` | `08ce467d` | the feed's settled line, 400 |
| `raw` | section 1, `raw` | `08ce467d` | one row's change, no prune: `tip` moves the tip; `fork` adds its competitor; `reorg` moves the tip, re-keys the summary and every rival forking above it, and adds the removed branch; `split` opens or lowers its key and adds its lower witness tip; `splitResolved` removes its key alone; the rest change nothing |
| `prune` | `prune` (with `keep`) | `08ce467d` | the rivals the settled line keeps stay in order; every other is merged into the summary |
| `step` | `step` | `08ce467d` | `raw`, then `prune`: the doubts after one row |
| `run` | `run` | `08ce467d` | `step` folded from the left over rows |
| `over` | `Host.lean` section 12, `over` | `08ce467d` | whether the open splits number more than `Params::bound`; never with the number absent |
| `Checked` (`Within`, `Out`), `doubts`, `into_doubts`, `is_out` | section 12, `HostC.out` beside the doubts | `08ce467d` | the doubts after the rows read, and whether the feed stayed within its named number after each; either way the doubts are the unchecked fold's |
| `step_checked` | section 12, `trips` of one row beside `step` | `08ce467d` | `step`, then the count against the number |
| `run_checked` | section 12, `trips` beside `run` | `08ce467d` | `run`, `Out` when any row left more open than the number, even if a later row brings the count back; the start is not counted |
| `lead_tip` | `Doubts.lean` section 2, `leadTip` | `08ce467d` | the tip a lead is read under: the lesser of the feed's tip and the host's verified tip |
| `eff_depth` | section 2, `effDepth` | `08ce467d` | the depth a host may trust at a height (the opening paragraph); an open split at or below it answers 0 before any header is read |
| `AppRule` (`name`, `allows`), `AppRule::new` | `Tracker.lean`, `AppRule` | `08ce467d` | an explicit application rule: the words it allows a spend under |
| `spendable_eff` | `Doubts.lean` section 4, `spendableEff` | `08ce467d` | the spend guard on the effective depth: a `Mined` word whose height's depth is at least `Params::spend_depth` and whose root the headers still carry; a word that claims nothing of the chain under a rule that allows it; `Stale`, `Rejected`, `Conflicted` never |
| `may_enter` | section 5, `mayEnter` | `08ce467d` | the builder's test: a stored path's root is active at its height and that height's effective depth is at least 2 |
| `eff_c` | `Host.lean` section 12, `effC` | `08ce467d` | the depth of a checked host: 0 at every height while `Out`, no header read; else `eff_depth` |
| `spendable_c` | section 12, `spendableC` | `08ce467d` | the guard of a checked host, on `eff_c` |
| `Snapshot` (`rival`, `splits`, `tip`), `Snapshot::new` | `Host.lean` section 6, `Snapshot` | `08ce467d` | what a page response would give a host that starts: one pair for every rival, the open splits, the feed's tip |
| `Snapshot::empty` | section 6, `Snapshot.empty` | `08ce467d` | the snapshot of today's host: blind to every fork and split shown before it |
| `Snapshot::least` | section 6, `Snapshot.least` | `08ce467d` | the least snapshot enough for given doubts |
| `Snapshot::doubts` | section 6, `Snapshot.doubts` | `08ce467d` | the doubts a snapshot gives: its pair held as the summary, so the prune never drops it |
| `Snapshot::fold` | section 12, `Snapshot.fold` | `08ce467d` | the service's fold: the least snapshot of `run` from a snapshot's doubts |
| `Served` (`snap`, `cursor`, `version`, `readable`, `tip_hash`), `Served::new` | section 12, `Served` (`tipHash`) | `08ce467d` | a snapshot as the service serves it: its cursor, the row version of the decoder that folded it, the cursor through which every row was readable to that decoder, its tip's hash |
| `Refused` | the conjuncts of section 12's `accepts` | `08ce467d` | why a snapshot is refused, one value a condition (below); the Lean answers a `Bool` |
| `accepts` | section 12, `accepts` (with `tipVerified`) | `08ce467d` | whether a host at a cursor takes a served snapshot |
| `start_from` | section 12, `startFrom` behind `accepts` | `08ce467d` | the doubts and the cursor of a taken snapshot |
| `Params::spend_depth` | `Tracker.lean`, `Params.spendDepth` | `08ce467d` | the depth a `Mined` word needs before a coin of it is spendable with no rule; read by `spendable_eff` and `spendable_c` |
| `Params::bound` | `Host.lean` section 12, `StartParams.bound` | `08ce467d` | the feed's named number: the most split keys it can have open at once; read by `over`, `step_checked`, `run_checked` and `accepts` |
| `Params::versions` | section 12, `StartParams.versions` | `08ce467d` | the row versions the host's decoder reads; read by `accepts` |
| `Params::VERSIONS` | none: the crate's decoder | | `[1, 2]`, the versions `decode_envelope` reads, `versions`' default |

`Params::age_threshold` is the tracker's and the module does not read it. The Lean's private helpers `add`, `rekey`, `rekeyAll`, `absorb`, `topHeight`, `lowTip`, `isOpen`, `openSplit`, `closeSplit`, `keep`, `leadBound`, `splitBelow` and `Tracker.lean`'s `depth`, `stillConfirmed`, `rootChecks` and `ruleAllows` are private functions of `src/doubts.rs` under the same names in snake case.

Where the two differ: the Lean's headers always answer and the crate's can fail, so every function that reads a header answers `Err(CheckError::Headers(_))` on a fault, never a pass (and `Refused::Headers` in `accepts`); a depth is read before a header, so a hold answers before any header is asked for. The Lean compares a snapshot's tip hash exactly, the crate in either case. The Lean's guards name a word `handed` that the crate's `Word` does not have.

## The stored form

`Doubts` has one stored form, its serde text, the fields in the Lean's order and under the Lean's names:

```json
{"rivals":[{"fork":101,"tip":101}],"splits":[["k95",95]],"tip":103,"gone":null}
```

An open split is a pair, `["<key>", <fork height>]`, in list order; `gone` is `null` or a rival. The rule is that a text the fold could not have meant is refused, never read loosely:

- all four fields are required, `gone` included: a text without it is refused and never read as "nothing dropped", since forgetting the summary is the permissive direction;
- a field this reader does not know is refused, in `Doubts` and in a rival; a field's name never changes, and a later minor only adds one;
- a split that is not a pair of a text key and a height is refused (a fork height alone, a third part, a key that is not text);
- one key named twice is refused, in any case: each key is read in lower case, as `raw` and `Doubts::new` read it, so two spellings of one key are one key named twice.

The reader checks shape and no invariant: any value the type holds is a legal start for `run`. A host that cannot read its stored doubts reports a fault; it does not start cold in silence. `Bounds`, `Checked`, `Snapshot`, `Served` and `Refused` have no stored form.

## The printed line

`Display` prints

```text
tip=103 splits=[k95:95] rivals=[101:101] summary=-
```

field by field: `tip=` the feed's tip at the host's cursor; `splits=[..]` each open split as `key:height`, the key in full and in lower case with no `split:` prefix, in list order (the newest at the front), separated by a space; `rivals=[..]` each kept rival as `fork:tip` in list order; `summary=` the stored field `gone`, as `fork:tip` or `-`. The stack's journal driver prints the same line after the row's cursor and a space, which is what the test of record compares.

## The bound and the out host

A feed may name the most split keys it can have open at once. The host holds that number as `Params::bound`, from the service's contract and never from a snapshot or a row: a check against a number the checked party sends checks nothing. With `bound: None` there is no check and a host is today's, row for row.

A host with a number folds its rows through `step_checked` or `run_checked`, which count the open keys after every row, not at a page's end. When some row leaves more open than the number the answer is `Checked::Out`: the feed is out of its contract. No split is dropped and no key merged: the doubts in the answer are the unchecked fold's. An out host then:

1. reads no row; its cursor stays where the check tripped;
2. reads its depths through `eff_c` (0 at every height) and its guard through `spendable_c`, which passes no `Mined` word under a spend depth of 1 or more (under a spend depth of 0, one whose root the headers carry); a word with no height is the guard of today;
3. at each page read makes one ask for a snapshot and nothing more: one start a page read at most, never a start that causes a start.

It takes a snapshot through `accepts` or `start_from` alone. The conditions are read in this order and the earliest that fails is the answer; the four on the snapshot's own parts read no header:

| condition | refusal |
|---|---|
| the snapshot's cursor is at the host's cursor or past it | `Refused::Behind { cursor, host }` |
| the row version of the decoder that folded it is one `Params::versions` names | `Refused::Version(v)` |
| every row through its cursor was readable to that decoder (`readable` at least `cursor`) | `Refused::Unreadable { readable, cursor }` |
| it holds at most `Params::bound` open splits | `Refused::Overfull { open, bound }` |
| its tip is the host's own verified header, by height and by hash | `Refused::TipAbsent(height)`, `Refused::TipOtherHash(height)`; `Refused::Headers(e)` when the headers cannot answer |

A taken snapshot gives the host its doubts (`Snapshot::doubts`) and its cursor, and the host is within again; it keeps its headers and its tracked words. A refused one builds no doubts: the host stays out, holds every act, and asks again at its next page read, no faster than its page poll. A host does not ask at its own cursor alone: it read no row since the trip, and an honest snapshot there holds the count that tripped it. The snapshot's version covers the rows to its cursor alone; every later row's version is the decoder's to check, as before a start. The tip's hash names the tip and says nothing of the rivals and splits, which stay the service's word.

A host with no snapshot to start from starts at `Doubts::cold(tip)`, which is `Snapshot::empty(tip).doubts()`: today's host, blind to a fork or split shown before it started, and no worse than today.

## What is not mirrored here, and where it belongs

- The hosts over a journal (`Host.lean`'s `Host`, `Policy`, `readStep`, `readMany`, `readC`, `readsC`, `HostC`'s counters, `pageTrips`, `Reach`, `ReachC`, `coldFrom`, `Covers`, the recheck after a gap, the load of a page read, the policy at an `unreadable` delivery): the host's. The crate holds no journal, cursor or page; a host's page loop is its own, built on `run_checked`, `accepts` and `start_from`.
- The snapshot's wire: the header service's. The crate reads none; a host reads the service's answer and builds a `Served` with `Served::new`.
- The proofs' own vocabulary (`Clear`, `Antichain`, `Quiet`, `Kept`, `Cov`, `img`, `fresh`, `rawAll`, `runAll`, `runUnpruned`, `dropped`, `peak`, `rowFits`, `WellFormed`, `Inv`, `SplitCov`, `Holds`, `CoversK`) and the counterexamples the Lean keeps: statements about the fold, mirrored in the tests where a property needs them.
- `ancestorWord` and `registerAncestor` (`Doubts.lean` section 6): no new item. A received coin's proven ancestor is `State::new(txid)` stepped with `Evidence::Proof(path)`, the tracker's own proof step.
- The BEEF builder itself: the host's. The crate gives the builder's test, `may_enter`, as a predicate.
- The suspect mark: the tracker's. `spendable_eff` does not read it; a host that keeps one reads it beside.

## The test of record

`tests/journal_states.rs`, `journals_reproduce_the_states_after_every_row`, proves that the crate's fold is the stack's: the header service's two journals, each folded from `Doubts::cold(0)` under `Bounds::new(400)` with the crate's own `decode_envelope` and `doubts::step`, reproduce the stack's journal driver's state after every row, compared two ways (the cursor, the tip, the splits in order, the rivals as a set and the summary; and the printed line byte for byte). A delivered `unreadable` row is skipped, as the driver skips it.

The journals and the driver's states files are private to the estate, and nothing of them is in this repository: no row, no states line, no cursor, hash or height, no fixture cut from them. The test reads them in place from two folders:

- `BSV_TRACKER_JOURNALS_DIR`: `production.events.jsonl` and `beta.events.jsonl`;
- `BSV_TRACKER_JOURNAL_STATES_DIR`: `production.states.txt` and `beta.states.txt`.

With `BSV_TRACKER_JOURNALS_DIR` unset (CI's run, and every host without the journals) the test prints this line and passes:

```text
SKIPPED journals_reproduce_the_states_after_every_row: BSV_TRACKER_JOURNALS_DIR is unset (the header service's journals are private to the estate; nothing of them is committed)
```

With the journals named and the states folder unset it fails, so a half-named run never reads as green. A failure names the journal, the line's number and the names of the fields that differ, never a value. `a_wrong_line_is_named_by_its_fields` pins that naming on hand-built values and runs everywhere. The run of record and its tallies are in `docs/VERIFICATION.md`.

The rows the journals do not hold (the split arms, a reorg of several headers or none, a start that is not cold, every depth and guard ask) are pinned in public by `tests/doubts_rows.rs` over `tests/vectors/doubts/rows_of_record.json` (rows taken from the Lean's own rows of record) and by `tests/doubts_guard.rs`.

## The properties

`tests/doubts_properties.rs` states the Lean's theorems as randomized properties, each test named for the theorem it mirrors. `docs/VERIFICATION.md`, "The doubts' theorems as properties", lists each test beside its theorem and what it checks.
