///! Node module.
mod store;
mod impls;

pub use self::store::{Store, NodeTemplate};

/// The maximum level a node can have.
const MAX_LEVEL: u8 = 64;

/// Bitmask to extract the bits of a level one node.
const LEVEL_ONE_MASK: u8 = 0b_0011_0011;
/// Bitmask to extract the northwest leaf of a level one node.
const LEVEL_ONE_NW_MASK: u8 = 0b_0010_0000;
/// Bitmask to extract the northeast leaf of a level one node.
const LEVEL_ONE_NE_MASK: u8 = 0b_0001_0000;
/// Bitmask to extract the southwest leaf of a level one node.
const LEVEL_ONE_SW_MASK: u8 = 0b_0000_0010;
/// Bitmask to extract the southeast leaf of a level one node.
const LEVEL_ONE_SE_MASK: u8 = 0b_0000_0001;

/// The inner enum representing a type of node.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum NodeBase {
    /// A level zero (1 by 1) node.
    Leaf {
        /// Whether or not the only cell in the node is alive.
        alive: bool,
    },

    /// A level one (2 by 2) node.
    LevelOne {
        /// The node
        ///
        /// ```txt
        /// +---+---+
        /// | a | b |
        /// +---+---+
        /// | c | d |
        /// +---+---+
        /// ```
        ///
        /// is represented by the eight bits
        ///
        /// ```txt
        /// 00ab_00cd
        /// ```
        ///
        /// The gaps make combining level one nodes into a level two node easier. Idea shamelessly
        /// stolen from [here](https://github.com/rntz/rust-hashlife/blob/master/hashlife.rs).
        cells: u8,
    },

    /// A level two (4 by 4) node.
    LevelTwo {
        /// The node
        ///
        /// ```txt
        /// +---+---+---+---+
        /// | a | b | c | d |
        /// +---+---+---+---+
        /// | e | f | g | h |
        /// +---+---+---+---+
        /// | i | j | k | l |
        /// +---+---+---+---+
        /// | m | n | o | p |
        /// +---+---+---+---+
        /// ```
        ///
        /// is represented by the sixteen bits
        ///
        /// ```txt
        /// abcd_efgh_ijkl_mnop
        /// ```
        cells: u16,
    },

    /// A node with a level greater than two.
    Interior {
        /// Index of the northeast child in the store.
        ne_index: usize,

        /// Index of the northwest child in the store.
        nw_index: usize,

        /// Index of the southeast child in the store.
        se_index: usize,

        /// Index of the southwest child in the store.
        sw_index: usize,
    },
}

/// An immutable quadtree representation of a Game of Life state.
///
/// # Introduction
///
/// A level `n` node represents a `2^n` by `2^n` square grid of dead or alive cells. A level 0 node
/// is called a leaf node, which contains a single cell. Leaf nodes (as well as level 1 and 2
/// nodes) store their cells directly as bits. Higher-level nodes take a different approach: each
/// node is itself composed of four smaller nodes, one for each quadrant. A node only has to store
/// the indices of its four children nodes, since all nodes live in the store.
///
/// # Coordinate system
///
/// Individual cells of a node can be accessed via the two-dimensional coordinates `(x, y)`.
/// Keeping with existing convention, the positive x direction is east and the positive y direction
/// is south.
///
/// For a level `n` node, `n > 0`, `(-2^(n-1), -2^(n-1))` is at the northwest corner of the node
/// and `(2^(n-1) - 1, 2^(n-1) - 1)` is at the southeast corner of the node. This places the origin
/// `(0, 0)` at the southeast tile nearest the center of the node.
///
/// The benefit of having the origin at the center is that the center subnode of a node shares the
/// same coordinate system as its parent, meaning the coordinate system doesn't change when a node
/// is advanced into the future.
///
/// Another benefit is that checking which quadrant a coordinate pair `(x, y)` is in is simply a
/// matter of checking the signs of the coordinates:
///
/// ```
/// # let x = 0;
/// # let y = 0;
/// let quadrant = match (x < 0, y < 0) {
///     (true, true) => "northwest",
///     (true, false) => "southwest",
///     (false, true) => "northeast",
///     (false, false) => "southeast",
/// };
/// ```
///
/// For level 0 node (leaf node), the only valid coordinate is `(0, 0)`.
#[derive(Clone, Copy, Debug)]
pub struct Node {
    base: NodeBase,
    level: u8,
    index: Option<usize>,
}

/// Internal node creation methods.
impl Node {
    fn new_leaf(alive: bool) -> Self {
        Self {
            base: NodeBase::Leaf { alive },
            level: 0,
            index: None,
        }
    }

    fn new_level_one(cells: u8) -> Self {
        Self {
            base: NodeBase::LevelOne {
                cells: cells & LEVEL_ONE_MASK,
            },
            level: 1,
            index: None,
        }
    }

    fn new_level_two(cells: u16) -> Self {
        Self {
            base: NodeBase::LevelTwo { cells },
            level: 2,
            index: None,
        }
    }

    fn new_interior(level: u8, indices: [usize; 4]) -> Self {
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
            index: None,
        }
    }

    fn create_level_one(ne: Node, nw: Node, se: Node, sw: Node) -> (Self, u128) {
        match (ne.base, nw.base, se.base, sw.base) {
            (
                NodeBase::Leaf { alive: ne_alive },
                NodeBase::Leaf { alive: nw_alive },
                NodeBase::Leaf { alive: se_alive },
                NodeBase::Leaf { alive: sw_alive },
            ) => {
                let mut cells = 0u8;
                if nw_alive {
                    cells |= LEVEL_ONE_NW_MASK
                }
                if ne_alive {
                    cells |= LEVEL_ONE_NE_MASK;
                }
                if sw_alive {
                    cells |= LEVEL_ONE_SW_MASK;
                }
                if se_alive {
                    cells |= LEVEL_ONE_SE_MASK;
                }

                (
                    Self {
                        base: NodeBase::LevelOne { cells },
                        level: 1,
                        index: None,
                    },
                    u128::from(cells.count_ones()),
                )
            }
            _ => panic!(),
        }
    }

    fn create_level_two(ne: Node, nw: Node, se: Node, sw: Node) -> (Self, u128) {
        match (ne.base, nw.base, se.base, sw.base) {
            (
                NodeBase::LevelOne { cells: ne_cells },
                NodeBase::LevelOne { cells: nw_cells },
                NodeBase::LevelOne { cells: se_cells },
                NodeBase::LevelOne { cells: sw_cells },
            ) => {
                let north = (nw_cells << 2) | ne_cells;
                let south = (sw_cells << 2) | se_cells;
                let cells = u16::from_be_bytes([north, south]);
                (
                    Self {
                        base: NodeBase::LevelTwo { cells },
                        level: 2,
                        index: None,
                    },
                    u128::from(cells.count_ones()),
                )
            }
            _ => panic!(),
        }
    }

    fn index(&self) -> usize {
        self.index.unwrap()
    }

    fn set_index(&self, index: usize) -> Node {
        Self {
            index: Some(index),
            ..*self
        }
    }
}

impl Eq for Node {}

impl std::hash::Hash for Node {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.base.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.base == other.base
    }
}
