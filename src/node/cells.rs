use crate::{node::NodeBase, Cell, Node, Store};

impl Node {
    fn get_cell(&self, store: &Store, x: i64, y: i64) -> Cell {
        assert!(x >= self.min_coord());
        assert!(y >= self.min_coord());
        assert!(x <= self.max_coord());
        assert!(y <= self.max_coord());

        match self.base {
            NodeBase::Leaf { alive } => {
                if alive {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }
            NodeBase::Interior { .. } => {
                if self.level == 1 {
                    match (x, y) {
                        (-1, -1) => {
                            // nw
                            self.nw(store).get_cell(store, 0, 0)
                        }
                        (-1, 0) => {
                            // sw
                            self.sw(store).get_cell(store, 0, 0)
                        }
                        (0, -1) => {
                            // ne
                            self.ne(store).get_cell(store, 0, 0)
                        }
                        (0, 0) => {
                            // se
                            self.se(store).get_cell(store, 0, 0)
                        }
                        _ => unreachable!(),
                    }
                } else {
                    // quarter side length
                    let offset = 1 << (self.level - 2);
                    match (x < 0, y < 0) {
                        (true, true) => {
                            // nw
                            self.nw(store).get_cell(store, x + offset, y + offset)
                        }
                        (true, false) => {
                            // sw
                            self.sw(store).get_cell(store, x + offset, y - offset)
                        }
                        (false, true) => {
                            // ne
                            self.ne(store).get_cell(store, x - offset, y + offset)
                        }
                        (false, false) => {
                            // se
                            self.nw(store).get_cell(store, x - offset, y - offset)
                        }
                    }
                }
            }
        }
    }
}
