/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Inner workings of `smeagol`.
mod impls;
mod store;

pub use self::store::{NodeTemplate, Store};
use packed_simd::u16x16;
use std::hash::{Hash, Hasher};

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
    let sw_grid = sw_grid << 8;
    let left: u16x16 = shuffle!(
        nw_grid, sw_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
    );

    let ne_grid = ne_grid >> 8;
    let se_grid = se_grid >> 8;
    let right: u16x16 = shuffle!(
        ne_grid, se_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
    );

    left | right
}

/// An index in a store.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Index(u32);

/// The level of a node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(pub u8);

/// The four quadrants of a node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId {
    /// The index of the node in the store.
    index: Index,
}

/// An immutable quadtree representation of a Life grid.
#[derive(Clone, Copy, Debug)]
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

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        match (self, other) {
            (Node::Leaf { grid }, Node::Leaf { grid: other_grid }) => grid == other_grid,
            (
                Node::Interior { nw, ne, sw, se, .. },
                Node::Interior {
                    nw: other_nw,
                    ne: other_ne,
                    sw: other_sw,
                    se: other_se,
                    ..
                },
            ) => nw == other_nw && ne == other_ne && sw == other_sw && se == other_se,
            _ => false,
        }
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Node::Leaf { grid } => grid.hash(state),
            Node::Interior { nw, ne, sw, se, .. } => {
                nw.hash(state);
                ne.hash(state);
                sw.hash(state);
                se.hash(state);
            }
        }
    }
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
