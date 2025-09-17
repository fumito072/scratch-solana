mod poh;
mod schedule;
mod producer;
mod slot;
mod driver;

use poh::{Hash, Entry, hash_once, hash_n, verify_entry};
use schedule::LeaderSchedule;
use driver::drive_round_robin;

fn main() {
    let sched = LeaderSchedule {
        order: vec!["A".into(), "B".into(), "C".into()],
    };

    let mut s = 0u64;
    while s < 7 {
        let who = sched.leader_for_slot(s).unwrap();
        println!("slot {s} → leader {who}");
        s += 1;
    }

    assert!(sched.is_leader(0, "A"));
    assert!(sched.is_leader(1, "B"));
    assert!(sched.is_leader(2, "C"));
    assert!(sched.is_leader(3, "A")); // 3 % 3 = 0
    println!("round-robin OK");
}