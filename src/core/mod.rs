// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Offset {
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
    const ORIGIN: Self = Self::new(0, 0);

    /// Creates a new `Position` from the given `x` and `y` coordinates.
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

    fn relative_to(&self, other: Position) -> Position {
        self.offset(Offset::Arbitrary {
            dx: -other.x,
            dy: -other.y,
        })
    }
}
