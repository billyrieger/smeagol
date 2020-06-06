// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(dead_code, unused_variables)]

pub mod core;
pub mod life;

mod node;
mod rle;
mod store;
mod util;

use store::{Id, Store};
use util::{Bool8x8, SumResult};

use std::{fmt, io};

use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Position {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

pub enum Offset {
    West(i64),
    East(i64),
    North(i64),
    South(i64),
    Northwest(i64),
    Northeast(i64),
    Southwest(i64),
    Southeast(i64),
    Arbitrary { dx: i64, dy: i64 },
}

impl Position {
    pub const ORIGIN: Self = Self::new(0, 0);

    /// Creates a new `Position` from the given `x` and `y` coordinates.
    pub const fn new(x: i64, y: i64) -> Self {
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

    fn relative_to(&self, other: Position) -> Position {
        self.offset(Offset::Arbitrary {
            dx: -other.x,
            dy: -other.y,
        })
    }

    fn offset(&self, offset: Offset) -> Position {
        match offset {
            Offset::West(dx) => Self::new(self.x - dx, self.y),
            Offset::East(dx) => Self::new(self.x + dx, self.y),
            Offset::North(dy) => Self::new(self.x, self.y - dy),
            Offset::South(dy) => Self::new(self.x, self.y + dy),
            Offset::Northwest(delta) => Self::new(self.x - delta, self.y - delta),
            Offset::Northeast(delta) => Self::new(self.x + delta, self.y - delta),
            Offset::Southwest(delta) => Self::new(self.x - delta, self.y + delta),
            Offset::Southeast(delta) => Self::new(self.x + delta, self.y + delta),
            Offset::Arbitrary { dx, dy } => Self::new(self.x + dx, self.y + dy),
        }
    }
}

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

/// An error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("parse")]
    RleParse,
    #[error("increment")]
    Increment,
    #[error("unbalanced")]
    Unbalanced,
    #[error("id not found")]
    IdNotFound(Id),
    #[error("out of bounds")]
    OutOfBounds,
    #[error("fmt {0}")]
    Fmt(#[from] fmt::Error),
    #[error("io {0}")]
    Io(#[from] io::Error),
}

/// The fundamental unit of a cellular automaton.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Dead,
    Alive,
}

impl Cell {
    /// Checks whether the cell is alive or not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::Cell;
    /// assert!(Cell::Alive.is_alive());
    /// assert!(!Cell::Dead.is_alive());
    /// ```
    pub fn is_alive(&self) -> bool {
        match self {
            Cell::Dead => false,
            Cell::Alive => true,
        }
    }
}

/// A description of how one state of a cellular automaton transitions into the next.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rule {
    birth_neighbors: SumResult,
    survival_neighbors: SumResult,
}

impl Rule {
    /// Creates a new `Rule` using B/S notation.
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
        let make_rule = |neighbor_count: &[usize]| -> SumResult {
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

    pub fn b3s23() -> Self {
        Self::new(&[3], &[2, 3])
    }
}

pub struct Universe {
    store: Store,
    root: Id,
}

impl Universe {
    pub fn new() -> Result<Self> {
        Self::empty(Rule::b3s23())
    }

    /// Creates new universe without any alive cells.
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::{Cell, Result, Rule, Universe};
    /// # fn main() -> Result<()> {
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// let empty = Universe::empty(life)?;
    /// assert_eq!(empty.population()?, 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn empty(rule: Rule) -> Result<Self> {
        let mut store = Store::new(rule);
        let root = store.initialize()?;
        Ok(Self { store, root })
    }

    pub fn from_rle_pattern(rule: Rule, pattern: &str) -> Result<Self> {
        use rle::Pattern;

        let mut universe = Self::empty(rule)?;
        let cells: Vec<Position> = Pattern::from_pattern(pattern)?.alive_cells().collect();
        println!("{:?}", cells);
        universe.root = universe
            .store
            .set_cells(universe.root, cells, Cell::Alive)?;
        Ok(universe)
    }

    pub fn population(&self) -> Result<u128> {
        self.store.population(self.root)
    }

    /// # Examples
    ///
    /// ```
    /// # use smeagol::{Cell, Result, Rule, Universe};
    /// # fn main() -> Result<()> {
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// let glider = Universe::from_rle_pattern(life, "bob$2bo$3o!")?;
    /// assert_eq!(glider.get_cell(0, 0)?, Cell::Dead);
    /// assert_eq!(glider.get_cell(1, 0)?, Cell::Alive);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_cell(&self, x: i64, y: i64) -> Result<Cell> {
        self.store.get_cell(self.root, Position::new(x, y))
    }

    /// # Examples
    ///
    /// ```
    /// # use smeagol::{Cell, Result, Rule, Universe};
    /// # fn main() -> Result<()> {
    /// #
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    ///
    /// let mut glider = Universe::from_rle_pattern(life, "bob$2bo$3o!")?;
    ///
    /// assert!(glider.get_cell(2, 2)?.is_alive());
    ///
    /// glider.step(4)?;
    /// assert!(glider.get_cell(3, 3)?.is_alive());
    ///
    /// glider.step(4)?;
    /// assert!(glider.get_cell(4, 4)?.is_alive());
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn step(&mut self, steps: u64) -> Result<()> {
        self.root = self.store.step(self.root, steps)?;
        Ok(())
    }
}
