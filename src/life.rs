use crate::util::{ArrayConcatExt, ArrayUnzipExt, Grid2, ToGrid};
use derive_more as dm;
use indexmap::IndexMap;
use std::ops::{Add, Shl, Shr, Sub};
use std::simd::{u16x16, u8x8};

// Derive macros from the standard library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// `derive_more` macros for bitwise operations.
#[derive(dm::BitAnd, dm::BitOr, dm::BitXor, dm::Not)]
pub struct Leaf {
    pub cells: u16x16,
}

impl Leaf {
    pub const SIDE: u8 = 16;
    pub const SIDE_LOG2: u8 = 4;

    pub const fn new(cells: u16x16) -> Self {
        Self { cells }
    }

    pub const fn empty() -> Self {
        Self::new(u16x16::splat(0))
    }

    pub fn from_parts(Grid2 { nw, ne, sw, se }: Grid2<QuarterLeaf>) -> Self {
        let west = nw.cells.to_array().array_concat(sw.cells.to_array());
        let east = ne.cells.to_array().array_concat(se.cells.to_array());
        let whole = west.zip(east).map(|(w, e)| u16::from_be_bytes([w, e]));
        Self::new(u16x16::from_array(whole))
    }

    pub fn to_parts(self) -> Grid2<QuarterLeaf> {
        let (west, east) = self
            .cells
            .to_array()
            .map(|row| row.to_be_bytes())
            .map(|[w, e]| (w, e))
            .unzip_array();
        let nw = west.split_array_ref().0;
        let ne = east.split_array_ref().0;
        let sw = west.rsplit_array_ref().1;
        let se = east.rsplit_array_ref().1;
        [*nw, *ne, *sw, *se]
            .map(u8x8::from_array)
            .map(QuarterLeaf::new)
            .to_grid()
    }

    pub fn population(&self) -> u128 {
        self.cells
            .to_array()
            .map(|row| u128::from(row.count_ones()))
            .iter()
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    pub fn center(&self) -> QuarterLeaf {
        // Start with 16 rows and 16 columns.
        let rows = self.cells.as_array();
        // Keep the first 12 of the 16 rows, thereby removing the last 4 rows.
        let rows: &[u16; 12] = rows.split_array_ref().0;
        // Then keep the last 8 of those 12 rows, thereby removing the first 4 rows.
        let rows: &[u16; 8] = rows.rsplit_array_ref().1;
        // Shift each row to the right by 4 and keep the right 8 columns.
        let rows: [u8; 8] = rows.map(|row| (row >> 4) as u8);
        // The final `u8x8` is the center of the original `u16x16`.
        QuarterLeaf::new(u8x8::from_array(rows))
    }
}

impl Shl<u16> for Leaf {
    type Output = Self;

    fn shl(self, rhs: u16) -> Self {
        Self::new(self.cells << u16x16::splat(rhs))
    }
}

impl Shr<u16> for Leaf {
    type Output = Self;

    fn shr(self, rhs: u16) -> Self {
        Self::new(self.cells >> u16x16::splat(rhs))
    }
}

impl Add<u16> for Leaf {
    type Output = Self;

    fn add(mut self, rhs: u16) -> Self {
        for _ in 0..rhs {
            self.cells = self.cells.rotate_lanes_left::<1>();
        }
        self
    }
}

impl Sub<u16> for Leaf {
    type Output = Self;

    fn sub(mut self, rhs: u16) -> Self {
        for _ in 0..rhs {
            self.cells = self.cells.rotate_lanes_right::<1>();
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuarterLeaf {
    cells: u8x8,
}

impl QuarterLeaf {
    fn new(cells: u8x8) -> Self {
        Self { cells }
    }
}

pub trait LifeRule {
    fn step(&self, cells: Leaf) -> Leaf;
}

pub struct B3S23;

impl LifeRule for B3S23 {
    fn step(&self, a: Leaf) -> Leaf {
        // TODO: figure out how this works.
        //
        // Original algorithm:
        //     Rokicki, Tomas. “Life Algorithms,” June 28, 2018.
        let (aw, ae) = (a << 1, a >> 1);
        let (s0, s1) = (aw ^ ae, aw & ae);
        let (hs0, hs1) = (s0 ^ a, (s0 & a) | s1);
        // I chose to abuse the addition/subtraction operators.
        let (hs0w8, hs0e8) = (hs0 + 1, hs0 - 1);
        let (hs1w8, hs1e8) = (hs1 + 1, hs1 - 1);
        let ts0 = hs0w8 ^ hs0e8;
        let ts1 = (hs0w8 & hs0e8) | (ts0 & s0);
        (hs1w8 ^ hs1e8 ^ ts1 ^ s1) & ((hs1w8 | hs1e8) ^ (ts1 | s1)) & ((ts0 ^ s0) | a)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Idx(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Gen(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: Idx,
    generation: Gen,
}

impl NodeId {
    fn new(idx: Idx, gen: Gen) -> Self {
        Self {
            index: idx,
            generation: gen,
        }
    }
}

pub struct Arena<T> {
    generation: Gen,
    nodes: IndexMap<Node, T, fnv::FnvBuildHasher>,
}

#[derive(Debug, Default)]
pub struct NoData;

pub struct Entry<'a, T> {
    pub node: &'a Node,
    pub data: &'a T,
}

impl<T> Arena<T>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            generation: Gen(0),
            nodes: IndexMap::default(),
        }
    }

    pub fn entry(&self, id: NodeId) -> Option<Entry<T>> {
        self.nodes
            .get_index(id.index.0 as usize)
            .map(|(node, data)| Entry { node, data })
    }

    pub fn get(&self, id: NodeId) -> Option<(&Node, &T)> {
        self.nodes.get_index(id.index.0 as usize)
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.get(id).map(|(k, _)| k)
    }

    pub fn get_data(&self, id: NodeId) -> Option<&T> {
        self.get(id).map(|(_, v)| v)
    }

    pub fn insert(&mut self, node: Node) -> NodeId {
        let entry = self.nodes.entry(node);
        let index = Idx(entry.index() as u32);
        // Actually insert the node into the indexmap.
        entry.or_default();
        NodeId {
            index,
            generation: self.generation,
        }
    }

    pub fn make_branch(&mut self, child_ids: Grid2<NodeId>) -> Option<NodeId> {
        let nodes: [&Node; 4] = child_ids.to_array().try_map(|id| self.get_node(id))?;
        let child_side_log2 = nodes[0].side_log2();
        assert!(nodes.iter().all(|node| node.side_log2() == child_side_log2));
        let branch = Branch {
            is_empty: nodes.iter().all(|node| node.is_empty()),
            side_log2: child_side_log2 + 1,
            gen: self.generation,
            children: child_ids.to_array().map(|id| id.index).to_grid(),
        };
        Some(self.insert(Node::Branch(branch)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Branch {
    side_log2: u8,
    is_empty: bool,
    gen: Gen,
    children: Grid2<Idx>,
}

impl Branch {
    fn child_ids(&self) -> Grid2<NodeId> {
        self.children
            .to_array()
            .map(|idx| NodeId::new(idx, self.gen))
            .to_grid()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    fn as_leaf(&self) -> Option<&Leaf> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            Self::Branch(_) => None,
        }
    }

    fn as_branch(&self) -> Option<&Branch> {
        match self {
            Self::Branch(branch) => Some(branch),
            Self::Leaf(_) => None,
        }
    }

    fn side_log2(&self) -> u8 {
        match self {
            Self::Leaf(_) => Leaf::SIDE_LOG2,
            Self::Branch(branch) => branch.side_log2,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.is_empty(),
            Self::Branch(branch) => branch.is_empty,
        }
    }
}

pub struct Universe<R = B3S23> {
    nodes: Arena<()>,
    rule: R,
}

impl<R> Universe<R>
where
    R: LifeRule,
{
    pub fn new() -> Self {
        todo!()
    }

    pub fn evolve(&mut self, root_id: NodeId, steps: u64) -> NodeId {
        let _: Option<_> = try {
            let branch = self.nodes.entry(root_id)?.node.as_branch()?;
            let kids = branch.child_ids();
            if branch.side_log2 == Leaf::SIDE_LOG2 + 1 {
                // base case: children are leaves
            } else {
                let _grandkids: Grid2<Grid2<_>> = kids
                    .to_array()
                    .try_map(|id| Some(self.nodes.entry(id)?.node.as_branch()?.child_ids()))?
                    .to_grid();
            }
        };
        todo!()
    }
}
