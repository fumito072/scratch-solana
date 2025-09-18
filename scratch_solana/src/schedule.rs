pub type NodeId = String;

#[derive(Clone, Debug)]
pub struct LeaderSchedule {
    pub order: Vec<NodeId>,
}

impl LeaderSchedule {
    pub fn new(order: Vec<NodeId>) -> Result<Self, &'static str> {
        if order.is_empty() {
            return Err("order must be non-empty");
        }
        Ok(Self { order })
    }

    pub fn leader_for_slot(&self, slot: u64) -> Option<&str> {
        if self.order.is_empty() {
            return None;
        }
        let i: usize = (slot as usize) % self.order.len();
        Some(self.order[i].as_str())
    }

    pub fn is_leader(&self, slot: u64, me: &str) -> bool {
        match self.leader_for_slot(slot) {
            Some(l) => l == me,
            None => false,
        }
    }
}