use crate::{node::*, Cell, Position, Quadrant};

const MIN_LVL4_COORD: i64 = -8;
const MAX_LVL4_COORD: i64 = 7;

impl NodeId {
    pub fn get_cell(self, store: &Store, pos: Position) -> Cell {
        match store.node(self) {
            Node::Leaf { grid } => {
                let x_offset = (7 - pos.x) as usize;
                let y_offset = (pos.y + 8) as usize;
                Cell::new(grid.extract(y_offset) & (1 << x_offset) > 0)
            }
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                ..
            } => {
                // quarter side length
                let offset = 1 << (level.0 - 2);

                match pos.quadrant() {
                    Quadrant::Northwest => nw.get_cell(store, pos.offset(offset, offset)),
                    Quadrant::Northeast => ne.get_cell(store, pos.offset(-offset, offset)),
                    Quadrant::Southwest => sw.get_cell(store, pos.offset(offset, -offset)),
                    Quadrant::Southeast => se.get_cell(store, pos.offset(-offset, -offset)),
                }
            }
        }
    }

    pub fn set_cell_alive(self, store: &mut Store, pos: Position) -> NodeId {
        match store.node(self) {
            Node::Leaf { mut grid } => {
                let x_offset = (7 - pos.x) as usize;
                let y_offset = (pos.y + 8) as usize;
                grid = grid.replace(y_offset, grid.extract(y_offset) | (1 << x_offset));
                store.create_leaf(grid)
            }
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                ..
            } => {
                // quarter side length
                let offset = 1 << (level.0 - 2);

                match pos.quadrant() {
                    Quadrant::Northwest => {
                        let nw = nw.set_cell_alive(store, pos.offset(offset, offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    Quadrant::Northeast => {
                        let ne = ne.set_cell_alive(store, pos.offset(-offset, offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    Quadrant::Southwest => {
                        let sw = sw.set_cell_alive(store, pos.offset(offset, -offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    Quadrant::Southeast => {
                        let se = se.set_cell_alive(store, pos.offset(-offset, -offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                }
            }
        }
    }

    pub fn get_alive_cells(self, store: &Store) -> Vec<Position> {
        match store.node(self) {
            Node::Leaf { grid } => {
                let mut alive_coords = vec![];
                for y in MIN_LVL4_COORD..=MAX_LVL4_COORD {
                    let y_offset = (y + 8) as usize;
                    let row = grid.extract(y_offset);
                    for x in MIN_LVL4_COORD..=MAX_LVL4_COORD {
                        let x_offset = (7 - x) as usize;
                        if row & (1 << x_offset) > 0 {
                            alive_coords.push(Position { x, y });
                        }
                    }
                }
                alive_coords
            }
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                ..
            } => {
                let mut alive_cells = vec![];

                // quarter side length
                let offset = 1 << (level.0 - 2);

                alive_cells.extend(
                    nw.get_alive_cells(store)
                        .into_iter()
                        .map(|pos| pos.offset(-offset, -offset)),
                );
                alive_cells.extend(
                    ne.get_alive_cells(store)
                        .into_iter()
                        .map(|pos| pos.offset(offset, -offset)),
                );
                alive_cells.extend(
                    sw.get_alive_cells(store)
                        .into_iter()
                        .map(|pos| pos.offset(-offset, offset)),
                );
                alive_cells.extend(
                    se.get_alive_cells(store)
                        .into_iter()
                        .map(|pos| pos.offset(offset, offset)),
                );

                alive_cells
            }
        }
    }

    pub fn set_cells_alive(
        self,
        store: &mut Store,
        coords: impl IntoIterator<Item = Position>,
    ) -> NodeId {
        self.set_cells_alive_recursive(store, &mut coords.into_iter().collect::<Vec<_>>(), 0, 0)
    }

    fn set_cells_alive_recursive(
        self,
        store: &mut Store,
        coords: &mut [Position],
        offset_x: i64,
        offset_y: i64,
    ) -> NodeId {
        if coords.is_empty() {
            return self;
        }

        match store.node(self) {
            Node::Leaf { mut grid } => {
                for &mut pos in coords {
                    let x = (7 - (pos.x - offset_x)) as usize;
                    let y = ((pos.y - offset_y) + 8) as usize;
                    grid = grid.replace(y, grid.extract(y) | (1 << x));
                }
                store.create_leaf(grid)
            }
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                ..
            } => {
                let vert_cutoff = partition_vert(coords, offset_y);
                let (north, south) = coords.split_at_mut(vert_cutoff);

                let horiz_cutoff = partition_horiz(north, offset_x);
                let (northwest, northeast) = north.split_at_mut(horiz_cutoff);

                let horiz_cutoff = partition_horiz(south, offset_x);
                let (southwest, southeast) = south.split_at_mut(horiz_cutoff);

                // quarter side length
                let offset = 1 << (level.0 - 2);

                let nw = nw.set_cells_alive_recursive(
                    store,
                    northwest,
                    offset_x - offset,
                    offset_y - offset,
                );
                let ne = ne.set_cells_alive_recursive(
                    store,
                    northeast,
                    offset_x + offset,
                    offset_y - offset,
                );
                let sw = sw.set_cells_alive_recursive(
                    store,
                    southwest,
                    offset_x - offset,
                    offset_y + offset,
                );
                let se = se.set_cells_alive_recursive(
                    store,
                    southeast,
                    offset_x + offset,
                    offset_y + offset,
                );

                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
    }
}

fn partition_horiz(coords: &mut [Position], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].x < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}

fn partition_vert(coords: &mut [Position], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].y < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_set_helper(level: u8) {
        let mut store = Store::new();
        let empty = store.create_empty(Level(level));

        let min = empty.min_coord(&store);
        let max = empty.max_coord(&store);
        for x in min..=max {
            for y in min..=max {
                let pos = Position { x, y };
                let one_alive = empty.set_cell_alive(&mut store, pos);
                let also_one_alive = empty.set_cells_alive(&mut store, vec![pos]);
                assert_eq!(one_alive, also_one_alive);
                assert!(one_alive.get_cell(&store, pos).is_alive());
                assert_eq!(one_alive.get_alive_cells(&store), vec![pos]);
            }
        }
    }

    mod level_4 {
        use super::*;

        #[test]
        fn get_set() {
            get_set_helper(4);
        }
    }

    mod level_5 {
        use super::*;

        #[test]
        fn get_set() {
            get_set_helper(5);
        }
    }
}
