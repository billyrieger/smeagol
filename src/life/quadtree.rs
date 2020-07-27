// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::rule::{Leaf, Rule},
    util::{
        grid::{Grid2, Grid3, Grid4},
        memory::Arena,
    },
    Error, Result,
};

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct NodeId {
    level: u8,
    index: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    fn level(&self) -> u8 {
        match self {
            Node::Leaf(_) => Leaf::LEVEL,
            Node::Branch(b) => b.level,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Branch {
    children: Grid2<NodeId>,
    level: u8,
}

impl Branch {
    fn from_children(children: Grid2<NodeId>) -> Self {
        assert_eq!(children.nw.level, children.ne.level);
        assert_eq!(children.nw.level, children.sw.level);
        assert_eq!(children.nw.level, children.se.level);

        Branch {
            level: children.nw.level + 1,
            children,
        }
    }
}

pub struct Tree<R> {
    arenas: Vec<Arena<Node>>,
    root: NodeId,
    rule: R,
    jump_cache: HashMap<Branch, NodeId>,
}

impl<R> Tree<R>
where
    R: Rule,
{
    const MIN_LEVEL: u8 = Leaf::LEVEL;
    const MAX_LEVEL: u8 = 63;

    pub fn new(rule: R) -> Self {
        let mut arenas = Vec::<Arena<Node>>::new();

        arenas.push(Arena::with_value(Node::Leaf(Leaf::default())));

        for level in (Self::MIN_LEVEL + 1)..=Self::MAX_LEVEL {
            let previous_id = NodeId {
                level: level - 1,
                index: 0,
            };
            let branch = Branch::from_children(Grid2::repeat(previous_id));
            arenas.push(Arena::with_value(Node::Branch(branch)));
        }

        let root = NodeId {
            level: Self::MAX_LEVEL,
            index: 0,
        };

        Self {
            arenas,
            root,
            rule,
            jump_cache: HashMap::new(),
        }
    }

    pub fn step(&mut self, steps: u64) -> Result<()> {
        match self.get_node(self.root)? {
            Node::Leaf(_) => unreachable!(),
            Node::Branch(branch) => {
                self.root = self.evolve(branch, steps)?;
                Ok(())
            }
        }
    }

    fn empty(&self, level: u8) -> Result<Node> {
        let empty_id = NodeId { level, index: 0 };
        self.get_node(empty_id)
    }

    fn register(&mut self, node: Node) -> NodeId {
        let level = node.level();
        let arena_index = self.arena_index(level);
        let index = self.arenas[arena_index].register(node);
        NodeId { level, index }
    }

    fn arena_index(&self, level: u8) -> usize {
        (level - Self::MIN_LEVEL) as usize
    }

    fn get_node(&self, id: NodeId) -> Result<Node> {
        self.arenas
            .get(self.arena_index(id.level))
            .and_then(|arena| arena.get(id.index))
            .ok_or(Error)
    }

    fn evolve(&mut self, branch: Branch, steps: u64) -> Result<NodeId> {
        if steps == 1 << (branch.level - 2) {
            if let Some(&jump) = self.jump_cache.get(&branch) {
                return Ok(jump);
            }
        }

        let children: Grid2<Node> = branch.children.try_map(|id| self.get_node(id))?;

        let node = match children.unpack() {
            [Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)] => {
                let leaves: Grid2<Leaf> = Grid2::pack([nw, ne, sw, se]);
                let new_leaf = self.rule.evolve(leaves, steps);
                Node::Leaf(new_leaf)
            }

            [Node::Branch(nw), Node::Branch(ne), Node::Branch(sw), Node::Branch(se)] => {
                let branches: Grid2<Branch> = Grid2::pack([nw, ne, sw, se]);

                let grandchildren: Grid4<NodeId> = branches.map(|branch| branch.children).flatten();

                let partial: Grid3<NodeId> =
                    grandchildren.shrink(|ids| self.evolve(Branch::from_children(ids), steps))?;
                let complete: Grid2<NodeId> =
                    partial.shrink(|ids| self.evolve(Branch::from_children(ids), steps))?;

                Node::Branch(Branch::from_children(complete))
            }

            _ => Err(Error)?,
        };
        let id = self.register(node);

        if steps == 1 << (branch.level - 2) {
            self.jump_cache.insert(branch, id);
        }

        Ok(id)
    }
}
