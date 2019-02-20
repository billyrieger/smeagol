#[macro_use]
extern crate packed_simd;

pub mod node;

const INITIAL_LEVEL: Level = Level(7);

use self::node::Quadrant;
use self::node::{Store, NodeId, Level};

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

pub struct Life {
    root: NodeId,
    store: Store,
    generation: u128,
}

impl Life {
    pub fn new() -> Self {
        let mut store = Store::new();
        let root = store.create_empty(INITIAL_LEVEL);
        Self {
            root,
            store,
            generation: 0,
        }
    }

    pub fn from_rle_file<P>(path: P) -> Result<Self, smeagol_rle::RleError>
    where
        P: AsRef<std::path::Path>,
    {
        let rle = smeagol_rle::Rle::from_file(path)?;
        Ok(Self::from_rle(&rle))
    }

    pub fn from_rle_pattern(pattern: &[u8]) -> Result<Self, smeagol_rle::RleError> {
        let rle = smeagol_rle::Rle::from_pattern(pattern)?;
        Ok(Self::from_rle(&rle))
    }

    fn from_rle(rle: &smeagol_rle::Rle) -> Self {
        let alive_cells = rle
            .alive_cells()
            .into_iter()
            .map(|(x, y)| Position::new(i64::from(x), i64::from(y)))
            .collect::<Vec<_>>();

        let mut store = Store::new();
        let mut root = store.create_empty(INITIAL_LEVEL);

        if !alive_cells.is_empty() {
            let x_min = alive_cells.iter().min_by_key(|pos| pos.x).unwrap().x;
            let x_max = alive_cells.iter().max_by_key(|pos| pos.x).unwrap().x;
            let y_min = alive_cells.iter().min_by_key(|pos| pos.y).unwrap().y;
            let y_max = alive_cells.iter().max_by_key(|pos| pos.y).unwrap().y;

            while x_min < root.min_coord(&store)
                || x_max > root.max_coord(&store)
                || y_min < root.min_coord(&store)
                || y_max > root.max_coord(&store)
            {
                root = root.expand(&mut store);
            }

            root = root.set_cells_alive(&mut store, alive_cells);
        }

        Self {
            root,
            store,
            generation: 0,
        }
    }

    pub fn get_alive_cells(&self) -> Vec<Position> {
        self.root.get_alive_cells(&self.store)
    }

    pub fn generation(&self) -> u128 {
        self.generation
    }

    pub fn step_size(&self) -> u64 {
        1 << self.store.step_log_2()
    }

    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        self.store.set_step_log_2(step_log_2);
    }

    fn pad(&mut self) {
        while self.root.level(&self.store) < INITIAL_LEVEL
            || self.store.step_log_2() > self.root.level(&self.store).0 - 2
            || self.root.ne(&self.store).population(&self.store)
                != self
                    .root
                    .ne(&self.store)
                    .sw(&self.store)
                    .sw(&self.store)
                    .population(&self.store)
            || self.root.nw(&self.store).population(&self.store)
                != self
                    .root
                    .nw(&self.store)
                    .se(&self.store)
                    .se(&self.store)
                    .population(&self.store)
            || self.root.se(&self.store).population(&self.store)
                != self
                    .root
                    .se(&self.store)
                    .nw(&self.store)
                    .nw(&self.store)
                    .population(&self.store)
            || self.root.sw(&self.store).population(&self.store)
                != self
                    .root
                    .sw(&self.store)
                    .ne(&self.store)
                    .ne(&self.store)
                    .population(&self.store)
        {
            self.root = self.root.expand(&mut self.store);
        }
    }

    pub fn step(&mut self) {
        self.pad();
        self.root = self.root.step(&mut self.store);
        self.generation += u128::from(self.step_size());
    }
}

impl Default for Life {
    fn default() -> Self {
        Self::new()
    }
}
