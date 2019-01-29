use crate::{node::NodeBase, Cell, Node, NodeTemplate, Store};

impl Node {
    pub fn get_cell(&self, store: &Store, x: i64, y: i64) -> Cell {
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
                            self.se(store).get_cell(store, x - offset, y - offset)
                        }
                    }
                }
            }
        }
    }

    pub fn set_cell(&self, store: &mut Store, x: i64, y: i64, cell: Cell) -> Node {
        assert!(x >= self.min_coord());
        assert!(y >= self.min_coord());
        assert!(x <= self.max_coord());
        assert!(y <= self.max_coord());

        match self.base {
            NodeBase::Leaf { .. } => store.create_leaf(cell),
            NodeBase::Interior { .. } => {
                let ne = self.ne(store);
                let nw = self.nw(store);
                let se = self.se(store);
                let sw = self.sw(store);

                if self.level == 1 {
                    match (x, y) {
                        (-1, -1) => {
                            // nw
                            let nw = nw.set_cell(store, 0, 0, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                        (-1, 0) => {
                            // sw
                            let sw = sw.set_cell(store, 0, 0, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                        (0, -1) => {
                            // ne
                            let ne = ne.set_cell(store, 0, 0, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                        (0, 0) => {
                            // se
                            let se = se.set_cell(store, 0, 0, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                        _ => unreachable!(),
                    }
                } else {
                    // quarter side length
                    let offset = 1 << (self.level - 2);
                    match (x < 0, y < 0) {
                        (true, true) => {
                            // nw
                            let nw = nw.set_cell(store, x + offset, y + offset, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                        (true, false) => {
                            // sw
                            let sw = sw.set_cell(store, x + offset, y - offset, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                        (false, true) => {
                            // ne
                            let ne = ne.set_cell(store, x - offset, y + offset, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                        (false, false) => {
                            // se
                            let se = se.set_cell(store, x - offset, y - offset, cell);
                            store.create_interior(NodeTemplate { ne, nw, se, sw })
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_set_helper(level: u8) {
        let mut store = Store::new();
        let empty = store.create_empty(level);

        for x in empty.min_coord()..=empty.max_coord() {
            for y in empty.min_coord()..=empty.max_coord() {
                assert_eq!(empty.get_cell(&store, x, y), Cell::Dead);

                let one_alive = empty.set_cell(&mut store, x, y, Cell::Alive);
                assert_eq!(one_alive.get_cell(&store, x, y), Cell::Alive);

                let dead_again = one_alive.set_cell(&mut store, x, y, Cell::Dead);
                assert_eq!(dead_again.get_cell(&store, x, y), Cell::Dead);
            }
        }
    }

    #[test]
    fn get_set_leaf() {
        get_set_helper(0)
    }

    #[test]
    fn get_set_level_1() {
        get_set_helper(1)
    }

    #[test]
    fn get_set_level_2() {
        get_set_helper(2)
    }

    #[test]
    fn get_set_level_3() {
        get_set_helper(3)
    }
}
