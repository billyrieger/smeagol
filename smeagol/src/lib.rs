#[macro_use]
extern crate packed_simd;

pub mod node;

use self::node::Quadrant;

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
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self {
            x: self.x + x_offset,
            y: self.y + y_offset,
        }
    }

    pub fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }
}
