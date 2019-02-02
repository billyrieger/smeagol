use crate::{node::MAX_LEVEL, Node, Store};

impl Node {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn population(&self, store: &mut Store) -> u128 {
        store.population(&self)
    }

    pub fn contains_alive_cells(&self) -> bool {
        self.contains_alive_cells
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
