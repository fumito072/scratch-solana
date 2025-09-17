use crate::poh::{Hash, Entry, verify_entry};

#[derive(Clone, Debug)]
pub struct Slot { // block
    pub slot: u64,
    pub last_hash: Hash,
    pub entries: Vec<Entry>,
}

pub fn verify_slot(prev: &Hash, slot: &Slot) -> bool {
    let mut curr: Hash = *prev;

    let mut i: usize = 0;
    while i < slot.entries.len() {
        let e = &slot.entries[i];
        if !verify_entry(&curr, e) {
            return false;
        }
        curr = e.hash;
        i += 1;
    }
    curr == slot.last_hash
}