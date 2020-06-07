// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::core::{Bool8x8, Grid2, Offset, Position};

use std::{convert::TryFrom, fmt};

/// A description of how one state of a cellular automaton transitions into the next.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rule {
    birth_neighbors: [Bool8x8; 9],
    survival_neighbors: [Bool8x8; 9],
}

impl Rule {
    /// Creates a new Life rule using B/S notation.
    ///
    /// From [LifeWiki]:
    ///
    /// > The most common notation for rulestrings B{number list}/S{number list}; this is referred
    /// > to as "B/S notation", and is sometimes called the rulestring of the [cellular automaton]
    /// > in question. B (for birth) is a list of all the numbers of live neighbors that cause a
    /// > dead cell to come alive (be born); S (for survival) is a list of all the numbers of live
    /// > neighbors that cause a live cell to remain alive (survive).
    ///
    /// [LifeWiki]: https://www.conwaylife.com/wiki/Rulestring#B.2FS_notation
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Rule;
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// ```
    pub fn new(birth: &[usize], survival: &[usize]) -> Self {
        let make_rule = |neighbor_count: &[usize]| -> [Bool8x8; 9] {
            let mut result = [Bool8x8::FALSE; 9];
            for &i in neighbor_count.iter().filter(|&&count| count < 9) {
                result[i] = Bool8x8::TRUE;
            }
            result
        };

        Self {
            birth_neighbors: make_rule(birth),
            survival_neighbors: make_rule(survival),
        }
    }
}

/// The fundamental unit of a cellular automaton.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Dead,
    Alive,
}

impl Cell {
    pub fn is_alive(&self) -> bool {
        match self {
            Cell::Dead => false,
            Cell::Alive => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    alive: Bool8x8,
}

impl Leaf {
    fn pos_to_idx(pos: Position) -> u8 {
        u8::try_from(63 - 8 * (pos.y + 4) - (pos.x + 4)).unwrap()
    }

    fn idx_to_pos(idx: u8) -> Position {
        let idx = i64::from(idx);
        Position::new(3 - (idx % 8), 3 - (idx / 8))
    }

    pub fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    pub fn alive_cells(&self) -> Vec<Position> {
        (0..64)
            .filter_map(|i| {
                if self.alive.get_bit(i) {
                    Some(Self::idx_to_pos(i))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_cell(&self, x: i64, y: i64) -> Cell {
        let idx = Self::pos_to_idx(Position { x, y });
        if self.alive.get_bit(idx) {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }

    pub fn set_cell(&self, x: i64, y: i64, value: Cell) -> Self {
        let idx = Self::pos_to_idx(Position { x, y });
        match value {
            Cell::Dead => Self::new(self.alive.unset_bit(idx)),
            Cell::Alive => Self::new(self.alive.set_bit(idx)),
        }
    }

    fn step(&self, rule: Rule) -> Leaf {
        let alive = self.alive;
        let dead = !alive;

        let alive_neighbors: [Bool8x8; 9] = Bool8x8::sum(&[
            alive.shift(Offset::West(1)),
            alive.shift(Offset::East(1)),
            alive.shift(Offset::North(1)),
            alive.shift(Offset::South(1)),
            alive.shift(Offset::Northwest(1)),
            alive.shift(Offset::Northeast(1)),
            alive.shift(Offset::Southwest(1)),
            alive.shift(Offset::Southeast(1)),
        ]);

        let any_both = |xs: [Bool8x8; 9], ys: [Bool8x8; 9]| -> Bool8x8 {
            xs.iter()
                .zip(ys.iter())
                .map(|(&x, &y)| x & y)
                .fold(Bool8x8::FALSE, |x, y| x | y)
        };

        let born = any_both(alive_neighbors, rule.birth_neighbors);
        let survives = any_both(alive_neighbors, rule.survival_neighbors);

        Leaf::new((dead & born) | (alive & survives))
    }

    fn jump(&self, rule: Rule) -> Leaf {
        self.step(rule).step(rule)
    }

    fn join_horiz(west: Leaf, east: Leaf) -> Leaf {
        let west_half = west.alive.shift(Offset::West(4)) & Bool8x8::WEST;
        let east_half = east.alive.shift(Offset::East(4)) & Bool8x8::EAST;
        Self::new(west_half | east_half)
    }

    fn join_vert(north: Leaf, south: Leaf) -> Leaf {
        let north_half = north.alive.shift(Offset::North(4)) & Bool8x8::NORTH;
        let south_half = south.alive.shift(Offset::South(4)) & Bool8x8::SOUTH;
        Self::new(north_half | south_half)
    }

    fn join_centers(leaves: Grid2<Leaf>) -> Leaf {
        let [nw, ne, sw, se] = leaves.0;
        let combined = Bool8x8::FALSE
            | nw.alive.shift(Offset::Northwest(2)) & Bool8x8::NORTHWEST
            | ne.alive.shift(Offset::Northeast(2)) & Bool8x8::NORTHEAST
            | sw.alive.shift(Offset::Southwest(2)) & Bool8x8::SOUTHWEST
            | se.alive.shift(Offset::Southeast(2)) & Bool8x8::SOUTHEAST;
        Self::new(combined)
    }

    fn join_corners(leaves: Grid2<Leaf>) -> Leaf {
        let [nw, ne, sw, se] = leaves.0;
        let combined = Bool8x8::FALSE
            | nw.alive.shift(Offset::Northwest(4)) & Bool8x8::NORTHWEST
            | ne.alive.shift(Offset::Northeast(4)) & Bool8x8::NORTHEAST
            | sw.alive.shift(Offset::Southwest(4)) & Bool8x8::SOUTHWEST
            | se.alive.shift(Offset::Southeast(4)) & Bool8x8::SOUTHEAST;
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

impl fmt::Display for Leaf {
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
