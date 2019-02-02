use crate::{Cell, Node, Store};
use png::HasParameters;

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

    fn from_rle(rle: crate::rle::Rle) -> Result<Self, crate::rle::RleError> {
        let mut alive_cells = rle
            .alive_cells()
            .into_iter()
            .map(|(x, y)| (x as i64, y as i64))
            .collect::<Vec<_>>();

        let mut store = Store::new();
        let mut root = store.create_empty(3);

        if alive_cells.len() > 0 {
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
    pub fn from_rle_pattern(pattern: &[u8]) -> Result<Self, crate::rle::RleError> {
        let rle = crate::rle::Rle::from_pattern(pattern)?;
        Self::from_rle(rle)
    }

    #[cfg(feature = "import-rle")]
    pub fn from_rle_file<P>(path: P) -> Result<Self, crate::rle::RleError>
    where
        P: AsRef<std::path::Path>,
    {
        let rle = crate::rle::Rle::from_file(path)?;
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
    pub fn get_cell(&mut self, x: i64, y: i64) -> Cell {
        if x < self.root.min_coord()
            || x > self.root.max_coord()
            || y < self.root.min_coord()
            || y > self.root.max_coord()
        {
            Cell::Dead
        } else {
            self.root.get_cell(&mut self.store, x, y)
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

    pub fn get_alive_cells(&mut self) -> Vec<(i64, i64)> {
        self.root.get_alive_cells(&mut self.store)
    }
}

/// Methods to evolve a Life board according to Life rules.
impl Life {
    fn pad(&mut self) {
        while self.root.level() < 3
            || self.root.ne(&mut self.store).population(&self.store)
                != self
                    .root
                    .ne(&mut self.store)
                    .sw(&mut self.store)
                    .sw(&mut self.store)
                    .population(&self.store)
            || self.root.nw(&mut self.store).population(&self.store)
                != self
                    .root
                    .nw(&mut self.store)
                    .se(&mut self.store)
                    .se(&mut self.store)
                    .population(&self.store)
            || self.root.se(&mut self.store).population(&self.store)
                != self
                    .root
                    .se(&mut self.store)
                    .nw(&mut self.store)
                    .nw(&mut self.store)
                    .population(&self.store)
            || self.root.sw(&mut self.store).population(&self.store)
                != self
                    .root
                    .sw(&mut self.store)
                    .ne(&mut self.store)
                    .ne(&mut self.store)
                    .population(&self.store)
        {
            self.root = self.root.expand(&mut self.store);
        }
    }

    pub fn step_pow_2(&mut self, step_log_2: u8) {
        self.pad();
        let level_cutoff = step_log_2 + 2;
        while self.root.level() < level_cutoff {
            self.root = self.root.expand(&mut self.store);
        }
        self.root = self.root.step(&mut self.store, level_cutoff);
        self.generation += 1 << step_log_2;
    }

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

/// Methods to export a Life board.
impl Life {
    #[cfg(feature = "export-png")]
    pub fn save_png<P>(&mut self, path: P)
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);

        let alive_cells = self.root.get_alive_cells(&mut self.store);
        if alive_cells.len() > 0 {
            let x_min = alive_cells.iter().map(|(x, _)| x).min().cloned().unwrap();
            let y_min = alive_cells.iter().map(|(_, y)| y).min().cloned().unwrap();
            let x_max = alive_cells.iter().map(|(x, _)| x).max().cloned().unwrap();
            let y_max = alive_cells.iter().map(|(_, y)| y).max().cloned().unwrap();
            let width = x_max - x_min + 1;
            let height = y_max - y_min + 1;

            // white rectangle
            let mut data = vec![255u8; (width * height * 4) as usize];

            for &(x, y) in &alive_cells {
                let offset_x = x - x_min;
                let offset_y = y - y_min;
                let index = (4 * (offset_y * width + offset_x)) as usize;
                data[index] = 0;
                data[index + 1] = 0;
                data[index + 2] = 0;
            }

            let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
            encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);

            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&data).unwrap();
        }
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
