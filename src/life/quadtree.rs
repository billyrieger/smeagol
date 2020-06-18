// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::{
    bit::BitSquare,
    grid::Grid2,
    memory::{Arena, Id},
};

struct NodeId {
    id: Id,
    level: Level,
}

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

    fn position_to_index(pos: Position) -> u32 {
        let side_len = Level::new(B::LOG_SIDE_LEN).side_len();
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Branch {
    children: Grid2<Id>,
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
    leaves: Arena<Leaf<B>>,
    branches: Vec<Arena<Branch>>,
    root: Id,
}

impl<B> Tree<B>
where
    B: BitSquare,
{
    pub fn new() -> Self {
        let mut leaves: Arena<Leaf<B>> = Arena::new();
        let mut branches: Vec<Arena<Branch>> =
            std::iter::repeat_with(|| Arena::new()).take(2).collect();

        let mut prev_id = leaves.register(Leaf::new(B::zero()));
        let mut current_level = Level::new(B::LOG_SIDE_LEN).increment();

        for arena in branches.iter_mut() {
            let empty_branch = Branch {
                level: current_level,
                children: Grid2::repeat(prev_id),
                population: 0,
            };

            prev_id = arena.register(empty_branch);
            current_level = current_level.increment();
        }

        Self {
            leaves,
            branches,
            root: prev_id,
        }
    }

    fn visit<T, V>(&mut self, level: Level, node: Id, visitor: V)
    where
        V: Visitor<B, T>,
    {
        todo!()
    }
}

trait Visitor<B, T> {
    fn visit_leaf(&mut self, leaf: Leaf<B>) -> T;
    fn visit_branch(&mut self, branch: Branch, results: Grid2<T>) -> T;
}

pub struct AliveCells<'t, B> {
    tree: &'t Tree<B>,
}
