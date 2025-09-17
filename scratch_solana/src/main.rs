use sha2::{Sha256, Digest};

type Hash = [u8; 32];

#[derive(Clone, Debug)]
struct Entry {
    num_hashes: u64,
    hash: Hash,
    payload: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
struct Slot {
    slot: u64,
    last_hash: Hash,
    entries: Vec<Entry>,
}

fn hash_once(prev: &Hash, mixin: Option<&[u8]>) -> Hash {
    let mut hasher = Sha256::new();

    hasher.update(prev);

    if let Some(data) = mixin {
        hasher.update(data);
    }
    hasher.finalize().into()
}

fn hash_n(prev: &Hash, n: u64) -> Hash {
    let mut curr: Hash = *prev;

    let mut i: u64 = 0;
    while i < n {
        curr = hash_once(&curr, None);
        i += 1;
    }
    curr
}

fn verify_entry(prev: &Hash, entry: &Entry) -> bool {
    if entry.num_hashes == 0 {
        return false;
    }

    let mut curr: Hash = *prev;

    let mut i: u64 = 0;
    while i + 1 < entry.num_hashes {
        curr = hash_once(&curr, None);
        i += 1;
    }

    let final_hash = match entry.payload.as_ref() {
        Some(p) => hash_once(&curr, Some(p.as_slice())),
        None => hash_once(&curr, None)
    };

    final_hash == entry.hash
}

fn verify_slot(prev: &Hash, slot: &Slot) -> bool {
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

fn main() {
    let zero: Hash = [0u8; 32];

    // 例1: payload なし、num_hashes=3
    let mut curr = zero;
    let mut i = 0;
    while i + 1 < 3 {
        curr = hash_once(&curr, None);
        i += 1;
    }
    let expected_hash = hash_once(&curr, None);

    let e1 = Entry { num_hashes: 3, hash: expected_hash, payload: None };
    assert!(verify_entry(&zero, &e1));

    // 例2: payload あり、num_hashes=2
    let mut curr2 = zero;
    let mut j = 0;
    while j + 1 < 2 {
        curr2 = hash_once(&curr2, None);
        j += 1;
    }
    let expected_hash2 = hash_once(&curr2, Some(b"HELLO"));
    let e2 = Entry { num_hashes: 2, hash: expected_hash2, payload: Some(b"HELL".to_vec()) };
    assert!(verify_entry(&zero, &e2));

    println!("verify_entry OK");
}
