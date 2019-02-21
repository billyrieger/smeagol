//! Life module.

mod render;

use crate::{
    node::{Level, NodeId, Store},
    BoundingBox, Position,
};

const INITIAL_LEVEL: Level = Level(7);

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

    pub fn set_cell_alive(&mut self, position: Position) {
        while position.x < self.root.min_coord(&self.store)
            || position.y < self.root.min_coord(&self.store)
            || position.x > self.root.max_coord(&self.store)
            || position.y > self.root.max_coord(&self.store)
        {
            self.root = self.root.expand(&mut self.store);
        }
        self.root = self.root.set_cell_alive(&mut self.store, position);
    }

    pub fn get_alive_cells(&self) -> Vec<Position> {
        self.root.get_alive_cells(&self.store)
    }

    pub fn contains_alive_cells(&self, bounding_box: BoundingBox) -> bool {
        self.root.contains_alive_cells(
            &self.store,
            bounding_box.upper_left,
            bounding_box.lower_right,
        )
    }

    pub fn bounding_box(&self) -> Option<BoundingBox> {
        self.root.bounding_box(&self.store)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_extremes() {
        let mut life = Life::new();

        let min = i64::min_value();
        let max = i64::max_value();

        life.set_cell_alive(Position::new(min, min));
        life.set_cell_alive(Position::new(min, max));
        life.set_cell_alive(Position::new(max, min));
        life.set_cell_alive(Position::new(max, max));

        assert_eq!(
            life.bounding_box(),
            Some(BoundingBox::new(
                Position::new(min, min),
                Position::new(max, max)
            ))
        );
    }
}
