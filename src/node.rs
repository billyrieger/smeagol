mod cells;
mod properties;
mod subregion;

use crate::Cell;

const MAX_LEVEL: u8 = 64;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum NodeBase {
    Leaf {
        alive: bool,
    },
    Interior {
        ne_index: usize,
        nw_index: usize,
        se_index: usize,
        sw_index: usize,
    },
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Node {
    base: NodeBase,
    level: u8,
    population: u128,
}

/// Internal node creation methods.
impl Node {
    pub(crate) fn new_leaf(cell: Cell) -> Self {
        let base = NodeBase::Leaf {
            alive: cell.is_alive(),
        };
        let level = 0;
        let population = if cell.is_alive() { 1 } else { 0 };
        Self {
            base,
            level,
            population,
        }
    }

    pub(crate) fn new_interior(level: u8, population: u128, indices: [usize; 4]) -> Self {
        if level > MAX_LEVEL {
            panic!("cannot create a node with level above {}", MAX_LEVEL);
        }
        Self {
            base: NodeBase::Interior {
                ne_index: indices[0],
                nw_index: indices[1],
                se_index: indices[2],
                sw_index: indices[3],
            },
            level,
            population,
        }
    }
}
