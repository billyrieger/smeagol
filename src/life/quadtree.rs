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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Level(u8);

impl Level {
    fn increment(&self) -> Level {
        Level(self.0 + 1)
    }

    fn decrement(&self) -> Level {
        Level(self.0 + 1)
    }

    fn side_len(&self) -> u64 {
        1 << self.0
    }

    fn max_steps(&self) -> u64 {
        self.side_len() / 4
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct NodeId {
    level: Level,
    index: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Node<L> {
    Leaf(L),
    Branch(Branch),
}

impl<L> Node<L>
where
    L: Leaf,
{
    fn level(&self) -> Level {
        match self {
            Node::Leaf(_) => Level(L::LOG_SIDE_LEN),
            Node::Branch(b) => b.level,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Branch {
    children: Grid2<NodeId>,
    level: Level,
}

impl Branch {
    fn from_children(children: Grid2<NodeId>) -> Self {
        debug_assert_eq!(children.nw.level, children.ne.level);
        debug_assert_eq!(children.nw.level, children.sw.level);
        debug_assert_eq!(children.nw.level, children.se.level);

        Branch {
            level: children.nw.level.increment(),
            children,
        }
    }
}

pub struct Tree<L, R> {
    arenas: Vec<Arena<Node<L>>>,
    root: NodeId,
    rule: R,
    jump_cache: HashMap<Branch, NodeId>,
}

impl<L, R> Tree<L, R>
where
    L: Leaf,
    R: Rule<Leaf = L>,
{
    fn min_level() -> Level {
        Level(L::LOG_SIDE_LEN)
    }

    fn max_level() -> Level {
        Level(63)
    }

    pub fn new(rule: R) -> Self {
        let min: u8 = Self::min_level().0;
        let max: u8 = Self::max_level().0;

        let arenas: Vec<Arena<Node<L>>> = (min..=max)
            .map(Level)
            .map(|level| {
                if level == Self::min_level() {
                    Arena::with_value(Node::Leaf(L::default()))
                } else {
                    let previous_id = NodeId {
                        level: level.decrement(),
                        index: 0,
                    };
                    let branch = Branch::from_children(Grid2::repeat(previous_id));
                    Arena::with_value(Node::Branch(branch))
                }
            })
            .collect();

        let root = NodeId {
            level: Self::max_level(),
            index: 0,
        };

        Self {
            arenas,
            root,
            rule,
            jump_cache: HashMap::new(),
        }
    }

    fn register(&mut self, node: Node<L>) -> NodeId {
        let level = node.level();
        let arena_index = self.arena_index(level);
        let index = self.arenas[arena_index].register(node);
        NodeId { level, index }
    }

    fn evolve(&mut self, branch: Branch, steps: u64) -> Result<NodeId> {
        if steps == branch.level.max_steps() {
            if let Some(&jump) = self.jump_cache.get(&branch) {
                return Ok(jump);
            }
        }

        let children: Grid2<Node<L>> = branch.children.try_map(|id| self.get_node(id))?;

        let node = match children.unpack() {
            [Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)] => {
                let leaves: Grid2<L> = Grid2::pack([nw, ne, sw, se]);
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

        if steps == branch.level.max_steps() {
            self.jump_cache.insert(branch, id);
        }

        Ok(id)
    }

    fn arena_index(&self, level: Level) -> usize {
        (level.0 - Self::min_level().0) as usize
    }

    fn get_node(&self, id: NodeId) -> Result<Node<L>> {
        self.arenas
            .get(self.arena_index(id.level))
            .and_then(|arena| arena.get(id.index))
            .ok_or(Error)
    }
}
