/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! A library to efficiently simulate Conway's Game of Life using the HashLife algorithm.
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), failure::Error> {
//! // create a gosper glider gun
//! let mut life = smeagol::Life::from_rle_pattern(
//!     b"
//! 24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8b
//! o3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!",
//! )?;
//!
//! // step 1024 generations into the future
//! life.set_step_log_2(10);
//! life.step();
//!
//! // save the result
//! life.save_png(
//!     std::env::temp_dir().join("gosper_glider_gun.png"),
//!     life.bounding_box().unwrap().pad(10),
//!     0,
//! )?;
//! # Ok(())
//! # }
//! ```
#[macro_use]
extern crate failure;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate packed_simd;

mod life;
pub mod node;
pub mod parse;

pub use crate::life::Life;
use crate::{node::Quadrant, parse::rle::RleError};

/// An error that can occur.
#[derive(Debug, Fail)]
pub enum Error {
    /// An IO error.
    #[fail(display = "IO error: {}", io)]
    Io { io: std::io::Error },
    #[fail(display = "RLE pattern error: {}", rle)]
    /// An RLE error.
    Rle { rle: RleError },
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
    ///
    /// # Examples
    ///
    /// ```
    /// let pos = smeagol::Position::new(1, 2);
    /// assert_eq!(pos.offset(3, 4), smeagol::Position::new(4, 6));
    /// ```
    pub fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self {
            x: self.x + x_offset,
            y: self.y + y_offset,
        }
    }

    /// Returns which quadrant of a node the position is in.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     smeagol::Position::new(-1, -1).quadrant(),
    ///     smeagol::node::Quadrant::Northwest
    /// );
    /// assert_eq!(
    ///     smeagol::Position::new(-1, 0).quadrant(),
    ///     smeagol::node::Quadrant::Southwest
    /// );
    /// assert_eq!(
    ///     smeagol::Position::new(0, -1).quadrant(),
    ///     smeagol::node::Quadrant::Northeast
    /// );
    /// assert_eq!(
    ///     smeagol::Position::new(0, 0).quadrant(),
    ///     smeagol::node::Quadrant::Southeast
    /// );
    /// ```
    pub fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }
}

/// A rectangular region of a Life grid.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundingBox {
    upper_left: Position,
    lower_right: Position,
}

impl BoundingBox {
    /// Creates a new bounding box with the given upper-left corner position and lower-right corner
    /// position.
    ///
    /// # Examples
    ///
    /// ```
    /// // create a bounding box around a single position
    /// let pos = smeagol::Position::new(0, 0);
    /// let bounding_box = smeagol::BoundingBox::new(pos, pos);
    /// ```
    pub fn new(upper_left: Position, lower_right: Position) -> Self {
        assert!(upper_left.x <= lower_right.x);
        assert!(upper_left.y <= lower_right.y);
        Self {
            upper_left,
            lower_right,
        }
    }

    /// Combines two bounding boxes, returning a bounding box that surrounds both boxes.
    ///
    /// # Examples
    ///
    /// ```
    /// let p0 = smeagol::Position::new(0, 0);
    /// let p1 = smeagol::Position::new(1, 1);
    ///
    /// let bbox0 = smeagol::BoundingBox::new(p0, p0);
    /// let bbox1 = smeagol::BoundingBox::new(p1, p1);
    ///
    /// assert_eq!(bbox0.combine(bbox1), smeagol::BoundingBox::new(p0, p1));
    /// ```
    pub fn combine(&self, other: BoundingBox) -> Self {
        let min_x = Ord::min(self.upper_left.x, other.upper_left.x);
        let min_y = Ord::min(self.upper_left.y, other.upper_left.y);
        let max_x = Ord::max(self.lower_right.x, other.lower_right.x);
        let max_y = Ord::max(self.lower_right.y, other.lower_right.y);

        Self::new(Position::new(min_x, min_y), Position::new(max_x, max_y))
    }

    /// Offsets the bounding box by the given amounts in the x and y directions.
    ///
    /// # Examples
    ///
    /// ```
    /// let p0 = smeagol::Position::new(0, 0);
    /// let p1 = smeagol::Position::new(2, 2);
    /// let bbox = smeagol::BoundingBox::new(p0, p1);
    /// let offset_bbox = bbox.offset(3, 4);
    /// ```
    pub fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self::new(
            self.upper_left.offset(x_offset, y_offset),
            self.lower_right.offset(x_offset, y_offset),
        )
    }

    /// Pads the outside of the bounding box by the given amount.
    ///
    /// # Panics
    ///
    /// Panics if `amount < 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// let p0 = smeagol::Position::new(0, 0);
    /// let p1 = smeagol::Position::new(2, 2);
    /// let bbox = smeagol::BoundingBox::new(p0, p1);
    /// let padded_bbox = bbox.pad(5);
    /// ```
    pub fn pad(&self, amount: i64) -> Self {
        assert!(amount >= 0);
        Self {
            upper_left: self.upper_left.offset(-amount, -amount),
            lower_right: self.lower_right.offset(amount, amount),
        }
    }
}
