use sha2::{Sha256, Digest};

pub type Hash = [u8; 32];

#[derive(Clone, Debug)]
pub struct Entry { // transaction
    pub num_hashes: u64,
    pub hash: Hash,
    pub payload: Option<Vec<u8>>,
}

pub fn hash_once(prev: &Hash, mixin: Option<&[u8]>) -> Hash {
    let mut hasher = Sha256::new();

    hasher.update(prev);

    if let Some(data) = mixin {
        hasher.update(data);
    }
    hasher.finalize().into()
}

pub fn hash_n(prev: &Hash, n: u64) -> Hash {
    let mut curr: Hash = *prev;

    let mut i: u64 = 0;
    while i < n {
        curr = hash_once(&curr, None);
        i += 1;
    }
    curr
}

pub fn verify_entry(prev: &Hash, entry: &Entry) -> bool {
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