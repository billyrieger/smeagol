// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    store::Id,
    util::{Bool8x8, Grid2, Offset, SumResult},
    Cell, Error, Position, Result, Rule,
};

use std::{convert::TryFrom, hash::Hash};

use either::Either;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(pub u8);

impl Level {
    const MAX_LEVEL: Self = Self(63);

    pub fn increment(&self) -> Result<Self> {
        if *self < Self::MAX_LEVEL {
            Ok(Self(self.0 + 1))
        } else {
            Err(Error::Increment)
        }
    }

    pub fn side_len(&self) -> u64 {
        1 << self.0
    }

    pub fn quadrant_centers(&self) -> Grid2<Position> {
        let delta = i64::try_from(self.side_len() / 4).unwrap();
        let nw_center = Position::new(-delta, -delta);
        let ne_center = Position::new(delta, -delta);
        let sw_center = Position::new(-delta, delta);
        let se_center = Position::new(delta, delta);
        Grid2([nw_center, ne_center, sw_center, se_center])
    }

    pub fn max_steps(&self) -> u64 {
        1u64 << (self.0 - 2)
    }

    pub fn min_coord(&self) -> i64 {
        -(1 << (self.0 - 1))
    }

    pub fn max_coord(&self) -> i64 {
        1 << (self.0 - 1) - 1
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: Bool8x8,
}

use std::fmt;

impl fmt::Debug for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for index in (0..64).rev() {
            write!(f, "{}", if self.alive.get_bit(index) { '#' } else { '.' })?;
            if index % 8 == 0 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Branch {
    pub children: Grid2<Id>,
    pub level: Level,
    pub population: u128,
}

impl Leaf {
    pub fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    pub fn dead() -> Self {
        Self::new(Bool8x8::FALSE)
    }

    pub fn alive() -> Self {
        Self::new(Bool8x8::TRUE)
    }

    pub fn alive_cells(&self) -> Vec<Position> {
        (0..64)
            .rev()
            .filter_map(|i| {
                if self.alive.get_bit(i) {
                    let i = i64::try_from(i).unwrap();
                    Some(Position::new(3 - (i % 8), 3 - (i / 8)))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_cell(&self, x: i64, y: i64) -> Cell {
        // (-4, -4) -> 63
        // (3, -4) -> 56
        // (-4, 3) -> 7
        // (3, 3) -> 0
        let index = usize::try_from(63 - 8 * (y + 4) - (x + 4)).unwrap();
        if self.alive.get_bit(index) {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }

    pub fn set_cell(&self, x: i64, y: i64, value: Cell) -> Self {
        let index = usize::try_from(63 - 8 * (y + 4) - (x + 4)).unwrap();
        match value {
            Cell::Dead => Self::new(self.alive.set_bit(index, false)),
            Cell::Alive => Self::new(self.alive.set_bit(index, true)),
        }
    }

    fn step(&self, rule: Rule) -> Leaf {
        let (alive, dead) = (self.alive, !self.alive);

        let alive_neighbors: SumResult = Bool8x8::sum(&[
            alive.offset(Offset::West(1)),
            alive.offset(Offset::East(1)),
            alive.offset(Offset::North(1)),
            alive.offset(Offset::South(1)),
            alive.offset(Offset::Northwest(1)),
            alive.offset(Offset::Northeast(1)),
            alive.offset(Offset::Southwest(1)),
            alive.offset(Offset::Southeast(1)),
        ]);

        let any_both = |xs: &SumResult, ys: &SumResult| -> Bool8x8 {
            xs.iter()
                .zip(ys.iter())
                .map(|(&x, &y)| x & y)
                .fold(Bool8x8::FALSE, std::ops::BitOr::bitor)
        };

        let born = any_both(&alive_neighbors, &rule.birth_neighbors);
        let survives = any_both(&alive_neighbors, &rule.survival_neighbors);

        Leaf::new((dead & born) | (alive & survives))
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

    pub fn evolve_leaves(leaves: Grid2<Leaf>, steps: u64, rule: Rule) -> Leaf {
        assert!(steps <= 4);

        let [northwest, northeast, southwest, southeast] = leaves.0;
        let north = Self::join_horiz(northwest, northeast);
        let south = Self::join_horiz(southwest, southeast);
        let west = Self::join_vert(northwest, southwest);
        let east = Self::join_vert(northeast, southeast);
        let center = Self::join_corners(leaves);

        let join_idle = |leaves: Grid2<Leaf>| -> Leaf { Leaf::join_centers(leaves) };

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

        let result = match steps {
            0 => join_idle(partial_leaves),
            1 => join_step(partial_leaves),
            2 | 3 | 4 => join_jump(partial_leaves),
            _ => unreachable!(),
        };

        result
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
    fn blinker() {
        let life = Rule::new(&[3], &[2, 3]);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x38 | . . # # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let blinker_horiz = Leaf::new(Bool8x8(0x0000_0038_0000_0000));

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x10 | . . . # . . . .
        // 0x10 | . . . # . . . .
        // 0x10 | . . . # . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let blinker_vert = Leaf::new(Bool8x8(0x0000_1010_1000_0000));

        assert_eq!(blinker_horiz.step(life), blinker_vert);
        assert_eq!(blinker_vert.step(life), blinker_horiz);
    }

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

        assert_eq!(Leaf::evolve_leaves(start, 0, life), idle_leaf);
        assert_eq!(Leaf::evolve_leaves(start, 4, life), jump_leaf);
    }

    #[test]
    fn alive_cells() {
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x10 | . . . # . . . .
        // 0x08 | . . . . # . . .
        // 0x38 | . . # # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let leaf = Leaf::new(Bool8x8(0x_00_00_10_08_38_00_00_00));
        let mut coords = leaf.alive_cells();
        coords.sort();
        assert_eq!(
            coords,
            [(-2, 0), (-1, -2), (-1, 0), (0, -1), (0, 0)]
                .iter()
                .map(|&(x, y)| Position::new(x, y))
                .collect::<Vec<_>>()
        );
    }
}
