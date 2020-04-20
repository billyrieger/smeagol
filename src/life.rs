/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

mod render;

use crate::{
    node::{Level, NodeId, Store},
    parse::rle::Rle,
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
    /// How many generations the Life grid has been advanced.
    generation: u128,
    /// A bounding box containing all alive cells.
    bounding_box: Option<BoundingBox>,
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
            bounding_box: None,
        }
    }

    /// Creates a Life grid from the given RLE file.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut life = smeagol::Life::from_rle_file("./assets/glider.rle").unwrap();
    /// assert_eq!(life.population(), 5);
    /// ```
    pub fn from_rle_file<P>(path: P) -> Result<Self, failure::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let rle = Rle::from_file(path)?;
        Ok(Self::from_rle(&rle))
    }

    pub fn from_rle_file_contents(contents: &[u8]) -> Result<Self, failure::Error> {
        let rle = Rle::from_file_contents(contents)?;
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
    pub fn from_rle_pattern(pattern: &[u8]) -> Result<Self, failure::Error> {
        let rle = Rle::from_pattern(pattern)?;
        Ok(Self::from_rle(&rle))
    }

    /// Creates a Life grid from the given RLE struct.
    pub fn from_rle(rle: &Rle) -> Self {
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
            bounding_box: root.bounding_box(&store),
            root,
            store,
            generation: 0,
        }
    }

    /// Sets the cell at the given position in the Life grid to be an alive cell.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut life = smeagol::Life::new();
    ///
    /// // create a block
    /// life.set_cell_alive(smeagol::Position::new(0, 0));
    /// life.set_cell_alive(smeagol::Position::new(1, 0));
    /// life.set_cell_alive(smeagol::Position::new(0, 1));
    /// life.set_cell_alive(smeagol::Position::new(1, 1));
    ///
    /// assert_eq!(life.population(), 4);
    /// ```
    pub fn set_cell_alive(&mut self, position: Position) {
        while position.x < self.root.min_coord(&self.store)
            || position.y < self.root.min_coord(&self.store)
            || position.x > self.root.max_coord(&self.store)
            || position.y > self.root.max_coord(&self.store)
        {
            self.root = self.root.expand(&mut self.store);
        }
        self.root = self.root.set_cell_alive(&mut self.store, position);
        self.bounding_box = self.root.bounding_box(&self.store);
    }

    /// Returns a list of the positions of the alive cells in the Life grid.
    ///
    /// ```
    /// # fn main() -> Result<(), failure::Error> {
    /// // glider
    /// let life = smeagol::Life::from_rle_pattern(b"bob$2bo$3o!")?;
    ///
    /// for pos in life.get_alive_cells() {
    ///     // do something
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_alive_cells(&self) -> Vec<Position> {
        self.root.get_alive_cells(&self.store)
    }

    /// Returns true if the given bounding box contains any alive cells.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), failure::Error> {
    /// // glider
    /// let life = smeagol::Life::from_rle_pattern(b"bob$2bo$3o!")?;
    ///
    /// assert!(life.contains_alive_cells(life.bounding_box().unwrap()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains_alive_cells(&self, bounding_box: BoundingBox) -> bool {
        if let Some(self_bbox) = self.bounding_box {
            if let Some(intersect) = self_bbox.intersect(bounding_box) {
                self.root.contains_alive_cells(&self.store, intersect)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns a bounding box containing all the alive cells in the Life grid.
    ///
    /// Returns `None` if there are no alive cells in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut life = smeagol::Life::new();
    /// assert!(life.bounding_box().is_none());
    ///
    /// life.set_cell_alive(smeagol::Position::new(0, 0));
    /// assert!(life.bounding_box().is_some());
    /// ```
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        self.bounding_box
    }

    /// Returns the number of generations that have been advanced in the Life grid.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), failure::Error> {
    /// let mut life = smeagol::Life::from_rle_pattern(b"bob$2bo$3o!")?;
    /// assert_eq!(life.generation(), 0);
    ///
    /// life.step();
    /// assert_eq!(life.generation(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn generation(&self) -> u128 {
        self.generation
    }

    /// Returns the number of alive cells in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), failure::Error> {
    /// let mut life = smeagol::Life::from_rle_pattern(b"bob$2bo$3o!")?;
    /// assert_eq!(life.population(), 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn population(&self) -> u128 {
        self.root.population(&self.store)
    }

    /// Returns the current step size.
    ///
    /// The default step size is 1.
    pub fn step_size(&self) -> u64 {
        1 << self.store.step_log_2()
    }

    /// Returns the step size log 2.
    pub fn step_log_2(&self) -> u8 {
        self.store.step_log_2()
    }

    /// Sets the step size to be `2^step_log_2`.
    ///
    /// This clears the cache of previously computed steps.
    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        self.store.set_step_log_2(step_log_2);
    }

    /// Pads the Life grid such that it can be advanced into the future without the edges of the
    /// node interfering.
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

    /// Advances the Life grid into the future.
    ///
    /// The number of generations advanced is determined by the step size.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), failure::Error> {
    /// let mut life = smeagol::Life::from_rle_pattern(b"bob$2bo$3o!")?;
    ///
    /// // step size of 32
    /// life.set_step_log_2(5);
    ///
    /// life.step();
    /// assert_eq!(life.generation(), 32);
    /// # Ok(())
    /// # }
    /// ```
    pub fn step(&mut self) {
        self.pad();
        self.root = self.root.step(&mut self.store);
        self.generation += u128::from(self.step_size());
        self.bounding_box = self.root.bounding_box(&self.store);
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
    fn default() {
        let life = Life::default();
        assert_eq!(life.generation(), 0);
        assert_eq!(life.population(), 0);
    }

    #[test]
    fn get_set_step_size() {
        let mut life = Life::new();
        assert_eq!(life.step_log_2(), 0);
        assert_eq!(life.step_size(), 1);

        life.set_step_log_2(10);
        assert_eq!(life.step_log_2(), 10);
        assert_eq!(life.step_size(), 1024);
    }

    #[test]
    fn empty() {
        let min = i64::min_value();
        let max = i64::max_value();
        let life = Life::new();
        assert_eq!(life.bounding_box(), None);
        assert!(!life.contains_alive_cells(BoundingBox::new(
            Position::new(min, min),
            Position::new(max, max)
        )));
    }

    #[test]
    fn from_rle_file_contents() {
        let life = Life::from_rle_file_contents(b"x = 2, y = 2\n2o$2o!").unwrap();
        assert_eq!(life.population(), 4);
    }

    #[test]
    fn from_rle_pattern() {
        let life = Life::from_rle_pattern(b"bob$2bo$3o!").unwrap();
        assert_eq!(life.population(), 5);
    }

    #[test]
    fn position_extremes() {
        let mut life = Life::new();

        let min = i64::min_value();
        let max = i64::max_value();

        life.set_cell_alive(Position::new(min, min));
        life.set_cell_alive(Position::new(min, max));
        life.set_cell_alive(Position::new(max, min));
        life.set_cell_alive(Position::new(max, max));

        assert_eq!(life.population(), 4);
        let bbox = life.bounding_box().unwrap();
        assert_eq!(bbox.upper_left(), Position::new(min, min),);
        assert_eq!(bbox.lower_right(), Position::new(max, max),);
    }
}
