use crate::{Cell, Node, Store};

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
}

/// Methods to evolve a Life board according to Life rules.
impl Life {
    pub fn step_pow_2(&mut self, step_log_2: u8) {
        let level_cutoff = step_log_2 + 2;
        while self.root.level() < level_cutoff {
            self.root = self.root.expand(&mut self.store);
        }
        self.root = self.root.step(&mut self.store, level_cutoff);
        self.generation += 1 << step_log_2;
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
