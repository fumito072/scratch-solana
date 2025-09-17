use crate::poh::Hash;
use crate::schedule::LeaderSchedule;
use crate::producer::{EntrySpec, produce_slot};
use crate::slot::{Slot, verify_slot};


pub fn drive_round_robin(
    schedule: &LeaderSchedule,
    me: &str,
    mut prev: Hash,
    start_slot: u64,
    n_slots: u64,
) -> Vec<Slot> {
    let mut out: Vec<Slot> = Vec::with_capacity(n_slots as usize);

    let mut s = start_slot;
    let mut k: u64 = 0;
    while k < n_slots {
        let leader = schedule.leader_for_slot(s).expect("schedule must be non-empty");

        let specs: Vec<EntrySpec> = if leader == me {
            let text = format!("ME:{}:slot={}", me, s);
            vec![
                EntrySpec { num_hashes: 2, payload: None },
                EntrySpec { num_hashes: 3, payload: Some(text.into_bytes()) },
            ]
        } else {
            let text = format!("OTHER:{}:slot={}", leader, s);
            vec![
                EntrySpec { num_hashes: 1, payload: None },
                EntrySpec { num_hashes: 2, payload: Some(text.into_bytes()) },
            ]
        };
        let slot = produce_slot(&prev, s, &specs);
        assert!(verify_slot(&prev, &slot), "slot {} verification failed", s);

        println!(
            "slot {:>3} leader {:>3}  last_hash={}",
            s,
            leader,
            hex::encode(slot.last_hash)
        );

        prev = slot.last_hash;
        out.push(slot);
        s += 1;
        k += 1;
    }
    out
}
