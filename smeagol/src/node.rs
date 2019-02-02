mod cells;
mod evolve;
mod properties;
mod subregion;

use crate::Cell;

/// The maximum level a node can have.
pub const MAX_LEVEL: u8 = 64;

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

impl std::hash::Hash for Node {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.base.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Node {
    base: NodeBase,
    level: u8,
    contains_alive_cells: bool,
}

/// Internal node creation methods.
impl Node {
    /// Creates a new leaf node corresponding to the given cell.
    pub(crate) fn new_leaf(cell: Cell) -> Self {
        let base = NodeBase::Leaf {
            alive: cell.is_alive(),
        };
        let level = 0;
        let contains_alive_cells = cell.is_alive();
        Self { base, level, contains_alive_cells }
    }

    /// Creates a new interior node with the given level, children node indices, and boolean
    /// representing whether the node contains any alive cells.
    ///
    /// The node indices should be in the following order: northeast, northwest, southeast,
    /// southwest.
    ///
    /// # Panics
    ///
    /// Panics if `level > MAX_LEVEL`.
    pub(crate) fn new_interior(level: u8, indices: [usize; 4], contains_alive_cells: bool) -> Self {
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
            contains_alive_cells,
        }
    }
}
