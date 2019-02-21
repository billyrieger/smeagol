//! Inner workings of `smeagol`.

mod impls;
mod store;

pub use self::store::{NodeTemplate, Store};
use packed_simd::u16x16;

const LEVEL_4_UPPER_HALF_MASK: u16x16 = u16x16::new(
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
);

const LEVEL_4_LOWER_HALF_MASK: u16x16 = u16x16::new(
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
    0b1111_1111_1111_1111,
);

const LEVEL_4_NW_MASK: u16x16 = u16x16::new(
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
);

const LEVEL_4_NE_MASK: u16x16 = u16x16::new(
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
);

const LEVEL_4_SW_MASK: u16x16 = u16x16::new(
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
    0b1111_1111_0000_0000,
);

const LEVEL_4_SE_MASK: u16x16 = u16x16::new(
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
    0b0000_0000_1111_1111,
);

fn center(nw_grid: u16x16, ne_grid: u16x16, sw_grid: u16x16, se_grid: u16x16) -> u16x16 {
    let nw_grid = nw_grid << 8;
    let nw_grid = shuffle!(
        nw_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7]
    ) & LEVEL_4_UPPER_HALF_MASK;

    let ne_grid = ne_grid >> 8;
    let ne_grid = shuffle!(
        ne_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7]
    ) & LEVEL_4_UPPER_HALF_MASK;

    let sw_grid = sw_grid << 8;
    let sw_grid = shuffle!(
        sw_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7]
    ) & LEVEL_4_LOWER_HALF_MASK;

    let se_grid = se_grid >> 8;
    let se_grid = shuffle!(
        se_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7]
    ) & LEVEL_4_LOWER_HALF_MASK;

    nw_grid | ne_grid | sw_grid | se_grid
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Index(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(pub u8);

/// The four quadrants of a node.
pub enum Quadrant {
    /// The northwest quadrant.
    Northwest,
    /// The northeast quadrant.
    Northeast,
    /// The southwest quadrant.
    Southwest,
    /// The southeast quadrant.
    Southeast,
}

/// An identifier referring to a node in a store.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId {
    /// The index of the node in the store.
    index: Index,
}

/// An immutable quadtree representation of a Life grid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Node {
    /// A leaf (16 by 16) node.
    Leaf {
        /// The grid itself.
        ///
        /// 1 represents an alive cell, 0 represents a dead cell.
        grid: u16x16,
    },
    /// A non-leaf node.
    Interior {
        /// The northwest child.
        nw: NodeId,
        /// The northeast child.
        ne: NodeId,
        /// The southwest child.
        sw: NodeId,
        /// The southeast child.
        se: NodeId,
        /// The level of the node.
        level: Level,
        /// The number of alive cells in the node.
        population: u128,
    },
}

/// Internal methods.
impl Node {
    /// Returns the inner grid of a leaf node.
    ///
    /// # Panics
    ///
    /// Panics if the node is not a leaf.
    fn unwrap_leaf(&self) -> u16x16 {
        match *self {
            Node::Leaf { grid } => grid,
            Node::Interior { .. } => panic!(),
        }
    }
}
