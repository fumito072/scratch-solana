use sha2::{Sha256, Digest};

type Hash = [u8; 32];

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

fn main() {
    let zero: Hash = [0u8; 32];

    let a0 = hash_n(&zero, 0);
    let a1 = hash_n(&zero, 1);
    let once = hash_once(&zero, None);

    println!("zero = {}", hex::encode(zero)); // 64個の'0'が並ぶ
    println!("a0   = {}", hex::encode(a0));   // 修正後は zero と同じ
    println!("a1   = {}", hex::encode(a1));   // 1回だけ回した値
    println!("once = {}", hex::encode(once)); // a1 と一致する

    assert_eq!(a0, zero);
    assert_eq!(a1, once);
}

