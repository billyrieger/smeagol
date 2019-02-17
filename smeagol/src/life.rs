use crate::{Cell, NodeTemplate, Node, Store};
use smeagol_rle as rle;

mod render;

#[derive(Clone, Debug)]
pub struct Life {
    root: Node,
    store: Store,
    generation: u128,
}

/// Methods to create a Life board.
impl Life {
    /// Creates an empty Life board.
    pub fn new() -> Self {
        let mut store = Store::new();
        let root = store.create_empty(3);
        Self {
            root,
            store,
            generation: 0,
        }
    }

    pub fn from_macrocell_file<P>(path: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let mut store = Store::new();
        let mc = smeagol_mc::Macrocell::from_file(path)?;
        let mut nodes = vec![];
        for cell in mc.cells {
            match cell {
                smeagol_mc::Cell::LevelThree { cells } => {
                    let mut x = -4;
                    let mut y = -4;
                    let mut positions = vec![];
                    for cell in cells {
                        match cell {
                            '$' => {
                                y += 1;
                                x = -4;
                            }
                            '.' => x += 1,
                            '*' => {
                                positions.push((x, y));
                                x += 1;
                            }
                            _ => unreachable!(),
                        }
                    }
                    nodes.push(store.create_empty(3).set_cells_alive(&mut store, &mut positions));
                }
                smeagol_mc::Cell::Interior { children, level } => {
                    let nw = if children[0] == 0 {
                        store.create_empty(level - 1)
                    } else {
                        nodes[children[0] - 1]
                    };
                    let ne = if children[1] == 0 {
                        store.create_empty(level - 1)
                    } else {
                        nodes[children[1] - 1]
                    };
                    let sw = if children[2] == 0 {
                        store.create_empty(level - 1)
                    } else {
                        nodes[children[2] - 1]
                    };
                    let se = if children[3] == 0 {
                        store.create_empty(level - 1)
                    } else {
                        nodes[children[3] - 1]
                    };
                    nodes.push(store.create_interior(NodeTemplate { nw, ne, sw, se }));
                }
            }
        }
        let root = nodes.last().cloned().unwrap();
        Ok(Self {
            root,
            store,
            generation: 0,
        })
    }

    /// Creates a Life board from the given `Rle` struct.
    #[cfg(feature = "import-rle")]
    fn from_rle(rle: rle::Rle) -> Result<Self, rle::RleError> {
        let mut alive_cells = rle
            .alive_cells()
            .into_iter()
            .map(|(x, y)| (i64::from(x), i64::from(y)))
            .collect::<Vec<_>>();

        let mut store = Store::new();
        let mut root = store.create_empty(3);

        if !alive_cells.is_empty() {
            let x_min = alive_cells.iter().min_by_key(|&(x, _)| x).unwrap().0;
            let x_max = alive_cells.iter().max_by_key(|&(x, _)| x).unwrap().0;
            let y_min = alive_cells.iter().min_by_key(|&(_, y)| y).unwrap().1;
            let y_max = alive_cells.iter().max_by_key(|&(_, y)| y).unwrap().1;

            while x_min < root.min_coord()
                || x_max > root.max_coord()
                || y_min < root.min_coord()
                || y_max > root.max_coord()
            {
                root = root.expand(&mut store);
            }

            root = root.set_cells_alive(&mut store, &mut alive_cells);
        }

        Ok(Self {
            root,
            store,
            generation: 0,
        })
    }

    #[cfg(feature = "import-rle")]
    pub fn from_rle_pattern(pattern: &[u8]) -> Result<Self, rle::RleError> {
        let rle = rle::Rle::from_pattern(pattern)?;
        Self::from_rle(rle)
    }

    #[cfg(feature = "import-rle")]
    pub fn from_rle_file<P>(path: P) -> Result<Self, rle::RleError>
    where
        P: AsRef<std::path::Path>,
    {
        let rle = rle::Rle::from_file(path)?;
        Self::from_rle(rle)
    }

    pub fn generation(&self) -> u128 {
        self.generation
    }

    pub fn population(&self) -> u128 {
        self.root.population(&self.store)
    }
}

/// Methods to get and set individual cells.
impl Life {
    /// Gets the cell at the given coordinates.
    pub fn get_cell(&self, x: i64, y: i64) -> Cell {
        if x < self.root.min_coord()
            || x > self.root.max_coord()
            || y < self.root.min_coord()
            || y > self.root.max_coord()
        {
            Cell::Dead
        } else {
            self.root.get_cell(&self.store, x, y)
        }
    }

    /// Sets the cell at the given coordinates.
    pub fn set_cell(&mut self, x: i64, y: i64, cell: Cell) {
        while x < self.root.min_coord()
            || x > self.root.max_coord()
            || y < self.root.min_coord()
            || y > self.root.max_coord()
        {
            self.root = self.root.expand(&mut self.store);
        }
        self.root = self.root.set_cell(&mut self.store, x, y, cell);
    }

    pub fn get_alive_cells(&self) -> Vec<(i64, i64)> {
        self.root.get_alive_cells(&self.store)
    }

    pub fn set_cells_alive(&mut self, alive_coords: impl IntoIterator<Item = (i64, i64)>) {
        let mut alive_coords = alive_coords.into_iter().collect::<Vec<_>>();
        self.root = self
            .root
            .set_cells_alive(&mut self.store, &mut alive_coords);
    }

    pub fn min_alive_x(&self) -> Option<i64> {
        self.root.min_alive_x(&self.store)
    }

    pub fn min_alive_y(&self) -> Option<i64> {
        self.root.min_alive_y(&self.store)
    }

    pub fn max_alive_x(&self) -> Option<i64> {
        self.root.max_alive_x(&self.store)
    }

    pub fn max_alive_y(&self) -> Option<i64> {
        self.root.max_alive_y(&self.store)
    }

    pub fn contains_alive_cells(&mut self, min: (i64, i64), max: (i64, i64)) -> bool {
        while min.0 < self.root.min_coord()
            || max.0 > self.root.max_coord()
            || min.1 < self.root.min_coord()
            || max.1 > self.root.max_coord()
        {
            self.root = self.root.expand(&mut self.store);
        }
        self.root.contains_alive_cells(&self.store, min, max)
    }
}

/// Methods to evolve a Life board according to Life rules.
impl Life {
    fn pad(&mut self) {
        while self.root.level() < 3
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

    fn step_pow_2(&mut self, step_log_2: u8) {
        self.pad();
        let level_cutoff = step_log_2 + 2;
        while self.root.level() < level_cutoff {
            self.root = self.root.expand(&mut self.store);
        }
        self.root = self.root.step(&mut self.store, level_cutoff);
        self.generation += 1 << step_log_2;
    }

    /// Advances the Life board `step` generations into the future.
    ///
    /// Performs best if the step size is a power of two.
    pub fn step(&mut self, step: u64) {
        let mut step = step;
        let mut power = 0;
        while step > 0 {
            if step % 2 == 1 {
                self.step_pow_2(power);
            }
            step /= 2;
            power += 1;
        }
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
    fn extremes() {
        let mut life = Life::new();

        let min = i64::min_value();
        let max = i64::max_value();

        assert_eq!(life.get_cell(min, min), Cell::Dead);
        assert_eq!(life.get_cell(min, max), Cell::Dead);
        assert_eq!(life.get_cell(max, min), Cell::Dead);
        assert_eq!(life.get_cell(max, max), Cell::Dead);

        life.set_cell(min, min, Cell::Alive);
        life.set_cell(min, max, Cell::Alive);
        life.set_cell(max, min, Cell::Alive);
        life.set_cell(max, max, Cell::Alive);

        assert_eq!(life.get_cell(min, min), Cell::Alive);
        assert_eq!(life.get_cell(min, max), Cell::Alive);
        assert_eq!(life.get_cell(max, min), Cell::Alive);
        assert_eq!(life.get_cell(max, max), Cell::Alive);
    }
}
