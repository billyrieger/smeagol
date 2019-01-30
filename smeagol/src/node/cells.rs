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

    pub fn get_alive_cells(&self, store: &Store) -> Vec<(i64, i64)> {
        match self.base {
            NodeBase::Leaf { alive } => {
                if alive {
                    vec![(0, 0)]
                } else {
                    vec![]
                }
            }
            NodeBase::Interior { .. } => {
                let mut alive_cells = Vec::with_capacity(self.population as usize);

                if self.population > 0 {
                    if self.level == 1 {
                        alive_cells.extend(
                            self.nw(store)
                                .get_alive_cells(store)
                                .into_iter()
                                .map(|(x, y)| (x - 1, y - 1)),
                        );
                        alive_cells.extend(
                            self.ne(store)
                                .get_alive_cells(store)
                                .into_iter()
                                .map(|(x, y)| (x, y - 1)),
                        );
                        alive_cells.extend(
                            self.sw(store)
                                .get_alive_cells(store)
                                .into_iter()
                                .map(|(x, y)| (x - 1, y)),
                        );
                        alive_cells.extend(self.se(store).get_alive_cells(store));
                    } else {
                        // quarter side length
                        let offset = 1 << (self.level - 2);
                        alive_cells.extend(
                            self.nw(store)
                                .get_alive_cells(store)
                                .into_iter()
                                .map(|(x, y)| (x - offset, y - offset)),
                        );
                        alive_cells.extend(
                            self.ne(store)
                                .get_alive_cells(store)
                                .into_iter()
                                .map(|(x, y)| (x + offset, y - offset)),
                        );
                        alive_cells.extend(
                            self.sw(store)
                                .get_alive_cells(store)
                                .into_iter()
                                .map(|(x, y)| (x - offset, y + offset)),
                        );
                        alive_cells.extend(
                            self.se(store)
                                .get_alive_cells(store)
                                .into_iter()
                                .map(|(x, y)| (x + offset, y + offset)),
                        );
                    }
                }

                alive_cells
            }
        }
    }

    pub fn set_cells_alive(&self, store: &mut Store, coords: &mut [(i64, i64)]) -> Node {
        self.set_cells_alive_recursive(store, coords, 0, 0)
    }

    fn set_cells_alive_recursive(
        &self,
        store: &mut Store,
        coords: &mut [(i64, i64)],
        offset_x: i64,
        offset_y: i64,
    ) -> Node {
        if coords.len() == 0 {
            return *self;
        }

        for &(x, y) in coords.iter() {
            assert!(x >= self.min_coord() + offset_x);
            assert!(y >= self.min_coord() + offset_y);
            assert!(x <= self.max_coord() + offset_x);
            assert!(y <= self.max_coord() + offset_y);
        }

        match self.base {
            NodeBase::Leaf { .. } => {
                assert!(coords.len() == 1);
                assert_eq!(coords[0].0 - offset_x, 0);
                assert_eq!(coords[0].1 - offset_y, 0);
                store.create_leaf(Cell::Alive)
            }

            NodeBase::Interior { .. } => {
                let vert_cutoff = partition_vert(coords, offset_y);
                let (north, south) = coords.split_at_mut(vert_cutoff);

                let horiz_cutoff = partition_horiz(north, offset_x);
                let (northwest, northeast) = north.split_at_mut(horiz_cutoff);

                let horiz_cutoff = partition_horiz(south, offset_x);
                let (southwest, southeast) = south.split_at_mut(horiz_cutoff);

                if self.level == 1 {
                    let nw = self.nw(store).set_cells_alive_recursive(
                        store,
                        northwest,
                        offset_x - 1,
                        offset_y - 1,
                    );
                    let ne = self.ne(store).set_cells_alive_recursive(
                        store,
                        northeast,
                        offset_x,
                        offset_y - 1,
                    );
                    let sw = self.sw(store).set_cells_alive_recursive(
                        store,
                        southwest,
                        offset_x - 1,
                        offset_y,
                    );
                    let se = self
                        .se(store)
                        .set_cells_alive_recursive(store, southeast, offset_x, offset_y);

                    store.create_interior(NodeTemplate { ne, nw, se, sw })
                } else {
                    // quarter side length
                    let offset = 1 << (self.level - 2);

                    let nw = self.nw(store).set_cells_alive_recursive(
                        store,
                        northwest,
                        offset_x - offset,
                        offset_y - offset,
                    );
                    let ne = self.ne(store).set_cells_alive_recursive(
                        store,
                        northeast,
                        offset_x + offset,
                        offset_y - offset,
                    );
                    let sw = self.sw(store).set_cells_alive_recursive(
                        store,
                        southwest,
                        offset_x - offset,
                        offset_y + offset,
                    );
                    let se = self.se(store).set_cells_alive_recursive(
                        store,
                        southeast,
                        offset_x + offset,
                        offset_y + offset,
                    );

                    store.create_interior(NodeTemplate { ne, nw, se, sw })
                }
            }
        }
    }
}

fn partition_horiz(coords: &mut [(i64, i64)], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].0 < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}

fn partition_vert(coords: &mut [(i64, i64)], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].1 < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_alive() {
        let mut coords = vec![(0, 0), (1, 1), (-2, 3), (1, -1), (-4, -5)];

        let mut store = Store::new();
        let node = store
            .create_empty(4)
            .set_cells_alive(&mut store, &mut coords);

        for &(x, y) in &coords {
            assert_eq!(node.get_cell(&store, x, y), Cell::Alive);
        }

        let mut alive_cells = node.get_alive_cells(&store);
        assert_eq!(alive_cells.len(), 5);
        coords.sort();
        alive_cells.sort();
        assert_eq!(coords, alive_cells);
    }

    fn get_set_helper(level: u8) {
        let mut store = Store::new();
        let empty = store.create_empty(level);

        for x in empty.min_coord()..=empty.max_coord() {
            for y in empty.min_coord()..=empty.max_coord() {
                assert_eq!(empty.get_cell(&store, x, y), Cell::Dead);

                let one_alive = empty.set_cell(&mut store, x, y, Cell::Alive);
                assert_eq!(one_alive.get_cell(&store, x, y), Cell::Alive);

                let alive_cells = one_alive.get_alive_cells(&store);
                assert_eq!(alive_cells.len(), 1);
                assert_eq!(alive_cells[0], (x, y));

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

    #[test]
    fn partition() {
        let mut coords = vec![(1, 0), (-2, 1), (3, -2), (-1, -2), (0, 0), (-1, -1), (5, 5)];
        let pivot = 0;

        let index = partition_horiz(&mut coords, pivot);
        assert_eq!(index, 3);
        for i in 0..index {
            assert!(coords[i].0 < pivot);
        }
        for i in index..coords.len() {
            assert!(coords[i].0 >= pivot);
        }

        let index = partition_vert(&mut coords, pivot);
        assert_eq!(index, 3);
        for i in 0..index {
            assert!(coords[i].1 < pivot);
        }
        for i in index..coords.len() {
            assert!(coords[i].1 >= pivot);
        }
    }
}