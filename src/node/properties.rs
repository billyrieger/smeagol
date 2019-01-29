use crate::{node::MAX_LEVEL, Node};

impl Node {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn population(&self) -> u128 {
        self.population
    }

    pub fn min_coord(&self) -> i64 {
        if self.level == 0 {
            0
        } else if self.level < MAX_LEVEL {
            -(1 << (self.level - 1))
        } else {
            i64::min_value()
        }
    }

    pub fn max_coord(&self) -> i64 {
        if self.level == 0 {
            0
        } else if self.level < MAX_LEVEL {
            (1 << (self.level - 1)) - 1
        } else {
            i64::max_value()
        }
    }
}
