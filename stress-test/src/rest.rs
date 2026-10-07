use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct Exhaustion {
    level: usize,
}

impl Exhaustion {
    pub fn increase(&mut self) {
        if self.level < 6 {
            self.level += 1
        }
    }

    pub fn reset(&mut self) {
        self.level = 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Rest {
    Short,
    Long,
}