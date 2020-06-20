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
    fn from_children(children: Grid2<NodeId>) -> Self {
        debug_assert_eq!(children.nw.level, children.ne.level);
        debug_assert_eq!(children.nw.level, children.sw.level);
        debug_assert_eq!(children.nw.level, children.se.level);

        let branch = Branch {
            level: children.nw.level.increment(),
            children,
        };
        Node::Branch(branch)
    }

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

impl Branch {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

impl Position {
    const ORIGIN: Position = Self::new(0, 0);

    const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }

    fn offset(&self, dx: i64, dy: i64) -> Position {
        Self::new(self.x + dx, self.y + dy)
    }

    fn relative_to(&self, other: Position) -> Position {
        self.offset(-other.x, -other.y)
    }
}

pub struct Tree<L, R> {
    arenas: Vec<Arena<Node<L>>>,
    root: NodeId,
    rule: R,
    cache: HashMap<Grid2<NodeId>, NodeId>,
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
                    let children = Grid2::repeat(previous_id);
                    Arena::with_value(Node::from_children(children))
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
            cache: HashMap::new(),
        }
    }

    fn register(&mut self, node: Node<L>) -> NodeId {
        let level = node.level();
        let arena_index = self.arena_index(level);
        let index = self.arenas[arena_index].register(node);
        NodeId { level, index }
    }

    fn classify(&self, ids: Grid2<NodeId>) -> Result<Children<L>> {
        let nodes: Grid2<Node<L>> = ids.try_map(|id| self.get_node(id))?;
        match nodes.unpack() {
            [Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)] => {
                Ok(Children::Leaves(Grid2::pack([nw, ne, sw, se])))
            }
            [Node::Branch(nw), Node::Branch(ne), Node::Branch(sw), Node::Branch(se)] => {
                Ok(Children::Branches(Grid2::pack([nw, ne, sw, se])))
            }
            _ => Err(Error),
        }
    }

    fn evolve(&mut self, grid: Grid2<NodeId>, steps: u64) -> Result<NodeId> {
        if let Some(&evolution) = self.cache.get(&grid) {
            return Ok(evolution);
        }

        match self.classify(grid)? {
            Children::Leaves(leaves) => {
                let new_leaf = self.rule.evolve(leaves, steps);
                let id = self.register(Node::Leaf(new_leaf));
                Ok(id)
            }
            Children::Branches(branches) => {
                let grandchildren: Grid4<NodeId> = branches.map(|branch| branch.children).flatten();

                let partial: Grid3<NodeId> = grandchildren.shrink(|ids| self.evolve(ids, steps))?;
                let complete: Grid2<NodeId> = partial.shrink(|ids| self.evolve(ids, steps))?;
                todo!()
            }
        }
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

enum Children<L> {
    Leaves(Grid2<L>),
    Branches(Grid2<Branch>),
}
