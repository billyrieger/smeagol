use crate::{node::*, BoundingBox, Cell, Position, Quadrant};

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
                if grid.count_ones().wrapping_sum() == 0 {
                    return vec![];
                }

                let mut alive_coords = vec![];
                for y in -8..8 {
                    let y_offset = (y + 8) as usize;
                    let row = grid.extract(y_offset);
                    for x in -8..8 {
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
                population,
            } => {
                if population == 0 {
                    return vec![];
                }

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
                let (north, south) = partition_vert(coords, offset_y);

                let (northwest, northeast) = partition_horiz(north, offset_x);

                let (southwest, southeast) = partition_horiz(south, offset_x);

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

    pub fn contains_alive_cells(
        self,
        store: &Store,
        upper_left: Position,
        lower_right: Position,
    ) -> bool {
        assert!(upper_left.x <= lower_right.x);
        assert!(upper_left.y <= lower_right.y);

        match store.node(self) {
            Node::Leaf { grid } => {
                if grid.count_ones().wrapping_sum() == 0 {
                    return false;
                }
                for x in upper_left.x..=lower_right.x {
                    for y in upper_left.y..=lower_right.y {
                        let x_offset = (7 - x) as usize;
                        let y_offset = (y + 8) as usize;
                        if grid.extract(y_offset) & (1 << x_offset) > 0 {
                            return true;
                        }
                    }
                }
                false
            }
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                population,
            } => {
                if population == 0 {
                    return false;
                }

                // quarter side length
                let offset = 1 << (level.0 - 2);

                match (upper_left.quadrant(), lower_right.quadrant()) {
                    (Quadrant::Northwest, Quadrant::Northwest) => nw.contains_alive_cells(
                        store,
                        upper_left.offset(offset, offset),
                        lower_right.offset(offset, offset),
                    ),
                    (Quadrant::Northeast, Quadrant::Northeast) => ne.contains_alive_cells(
                        store,
                        upper_left.offset(-offset, offset),
                        lower_right.offset(-offset, offset),
                    ),
                    (Quadrant::Southwest, Quadrant::Southwest) => sw.contains_alive_cells(
                        store,
                        upper_left.offset(offset, -offset),
                        lower_right.offset(offset, -offset),
                    ),
                    (Quadrant::Southeast, Quadrant::Southeast) => se.contains_alive_cells(
                        store,
                        upper_left.offset(-offset, -offset),
                        lower_right.offset(-offset, -offset),
                    ),

                    (Quadrant::Northwest, Quadrant::Northeast) => {
                        let nw_lower_right = Position::new(-1, lower_right.y);
                        let ne_upper_left = Position::new(0, upper_left.y);
                        nw.contains_alive_cells(
                            store,
                            upper_left.offset(offset, offset),
                            nw_lower_right.offset(offset, offset),
                        ) || ne.contains_alive_cells(
                            store,
                            ne_upper_left.offset(-offset, offset),
                            lower_right.offset(-offset, offset),
                        )
                    }
                    (Quadrant::Northwest, Quadrant::Southwest) => {
                        let nw_lower_right = Position::new(lower_right.x, -1);
                        let sw_upper_left = Position::new(upper_left.x, 0);
                        nw.contains_alive_cells(
                            store,
                            upper_left.offset(offset, offset),
                            nw_lower_right.offset(offset, offset),
                        ) || sw.contains_alive_cells(
                            store,
                            sw_upper_left.offset(offset, -offset),
                            lower_right.offset(offset, -offset),
                        )
                    }
                    (Quadrant::Southwest, Quadrant::Southeast) => {
                        let sw_lower_right = Position::new(-1, lower_right.y);
                        let se_upper_left = Position::new(0, upper_left.y);
                        sw.contains_alive_cells(
                            store,
                            upper_left.offset(offset, -offset),
                            sw_lower_right.offset(offset, -offset),
                        ) || ne.contains_alive_cells(
                            store,
                            se_upper_left.offset(-offset, -offset),
                            lower_right.offset(-offset, -offset),
                        )
                    }
                    (Quadrant::Northeast, Quadrant::Southeast) => {
                        let ne_lower_right = Position::new(lower_right.x, -1);
                        let se_upper_left = Position::new(upper_left.x, 0);
                        ne.contains_alive_cells(
                            store,
                            upper_left.offset(-offset, offset),
                            ne_lower_right.offset(-offset, offset),
                        ) || se.contains_alive_cells(
                            store,
                            se_upper_left.offset(-offset, -offset),
                            lower_right.offset(-offset, -offset),
                        )
                    }

                    (Quadrant::Northwest, Quadrant::Southeast) => {
                        let nw_upper_left = upper_left;
                        let nw_lower_right = Position::new(-1, -1);

                        let ne_upper_left = Position::new(0, upper_left.y);
                        let ne_lower_right = Position::new(lower_right.x, -1);

                        let sw_upper_left = Position::new(upper_left.x, 0);
                        let sw_lower_right = Position::new(-1, lower_right.y);

                        let se_upper_left = Position::new(0, 0);
                        let se_lower_right = lower_right;

                        nw.contains_alive_cells(
                            store,
                            nw_upper_left.offset(offset, offset),
                            nw_lower_right.offset(offset, offset),
                        ) || ne.contains_alive_cells(
                            store,
                            ne_upper_left.offset(-offset, offset),
                            ne_lower_right.offset(-offset, offset),
                        ) || sw.contains_alive_cells(
                            store,
                            sw_upper_left.offset(offset, -offset),
                            sw_lower_right.offset(offset, -offset),
                        ) || se.contains_alive_cells(
                            store,
                            se_upper_left.offset(-offset, -offset),
                            se_lower_right.offset(-offset, -offset),
                        )
                    }

                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn bounding_box(self, store: &Store) -> Option<BoundingBox> {
        match store.node(self) {
            Node::Leaf { grid } => {
                if grid.count_ones().wrapping_sum() == 0 {
                    return None;
                }
                let mut min_x = i64::max_value();
                let mut min_y = i64::max_value();
                let mut max_x = i64::min_value();
                let mut max_y = i64::min_value();
                for y in -8..8 {
                    let y_offset = (y + 8) as usize;
                    let row = grid.extract(y_offset);
                    for x in -8..8 {
                        let x_offset = (7 - x) as usize;
                        if row & (1 << x_offset) > 0 {
                            min_x = min_x.min(x);
                            min_y = min_y.min(y);
                            max_x = max_x.max(x);
                            max_y = max_y.max(y);
                        }
                    }
                }
                Some(BoundingBox::new(
                    Position::new(min_x, min_y),
                    Position::new(max_x, max_y),
                ))
            }
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                population,
            } => {
                if population == 0 {
                    return None;
                }

                // quarter side length
                let offset = 1 << (level.0 - 2);

                let mut bounding_box = None::<BoundingBox>;

                if let Some(nw_bounding_box) = nw.bounding_box(store) {
                    let nw_bounding_box = nw_bounding_box.offset(-offset, -offset);
                    bounding_box = if let Some(bbox) = bounding_box {
                        Some(bbox.combine(nw_bounding_box))
                    } else {
                        Some(nw_bounding_box)
                    }
                };

                if let Some(ne_bounding_box) = ne.bounding_box(store) {
                    let ne_bounding_box = ne_bounding_box.offset(offset, -offset);
                    bounding_box = if let Some(bbox) = bounding_box {
                        Some(bbox.combine(ne_bounding_box))
                    } else {
                        Some(ne_bounding_box)
                    }
                };

                if let Some(sw_bounding_box) = sw.bounding_box(store) {
                    let sw_bounding_box = sw_bounding_box.offset(-offset, offset);
                    bounding_box = if let Some(bbox) = bounding_box {
                        Some(bbox.combine(sw_bounding_box))
                    } else {
                        Some(sw_bounding_box)
                    }
                };

                if let Some(se_bounding_box) = se.bounding_box(store) {
                    let se_bounding_box = se_bounding_box.offset(offset, offset);
                    bounding_box = if let Some(bbox) = bounding_box {
                        Some(bbox.combine(se_bounding_box))
                    } else {
                        Some(se_bounding_box)
                    }
                };

                bounding_box
            }
        }
    }
}

fn partition_horiz(coords: &mut [Position], pivot: i64) -> (&mut [Position], &mut [Position]) {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].x < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    coords.split_at_mut(next_index)
}

fn partition_vert(coords: &mut [Position], pivot: i64) -> (&mut [Position], &mut [Position]) {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].y < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    coords.split_at_mut(next_index)
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
                assert_eq!(
                    one_alive.bounding_box(&store),
                    Some(BoundingBox::new(pos, pos))
                );

                assert!(one_alive.contains_alive_cells(&store, pos, pos));
                assert!(one_alive.contains_alive_cells(&store, Position::new(min, min), pos));
                assert!(one_alive.contains_alive_cells(&store, pos, Position::new(max, max)));
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
