mod cells;
mod properties;
mod subregion;

use crate::Cell;

const MAX_LEVEL: u8 = 64;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
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

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Node {
    base: NodeBase,
    level: u8,
    population: u128,
}
