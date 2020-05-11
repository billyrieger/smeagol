// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    util::{Bool8x8, Grid2, Grid4, Offset},
    Error, Result, Rule,
};

use std::{collections::HashMap, hash::Hash};

use either::Either;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: Bool8x8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Branch {
    pub children: Grid2<Id>,
    pub level: Level,
    pub population: u128,
}

impl Leaf {
    fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    fn dead() -> Self {
        Self::new(Bool8x8::FALSE)
    }

    fn alive() -> Self {
        Self::new(Bool8x8::TRUE)
    }

    fn step(&self, rule: Rule) -> Leaf {
        Self::new(rule.step(self.alive))
    }

    fn jump(&self, rule: Rule) -> Leaf {
        self.step(rule).step(rule)
    }

    fn join_horiz(west: Leaf, east: Leaf) -> Leaf {
        let combined = Bool8x8::FALSE
            | west.alive.offset(Offset::West(4)) & Bool8x8::WEST
            | east.alive.offset(Offset::East(4)) & Bool8x8::EAST;
        Self::new(combined)
    }

    fn join_vert(north: Leaf, south: Leaf) -> Leaf {
        let combined = Bool8x8::FALSE
            | north.alive.offset(Offset::North(4)) & Bool8x8::NORTH
            | south.alive.offset(Offset::South(4)) & Bool8x8::SOUTH;
        Self::new(combined)
    }

    fn join_centers(leaves: Grid2<Leaf>) -> Leaf {
        let [nw, ne, sw, se] = leaves.0;
        let combined = Bool8x8::FALSE
            | nw.alive.offset(Offset::Northwest(2)) & Bool8x8::NORTHWEST
            | ne.alive.offset(Offset::Northeast(2)) & Bool8x8::NORTHEAST
            | sw.alive.offset(Offset::Southwest(2)) & Bool8x8::SOUTHWEST
            | se.alive.offset(Offset::Southeast(2)) & Bool8x8::SOUTHEAST;
        Self::new(combined)
    }

    fn join_corners(leaves: Grid2<Leaf>) -> Leaf {
        let [nw, ne, sw, se] = leaves.0;
        let combined = Bool8x8::FALSE
            | nw.alive.offset(Offset::Northwest(4)) & Bool8x8::NORTHWEST
            | ne.alive.offset(Offset::Northeast(4)) & Bool8x8::NORTHEAST
            | sw.alive.offset(Offset::Southwest(4)) & Bool8x8::SOUTHWEST
            | se.alive.offset(Offset::Southeast(4)) & Bool8x8::SOUTHEAST;
        Self::new(combined)
    }

    fn evolve_leaves(leaves: Grid2<Leaf>, steps: u64, rule: Rule) -> Leaf {
        assert!(steps <= 4);

        let [northwest, northeast, southwest, southeast] = leaves.0;
        let north = Self::join_horiz(northwest, northeast);
        let south = Self::join_horiz(southwest, southeast);
        let west = Self::join_vert(northwest, southwest);
        let east = Self::join_vert(northeast, southeast);
        let center = Self::join_corners(leaves);

        let join_idle = |leaves: Grid2<Leaf>| -> Leaf { Leaf::join_corners(leaves) };

        let join_step = |leaves: Grid2<Leaf>| -> Leaf {
            let [nw, ne, sw, se] = leaves.0;
            let new_leaves = Grid2([nw.step(rule), ne.step(rule), sw.step(rule), se.step(rule)]);
            Leaf::join_centers(new_leaves)
        };

        let join_jump = |leaves: Grid2<Leaf>| -> Leaf {
            let [nw, ne, sw, se] = leaves.0;
            let new_leaves = Grid2([nw.jump(rule), ne.jump(rule), sw.jump(rule), se.jump(rule)]);
            Leaf::join_centers(new_leaves)
        };

        let make_partial = |leaves: Grid2<Leaf>| -> Leaf {
            match steps {
                0 | 1 | 2 => join_idle(leaves),
                3 => join_step(leaves),
                4 => join_jump(leaves),
                _ => unreachable!(),
            }
        };

        let partial_nw = make_partial(Grid2([northwest, north, west, center]));
        let partial_ne = make_partial(Grid2([north, northeast, center, east]));
        let partial_sw = make_partial(Grid2([west, center, southwest, south]));
        let partial_se = make_partial(Grid2([center, east, south, southeast]));

        let partial_leaves = Grid2([partial_nw, partial_ne, partial_sw, partial_se]);

        match steps {
            0 => join_idle(partial_leaves),
            1 => join_step(partial_leaves),
            2 | 3 | 4 => join_jump(partial_leaves),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Data {
    node: Node,
    idle: Option<Id>,
    jump: Option<Id>,
}

#[derive(Clone, Default)]
pub struct Store {
    rule: Rule,
    id_lookup: HashMap<Node, Id>,
    node_data: Vec<Data>,
}

impl Store {
    pub fn new(rule: Rule) -> Self {
        Self {
            rule,
            id_lookup: HashMap::new(),
            node_data: Vec::new(),
        }
    }

    fn get_data(&self, id: Id) -> Result<&Data> {
        self.node_data.get(id.index()).ok_or(Error::IdNotFound(id))
    }

    fn get_data_mut(&mut self, id: Id) -> Result<&mut Data> {
        self.node_data
            .get_mut(id.index())
            .ok_or(Error::IdNotFound(id))
    }

    fn make_id(&mut self, node: Node) -> Id {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let data = Data {
                node,
                idle: None,
                jump: None,
            };
            let new_id = Id::new(self.node_data.len());
            self.node_data.push(data);
            self.id_lookup.insert(node, new_id);
            new_id
        })
    }

    fn make_branch(&mut self, children: Grid2<Id>) -> Result<Branch> {
        let data = children.try_map(|id| self.get_data(id))?;
        let level = data.0[0].node.level().increment()?;
        let population = data.0.iter().map(|data| data.node.population()).sum();
        Ok(Branch {
            children,
            level,
            population,
        })
    }

    fn step(&mut self, id: Id, step: u64) -> Result<Id> {
        match self.get_data(id)?.node {
            Node::Leaf(_) => todo!(),
            Node::Branch(branch) => self.evolve(branch, step),
        }
    }

    fn evolve(&mut self, branch: Branch, steps: u64) -> Result<Id> {
        let rule = self.rule;

        let max_steps = branch.level.max_steps();
        let half_max = max_steps / 2;
        assert!(steps <= max_steps, format!("step too large"));

        let children: Either<Grid2<Leaf>, Grid2<Branch>> = branch
            .children
            .try_map(|id| self.get_data(id))?
            .map(|data| data.node)
            .classify()?;

        let result: Node = match children {
            Either::Left(leaf_grid) => Node::Leaf(Leaf::evolve_leaves(leaf_grid, steps, rule)),

            Either::Right(branch_grid) => {
                let grandchildren: Grid4<Id> = branch_grid.map(|branch| branch.children).flatten();

                let (first_step, second_step) = if steps < half_max {
                    (0, steps)
                } else {
                    (steps - half_max, half_max)
                };

                let new_children: Grid2<Id> = grandchildren
                    .shrink(|ids: Grid2<Id>| -> Result<Id> {
                        let branch = self.make_branch(ids)?;
                        self.evolve(branch, first_step)
                    })?
                    .shrink(|ids: Grid2<Id>| -> Result<Id> {
                        let branch = self.make_branch(ids)?;
                        self.evolve(branch, second_step)
                    })?;

                Node::Branch(self.make_branch(new_children)?)
            }
        };

        Ok(self.make_id(result))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

impl Level {
    const MAX_LEVEL: Self = Self(63);

    fn increment(self) -> Result<Self> {
        if self < Self::MAX_LEVEL {
            Ok(Self(self.0 + 1))
        } else {
            Err(Error::Increment)
        }
    }

    pub fn max_steps(&self) -> u64 {
        1u64 << (self.0 - 2)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id {
    index: usize,
}

impl Id {
    fn new(index: usize) -> Self {
        Self { index }
    }

    fn index(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn level(&self) -> Level {
        match self {
            Self::Leaf(_) => Level(3),
            Self::Branch(branch) => branch.level,
        }
    }

    pub fn population(&self) -> u128 {
        match self {
            Self::Leaf(leaf) => u128::from(leaf.alive.0.count_ones()),
            Self::Branch(branch) => branch.population,
        }
    }
}

impl Grid2<Node> {
    pub fn classify(&self) -> Result<Either<Grid2<Leaf>, Grid2<Branch>>> {
        match *self {
            Grid2([Node::Leaf(a), Node::Leaf(b), Node::Leaf(c), Node::Leaf(d)]) => {
                Ok(Either::Left(Grid2([a, b, c, d])))
            }

            Grid2([Node::Branch(a), Node::Branch(b), Node::Branch(c), Node::Branch(d)]) => {
                Ok(Either::Right(Grid2([a, b, c, d])))
            }

            _ => Err(Error::Unbalanced),
        }
    }
}

impl Grid2<Leaf> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glider() {
        let life = Rule::new(&[3], &[2, 3]);

        //      +-----------------+-----------------+
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x01 | . . . . . . . # | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | # . . . . . . . | 0x80
        //      +-----------------+-----------------+
        // 0x03 | . . . . . . # # | # . . . . . . . | 0x80
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        //      +-----------------+-----------------+
        let nw_start = Leaf::new(Bool8x8(0x_00_00_00_00_00_00_01_00));
        let ne_start = Leaf::new(Bool8x8(0x_00_00_00_00_00_00_00_80));
        let sw_start = Leaf::new(Bool8x8(0x_03_00_00_00_00_00_00_00));
        let se_start = Leaf::new(Bool8x8(0x_80_00_00_00_00_00_00_00));
        let start = Grid2([nw_start, ne_start, sw_start, se_start]);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x10 | . . . # . . . .
        // 0x08 | . . . . # . . .
        // 0x38 | . . # # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let idle_leaf = Leaf::new(Bool8x8(0x_00_00_10_08_38_00_00_00));
        assert_eq!(idle_leaf, Leaf::join_corners(start));

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x08 | . . . . # . . .
        // 0x04 | . . . . . # . .
        // 0x1C | . . . # # # . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let jump_leaf = Leaf::new(Bool8x8(0x_00_00_00_08_04_1C_00_00));
        assert_eq!(
            idle_leaf.alive.offset(Offset::Southeast(1)),
            jump_leaf.alive
        );
    }
}
