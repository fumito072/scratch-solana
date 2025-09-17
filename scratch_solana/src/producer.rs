use crate::poh::{Hash, Entry, hash_once};
use crate::slot::Slot;

#[derive(Clone, Debug)]
pub struct EntrySpec {
    pub num_hashes: u64,
    pub payload: Option<Vec<u8>>,
}

pub fn make_entry_from_spec(prev: &Hash, spec: &EntrySpec) -> Entry {
    let mut curr: Hash = *prev;

    let mut i: u64 = 0;
    while i + 1 < spec.num_hashes {
        curr = hash_once(&curr, None);
        i += 1;
    }

    let final_hash = match spec.payload.as_deref() {
        Some(p) => hash_once(&curr, Some(p)),
        None => hash_once(&curr, None),
    };

    Entry {
        num_hashes: spec.num_hashes,
        hash: final_hash,
        payload: spec.payload.clone(),
    }
}

pub fn produce_slot(prev: &Hash, slot_id: u64, specs: &[EntrySpec]) -> Slot {
    let mut curr: Hash = *prev;
    let mut entries: Vec<Entry> = Vec::with_capacity(specs.len());

    let mut i: usize = 0;
    while i < specs.len() {
        let e = make_entry_from_spec(&curr, &specs[i]);
        curr = e.hash;
        entries.push(e);
        i += 1;
    }

    Slot { slot: slot_id, last_hash: curr, entries }
}