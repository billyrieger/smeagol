/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! A library to simulate Conway's Game of Life.
//!
//! # Examples
//!
//! ```
//! // create a gosper glider gun
//! let mut life = smeagol::Life::from_rle_pattern(
//!     b"
//! 24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8b
//! o3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!",
//! )
//! .unwrap();
//!
//! // step 1024 generations into the future
//! life.set_step_log_2(10);
//! life.step();
//! ```
#[macro_use]
extern crate packed_simd;

mod life;
pub mod node;

pub use self::life::Life;
use self::node::Quadrant;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    Rle(smeagol_rle::RleError),
}

impl From<std::io::Error> for Error {
    fn from(io: std::io::Error) -> Error {
        Error {
            kind: ErrorKind::Io(io),
        }
    }
}

impl From<smeagol_rle::RleError> for Error {
    fn from(rle: smeagol_rle::RleError) -> Error {
        Error {
            kind: ErrorKind::Rle(rle),
        }
    }
}

/// A cell in a Life grid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    /// An alive cell.
    Alive,
    /// A dead cell.
    Dead,
}

impl Cell {
    /// Creates a new `Cell`.
    ///
    /// # Examples
    ///
    /// ```
    /// let alive = smeagol::Cell::new(true);
    /// let dead = smeagol::Cell::new(false);
    /// ```
    pub fn new(alive: bool) -> Self {
        if alive {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }

    /// Returns true for `Cell::Alive` and false for `Cell::Dead`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(smeagol::Cell::Alive.is_alive());
    /// assert!(!smeagol::Cell::Dead.is_alive());
    /// ```
    pub fn is_alive(self) -> bool {
        match self {
            Cell::Alive => true,
            Cell::Dead => false,
        }
    }
}

/// The position of a cell in a Life grid.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    /// The x coordinate.
    pub x: i64,
    /// The y coordinate.
    pub y: i64,
}

impl Position {
    /// Creates a new position with the given coordinates.
    ///
    /// # Exampes
    ///
    /// ```
    /// let position = smeagol::Position::new(1, 2);
    /// ```
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    /// Offsets the position by the given amounts in the x and y directions.
    pub fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self {
            x: self.x + x_offset,
            y: self.y + y_offset,
        }
    }

    /// Returns which quadrant of a node this position is in.
    pub fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundingBox {
    upper_left: Position,
    lower_right: Position,
}

impl BoundingBox {
    pub fn new(upper_left: Position, lower_right: Position) -> Self {
        assert!(upper_left.x <= lower_right.x);
        assert!(upper_left.y <= lower_right.y);
        Self {
            upper_left,
            lower_right,
        }
    }

    pub fn combine(&self, other: BoundingBox) -> Self {
        let min_x = Ord::min(self.upper_left.x, other.upper_left.x);
        let min_y = Ord::min(self.upper_left.y, other.upper_left.y);
        let max_x = Ord::max(self.lower_right.x, other.lower_right.x);
        let max_y = Ord::max(self.lower_right.y, other.lower_right.y);

        Self::new(Position::new(min_x, min_y), Position::new(max_x, max_y))
    }

    pub fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self::new(
            self.upper_left.offset(x_offset, y_offset),
            self.lower_right.offset(x_offset, y_offset),
        )
    }

    pub fn pad(&self, amount: i64) -> Self {
        assert!(amount > 0);
        Self {
            upper_left: self.upper_left.offset(-amount, -amount),
            lower_right: self.lower_right.offset(amount, amount),
        }
    }
}
