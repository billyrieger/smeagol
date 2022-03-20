// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::leaf::Leaf;
use crate::mem::{Gen, Idx, NodeArena, NodeId};
use crate::util::{Grid2, ToGrid};

pub enum Cell {
    Off,
    On,
}

pub trait LifeRule {
    fn step(&self, cells: Leaf) -> Leaf;
}

pub struct B3S23;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Branch {
    pub side_log2: u8,
    pub is_empty: bool,
    pub gen: Gen,
    pub children: Grid2<Idx>,
}

impl Branch {
    pub(crate) fn child_ids(&self) -> Grid2<NodeId> {
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
    fn _as_leaf(&self) -> Option<&Leaf> {
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
    _rule: R,
}

impl<R> Universe<R>
where
    R: LifeRule,
{
    pub fn new() -> Self {
        todo!()
    }

    pub fn evolve(&mut self, root_id: NodeId, _steps: u64) -> NodeId {
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
