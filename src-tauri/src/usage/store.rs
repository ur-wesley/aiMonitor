use parking_lot::RwLock;

use crate::models::UsageSnapshot;

pub struct UsageStore {
    current: RwLock<UsageSnapshot>,
}

impl UsageStore {
    pub fn new() -> Self {
        Self {
            current: RwLock::new(UsageSnapshot::empty()),
        }
    }

    pub fn current(&self) -> UsageSnapshot {
        self.current.read().clone()
    }

    pub fn update(&self, snapshot: UsageSnapshot) {
        *self.current.write() = snapshot;
    }
}
