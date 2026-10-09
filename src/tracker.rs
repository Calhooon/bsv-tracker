use crate::{
    ChainEvent, CheckError, Hash, Headers, Height, Input, Params, Reask, State, TxId, Word,
};
use std::collections::{BTreeMap, BTreeSet};

/// A named result of an event on one affected transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainUpdate {
    /// The transaction selected by an index, never by a chain scan.
    pub txid: TxId,
    /// The transition's result. The updated state's re-ask survives an error.
    pub result: Result<(), CheckError>,
    /// The next host action after the event.
    pub reask: Option<Reask>,
}

/// A host's tracked transactions, with indexes for targeted chain events.
/// Hints, ages and spend attempts name one transaction. A tip uses only the
/// suspect set; reorgs and forks use only the relevant mined height range.
#[derive(Clone, Debug)]
pub struct Tracker {
    params: Params,
    states: BTreeMap<TxId, State>,
    mined: BTreeMap<Height, BTreeSet<TxId>>,
    headers: BTreeMap<Hash, BTreeSet<TxId>>,
    suspect: BTreeSet<TxId>,
}

#[derive(Debug, PartialEq, Eq)]
struct Index {
    height: Option<Height>,
    header: Option<Hash>,
    suspect: bool,
}
impl Index {
    fn of(state: &State) -> Self {
        Self {
            height: state.word().height(),
            header: match state.word() {
                Word::Mined(m) => Some(m.checked().header().hash.clone()),
                _ => None,
            },
            suspect: state.suspect(),
        }
    }
}

impl Tracker {
    /// An empty host registry with explicit age parameters.
    pub fn new(params: Params) -> Self {
        Self {
            params,
            states: BTreeMap::new(),
            mined: BTreeMap::new(),
            headers: BTreeMap::new(),
            suspect: BTreeSet::new(),
        }
    }
    /// Begin tracking one transaction; repeating this does not erase its state.
    pub fn track(&mut self, txid: impl Into<TxId>) -> &State {
        let txid = txid.into().to_ascii_lowercase();
        self.states
            .entry(txid.clone())
            .or_insert_with(|| State::new(txid))
    }
    /// An immutable state, or none when the host does not track this txid.
    pub fn get(&self, txid: &str) -> Option<&State> {
        self.states.get(&txid.to_ascii_lowercase())
    }
    /// Number of transactions this host tracks.
    pub fn len(&self) -> usize {
        self.states.len()
    }
    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
    /// Apply an input to one transaction and maintain the event indexes,
    /// including after a failed proof or header lookup.
    pub fn apply<H: Headers + ?Sized>(
        &mut self,
        txid: &str,
        headers: &H,
        input: Input,
    ) -> Result<(), CheckError> {
        self.apply_transition(txid, |params, state| state.step(params, headers, input))
    }
    fn apply_transition(
        &mut self,
        txid: &str,
        transition: impl FnOnce(&Params, &mut State) -> Result<(), CheckError>,
    ) -> Result<(), CheckError> {
        let txid = txid.to_ascii_lowercase();
        let state = self
            .states
            .entry(txid.clone())
            .or_insert_with(|| State::new(&txid));
        let old = Index::of(state);
        let result = transition(&self.params, state);
        let new = Index::of(state);
        if old != new {
            if let Some(h) = old.height {
                remove_index(&mut self.mined, &h, &txid);
            }
            if let Some(hash) = old.header {
                remove_index(&mut self.headers, &hash, &txid);
            }
            self.suspect.remove(&txid);
            if let Some(h) = new.height {
                self.mined.entry(h).or_default().insert(txid.clone());
            }
            if let Some(hash) = new.header {
                self.headers.entry(hash).or_default().insert(txid.clone());
            }
            if new.suspect {
                self.suspect.insert(txid);
            }
        }
        result
    }
    /// Deliver an evidence event only to the set it names. The output contains
    /// every selected txid and each fault for the host to report.
    pub fn on_chain<H: Headers + ?Sized>(
        &mut self,
        headers: &H,
        event: ChainEvent,
    ) -> Vec<ChainUpdate> {
        let targets: Vec<TxId> = match &event {
            ChainEvent::Fork { height, .. } => self.at_or_above(*height),
            ChainEvent::Reorg { fork_height, .. } => self.at_or_above(*fork_height),
            ChainEvent::Invalidated { block_hash } => self
                .headers
                .get(block_hash)
                .map(|ids| ids.iter().cloned().collect())
                .unwrap_or_default(),
            ChainEvent::Tip { .. } => self.suspect.iter().cloned().collect(),
            ChainEvent::Frozen { .. } | ChainEvent::TipAge { .. } => vec![],
        };
        targets
            .into_iter()
            .map(|txid| {
                let result =
                    self.apply_transition(&txid, |_, state| state.on_chain(headers, &event));
                let reask = self.states[&txid].reask().cloned();
                ChainUpdate {
                    txid,
                    result,
                    reask,
                }
            })
            .collect()
    }
    fn at_or_above(&self, height: Height) -> Vec<TxId> {
        self.mined
            .range(height..)
            .flat_map(|(_, ids)| ids.iter().cloned())
            .collect()
    }
}

fn remove_index<K: Ord>(index: &mut BTreeMap<K, BTreeSet<TxId>>, key: &K, txid: &str) {
    if let Some(ids) = index.get_mut(key) {
        ids.remove(txid);
        if ids.is_empty() {
            index.remove(key);
        }
    }
}
