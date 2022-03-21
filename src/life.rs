// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::leaf::{Leaf, QuarterLeaf};
use crate::util::{BitGrid, Dir, Grid2, ToGrid};
use indexmap::IndexMap;

pub trait LifeRule {
    fn tick<B: BitGrid>(&self, cells: B) -> B;
}

pub struct B3S23;

impl LifeRule for B3S23 {
    fn tick<B: BitGrid>(&self, a: B) -> B {
        // TODO: figure out how this works.
        // Original algorithm:
        //     Rokicki, Tomas. “Life Algorithms,” June 28, 2018.
        let (aw, ae) = (a.shift(Dir::West), a.shift(Dir::East));
        let (s0, s1) = (aw ^ ae, aw & ae);
        let (hs0, hs1) = (s0 ^ a, (s0 & a) | s1);
        let (hs0w8, hs0e8) = (hs0.shift(Dir::North), hs0.shift(Dir::South));
        let (hs1w8, hs1e8) = (hs1.shift(Dir::North), hs1.shift(Dir::South));
        let ts0 = hs0w8 ^ hs0e8;
        let ts1 = (hs0w8 & hs0e8) | (ts0 & s0);
        (hs1w8 ^ hs1e8 ^ ts1 ^ s1) & ((hs1w8 | hs1e8) ^ (ts1 | s1)) & ((ts0 ^ s0) | a)
    }
}

pub fn do_it(cells: std::simd::u16x16) -> std::simd::u16x16 {
    B3S23.tick(cells)
}

#[test]
fn test() {
    use std::simd::u8x8;
    let mut rows = [0; 8];
    rows[2] = 0b00100;
    rows[3] = 0b01000;
    rows[4] = 0b01110;
    let x = u8x8::from_array(rows);
    dbg!(x);
    dbg!(B3S23.tick(B3S23.tick(B3S23.tick(B3S23.tick(x)))));
}

pub enum Cell {
    Off,
    On,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Branch {
    pub(crate) side_log2: u8,
    pub(crate) is_empty: bool,
    pub(crate) gen: Gen,
    pub(crate) child_indices: Grid2<Idx>,
}

impl Branch {
    fn from_children(children: Grid2<Node>) -> Self {
        Self {
            side_log2: children.nw.side_log2() + 1,
            is_empty: children.to_array().iter().all(|node| node.is_empty()),
            child_indices: todo!(),
            gen: todo!(),
        }
    }
}

impl Branch {
    pub(crate) fn children(&self) -> Grid2<NodeId> {
        self.child_indices
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

    pub fn side_log2(&self) -> u8 {
        match self {
            Self::Leaf(_) => Leaf::SIDE_LOG2,
            Self::Branch(branch) => branch.side_log2,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.is_empty(),
            Self::Branch(branch) => branch.is_empty,
        }
    }
}

pub struct Universe<R = B3S23> {
    nodes: NodeArena<()>,
    rule: R,
}

impl<R> Universe<R>
where
    R: LifeRule,
{
    pub fn new() -> Self {
        todo!()
    }

    pub fn evolve(&mut self, root_id: NodeId, ticks: u64) -> NodeId {
        let _: Option<_> = try {
            // NOTE: can only evolve a branch, not a leaf.
            let branch = self.nodes.entry(root_id)?.node.as_branch()?;
            let ticks0 = 0;
            let ticks1 = 0;
            if branch.side_log2 == Leaf::SIDE_LOG2 + 1 {
                // base case: children are leaves
                let kids: Grid2<Leaf> = branch
                    .children()
                    .to_array()
                    .try_map(|id| self.nodes.get_node(id)?.as_leaf().copied())?
                    .to_grid();
                let f0 = |leaf: Leaf| leaf.step(&self.rule, ticks0).center();
                let f1 = |leaf: Leaf| leaf.step(&self.rule, ticks1).center();
                kids.do_it(f0, f1, Leaf::to_parts, Leaf::from_parts);
            } else {
                let split = |id: NodeId| -> Grid2<NodeId> {
                    Option::unwrap(try { self.nodes.get_node(id)?.as_branch()?.children() })
                };
                let combine =
                    |parts: Grid2<NodeId>| -> NodeId { self.nodes.make_branch(parts).unwrap() };
                let f0 = |id: NodeId| self.evolve(id, ticks0 as u64);
                let f1 = |id: NodeId| self.evolve(id, ticks0 as u64);
                // branch.children().do_it(f0, f1, split, combine);
                // let _grandkids: Grid2<Grid2<_>> = branch
                //     .children()
                //     .to_array()
                //     .try_map(|id| Some(self.nodes.entry(id)?.node.as_branch()?.children()))?
                //     .to_grid();
            }
        };
        todo!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: Idx,
    generation: Gen,
}

impl NodeId {
    pub fn new(idx: Idx, gen: Gen) -> Self {
        Self {
            index: idx,
            generation: gen,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Idx(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Gen(u32);

pub struct Entry<'a, T> {
    pub node: &'a Node,
    pub data: &'a T,
}

pub struct NodeArena<T> {
    generation: Gen,
    nodes: IndexMap<Node, T, fnv::FnvBuildHasher>,
}

impl<T> NodeArena<T>
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
            child_indices: child_ids.to_array().map(|id| id.index).to_grid(),
        };
        Some(self.insert(Node::Branch(branch)))
    }
}
