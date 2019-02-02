mod cells;
mod evolve;
mod properties;
mod subregion;

const MAX_LEVEL: u8 = 64;

const LEVEL_ONE_MASK: u8 = 0b_0011_0011;
const LEVEL_ONE_NW_MASK: u8 = 0b_0010_0000;
const LEVEL_ONE_NE_MASK: u8 = 0b_0001_0000;
const LEVEL_ONE_SW_MASK: u8 = 0b_0000_0010;
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

#[derive(Clone, Copy, Debug)]
pub struct Node {
    base: NodeBase,
    level: u8,
}

/// Internal node creation methods.
impl Node {
    pub(crate) fn new_leaf(alive: bool) -> Self {
        Self {
            base: NodeBase::Leaf { alive },
            level: 0,
        }
    }

    pub(crate) fn new_level_one(cells: u8) -> Self {
        Self {
            base: NodeBase::LevelOne { cells },
            level: 1,
        }
    }

    pub(crate) fn new_level_two(cells: u16) -> Self {
        Self {
            base: NodeBase::LevelTwo { cells },
            level: 2,
        }
    }

    pub(crate) fn new_interior(level: u8, indices: [usize; 4]) -> Self {
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
        }
    }

    pub(crate) fn create_level_one(ne: Node, nw: Node, se: Node, sw: Node) -> (Self, u128) {
        match (ne.base, nw.base, se.base, sw.base) {
            (
                NodeBase::Leaf { alive: ne_alive },
                NodeBase::Leaf { alive: nw_alive },
                NodeBase::Leaf { alive: se_alive },
                NodeBase::Leaf { alive: sw_alive },
            ) => {
                let mut cells = 0u8;
                if nw_alive {
                    cells |= 0b_0010_0000;
                }
                if ne_alive {
                    cells |= 0b_0001_0000;
                }
                if sw_alive {
                    cells |= 0b_0000_0010;
                }
                if se_alive {
                    cells |= 0b_0000_0001;
                }

                (
                    Self {
                        base: NodeBase::LevelOne { cells },
                        level: 1,
                    },
                    cells.count_ones() as u128,
                )
            }
            _ => panic!(),
        }
    }

    pub(crate) fn create_level_two(ne: Node, nw: Node, se: Node, sw: Node) -> (Self, u128) {
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
                    },
                    cells.count_ones() as u128,
                )
            }
            _ => panic!(),
        }
    }
}
