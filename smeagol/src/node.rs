use packed_simd::u16x16;

mod impls;
mod store;

pub use self::store::{NodeTemplate, Store};

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
pub struct Index(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(pub u8);

pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId {
    index: Index,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Node {
    Leaf {
        grid: u16x16,
    },
    Interior {
        nw: NodeId,
        ne: NodeId,
        sw: NodeId,
        se: NodeId,
        level: Level,
        population: u128,
    },
}

impl Node {
    pub fn unwrap_leaf(&self) -> u16x16 {
        match *self {
            Node::Leaf { grid } => grid,
            Node::Interior { .. } => panic!(),
        }
    }

    pub fn unwrap_interior(&self) -> (NodeId, NodeId, NodeId, NodeId, Level, u128) {
        match *self {
            Node::Leaf { .. } => panic!(),
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                population,
            } => (nw, ne, sw, se, level, population),
        }
    }
}
