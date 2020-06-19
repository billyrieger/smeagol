// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::{bit::BitSquare, grid::Grid2, memory::Arena};
use crate::Error;
use crate::Result;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Level {
    log_side_len: u8,
}

impl Level {
    const fn new(log_side_len: u8) -> Self {
        Self { log_side_len }
    }

    fn increment(&self) -> Level {
        Level::new(self.log_side_len + 1)
    }

    fn side_len(&self) -> u64 {
        1_u64 << self.log_side_len
    }

    fn min_coord(&self) -> i64 {
        let half = (self.side_len() / 2) as i64;
        -half
    }

    fn max_coord(&self) -> i64 {
        let half = (self.side_len() / 2) as i64;
        half - 1
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct NodeId {
    level: Level,
    index: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Node<B> {
    Leaf(Leaf<B>),
    Branch(Branch),
}

impl<B> Node<B>
where
    B: BitSquare,
{
    fn level(&self) -> Level {
        match self {
            Node::Leaf(_) => Level::new(B::LOG_SIDE_LEN),
            Node::Branch(branch) => branch.level,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Leaf<B> {
    pub alive: B,
}

impl<B> Leaf<B>
where
    B: BitSquare,
{
    fn new(alive: B) -> Self {
        Self { alive }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Branch {
    children: Grid2<NodeId>,
    population: u128,
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Dead,
    Alive,
}

pub struct Tree<B> {
    arenas: Vec<Arena<Node<B>>>,
    root: NodeId,
}

impl<B> Tree<B>
where
    B: BitSquare,
{
    fn min_level() -> Level {
        Level::new(B::LOG_SIDE_LEN)
    }

    fn max_level() -> Level {
        Level::new(63)
    }

    pub fn new() -> Self {
        let n_levels = (Self::max_level().log_side_len - B::LOG_SIDE_LEN) as usize;

        let _ = (0..n_levels).map(|i| {
            if i == 0 {
            } else {

            }
        });

        let mut arenas: Vec<Arena<Node<B>>> = std::iter::repeat_with(|| Arena::new())
            .take(n_levels)
            .collect();

        let mut prev_id = NodeId {
            level: Self::min_level(),
            index: arenas[0].register(Node::Leaf(Leaf::new(B::zero()))),
        };
        let mut current_level = Self::min_level().increment();

        let branch_arenas: Vec<Arena<Branch>> = std::iter::repeat_with(|| {
            let mut arena = Arena::new();

            let empty_branch = Branch {
                level: current_level,
                children: Grid2::repeat(prev_id),
                population: 0,
            };

            prev_id = NodeId {
                level: current_level,
                index: arena.register(empty_branch),
            };
            current_level = current_level.increment();

            arena
        })
        .take(n_levels)
        .collect();

        Self {
            arenas,
            root: prev_id,
        }
    }

    fn get_node(&self, id: NodeId) -> Result<Node<B>> {
        todo!()
        // match id {
        //     NodeId::Leaf { index } => {
        //         let leaf = self.leaf_arena.get(index).ok_or(Error)?;
        //         Ok(Node::Leaf(leaf))
        //     }
        //     NodeId::Branch { level, index } => {
        //         let arena_index = (level.log_side_len - B::LOG_SIDE_LEN - 1) as usize;
        //         let branch = self.branch_arenas[arena_index].get(index).ok_or(Error)?;
        //         Ok(Node::Branch(branch))
        //     }
        // }
    }
}

trait Visitor<B, T> {
    fn visit_leaf(&mut self, leaf: Leaf<B>) -> T;
    fn visit_branch(&mut self, branch: Branch) -> T;
}
