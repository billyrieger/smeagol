// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod bool8x8;
mod grid;

pub use bool8x8::Bool8x8;
pub use grid::{Grid2, Grid3, Grid4};

/// The four quadrants of a square grid.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

/// A delta from one `Position` to another `Position`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

/// A location.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub const ORIGIN: Self = Self::new(0, 0);

    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }

    pub fn offset(&self, offset: Offset) -> Position {
        let (dx, dy) = match offset {
            Offset::West(dx) => (-dx, 0),
            Offset::East(dx) => (dx, 0),
            Offset::North(dy) => (0, -dy),
            Offset::South(dy) => (0, dy),
            Offset::Northwest(delta) => (-delta, -delta),
            Offset::Northeast(delta) => (delta, -delta),
            Offset::Southwest(delta) => (-delta, delta),
            Offset::Southeast(delta) => (delta, delta),
            Offset::Arbitrary { dx, dy } => (dx, dy),
        };
        Self::new(self.x + dx, self.y + dy)
    }

    pub fn relative_to(&self, other: Position) -> Position {
        self.offset(Offset::Arbitrary {
            dx: -other.x,
            dy: -other.y,
        })
    }
}
