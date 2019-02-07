use crate::{
    node::{NodeBase, MAX_LEVEL},
    Node, Store,
};

impl Node {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn population(&self, store: &Store) -> u128 {
        match self.base {
            NodeBase::Leaf { alive } => {
                if alive {
                    1
                } else {
                    0
                }
            }
            NodeBase::LevelOne { cells } => u128::from(cells.count_ones()),
            NodeBase::LevelTwo { cells } => u128::from(cells.count_ones()),
            _ => store.population(&self),
        }
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
