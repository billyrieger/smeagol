use crate::node::*;

impl NodeId {
    pub fn level(self, store: &Store) -> Level {
        match store.node(self) {
            Node::Leaf { .. } => Level(4),
            Node::Interior { level, .. } => level,
        }
    }

    pub fn population(self, store: &Store) -> u128 {
        match store.node(self) {
            Node::Leaf { grid } => u128::from(grid.count_ones().wrapping_sum()),
            Node::Interior { population, .. } => population,
        }
    }

    pub fn min_coord(self, store: &Store) -> i64 {
        match store.node(self) {
            Node::Leaf { .. } => -8,
            Node::Interior { level, .. } => -(1 << (level.0 - 1)),
        }
    }

    pub fn max_coord(self, store: &Store) -> i64 {
        match store.node(self) {
            Node::Leaf { .. } => 7,
            Node::Interior { level, .. } => (1 << (level.0 - 1)) - 1,
        }
    }
}
