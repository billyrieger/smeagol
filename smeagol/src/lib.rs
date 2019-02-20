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

pub mod node;

use self::node::{Level, NodeId, Quadrant, Store};

const INITIAL_LEVEL: Level = Level(7);

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
    fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self {
            x: self.x + x_offset,
            y: self.y + y_offset,
        }
    }

    /// Returns which quadrant of a node this position is in.
    fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }
}

/// Conway's Game of Life.
#[derive(Clone, Debug)]
pub struct Life {
    /// The root node of the Life grid.
    root: NodeId,
    /// The store.
    store: Store,
    /// What generation the Life is on.
    generation: u128,
}

impl Life {
    /// Creates a new empty Life grid.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut life = smeagol::Life::new();
    /// assert_eq!(life.population(), 0);
    /// ```
    pub fn new() -> Self {
        let mut store = Store::new();
        let root = store.create_empty(INITIAL_LEVEL);
        Self {
            root,
            store,
            generation: 0,
        }
    }

    /// Creates a Life grid from the given RLE file.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut life = smeagol::Life::from_rle_file("../assets/glider.rle").unwrap();
    /// assert_eq!(life.population(), 5);
    /// ```
    pub fn from_rle_file<P>(path: P) -> Result<Self, smeagol_rle::RleError>
    where
        P: AsRef<std::path::Path>,
    {
        let rle = smeagol_rle::Rle::from_file(path)?;
        Ok(Self::from_rle(&rle))
    }

    /// Creates a Life grid from the given RLE pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// // integral sign
    /// let mut life = smeagol::Life::from_rle_pattern(b"3b2o$2bobo$2bo2b$obo2b$2o!").unwrap();
    /// assert_eq!(life.population(), 9);
    /// ```
    pub fn from_rle_pattern(pattern: &[u8]) -> Result<Self, smeagol_rle::RleError> {
        let rle = smeagol_rle::Rle::from_pattern(pattern)?;
        Ok(Self::from_rle(&rle))
    }

    /// Creates a Life grid from the given RLE struct.
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

    /// Returns the number of alive cells in the grid.
    pub fn population(&self) -> u128 {
        self.root.population(&self.store)
    }

    /// Returns the current step size.
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
