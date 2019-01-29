use crate::Cell;

const MAX_LEVEL: u8 = 64;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
enum NodeBase {
    Leaf {
        alive: bool,
    },

    LevelOne {
        cells: u8,
    },

    LevelTwo {
        cells: u16,
    },

    Interior {
        ne_index: usize,
        nw_index: usize,
        se_index: usize,
        sw_index: usize,
    },
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Node {
    base: NodeBase,
    level: u8,
    population: u128,
}

impl Node {
    pub(crate) fn new_leaf(cell: Cell) -> Self {
        let base = NodeBase::Leaf { alive: cell.is_alive() };
        let level = 0;
        let population = if cell.is_alive() { 1 } else { 0 };
        Self {
            base,
            level,
            population,
        }
    }
}
