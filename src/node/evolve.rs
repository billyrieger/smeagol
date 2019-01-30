use crate::{Cell, Node, NodeTemplate, Store};

impl Node {
    /// For a level `n` node, returns the center subnode of the node `2^(n-2)` ticks into the
    /// future.
    ///
    /// # Panics
    ///
    /// Panics if `n < 2`.
    #[allow(clippy::many_single_char_names)]
    pub fn jump(&self, store: &mut Store) -> Node {
        assert!(self.level >= 2);

        let jump_size = 1 << (self.level - 2);

        // check if the jump has been calculated previously
        if let Some(jump) = store.step(*self, jump_size) {
            return jump;
        }

        // base case: level = 2
        // jump size is 2^(2 - 2) = 1, so a jump is equivalent to a step
        if self.level == 2 {
            let result = self.step_level_2(store);

            store.add_step(*self, 1, result);

            return result;
        }

        // given a level n node, n >= 3, we want to calculate
        // the starred region 2^(n-2) ticks in the future
        //
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | * | * | * | * |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | * | * | * | * |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | * | * | * | * |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | * | * | * | * |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+

        // a-i are all 2^(n-3) ticks in the future
        // since each one is a jump of a level n-1 node
        //
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   | A | A | B | B | C | C |   |
        // +---+---+---+---+---+---+---+---+
        // |   | A | A | B | B | C | C |   |
        // +---+---+---+---+---+---+---+---+
        // |   | D | D | E | E | F | F |   |
        // +---+---+---+---+---+---+---+---+
        // |   | D | D | E | E | F | F |   |
        // +---+---+---+---+---+---+---+---+
        // |   | G | G | H | H | I | I |   |
        // +---+---+---+---+---+---+---+---+
        // |   | G | G | H | H | I | I |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+

        let a = self.nw(store).jump(store);

        let ne = self.ne(store);
        let nw = self.nw(store);
        let b = Node::horiz_jump(store, ne, nw);

        let c = self.ne(store).jump(store);

        let nw = self.nw(store);
        let sw = self.sw(store);
        let d = Node::vert_jump(store, nw, sw);

        let e = self.center_subnode(store).jump(store);

        let ne = self.ne(store);
        let se = self.se(store);
        let f = Node::vert_jump(store, ne, se);

        let g = self.sw(store).jump(store);

        let se = self.se(store);
        let sw = self.sw(store);
        let h = Node::horiz_jump(store, se, sw);

        let i = self.se(store).jump(store);

        // w-z are anther 2^(level - 3) ticks in the future
        // since each one is a jump of a level n-1 node
        //
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | W | W | X | X |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | W | W | X | X |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | Y | Y | Z | Z |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   | Y | Y | Z | Z |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        // |   |   |   |   |   |   |   |   |
        // +---+---+---+---+---+---+---+---+
        let w = store
            .create_interior(NodeTemplate {
                ne: b,
                nw: a,
                se: e,
                sw: d,
            })
            .jump(store);
        let x = store
            .create_interior(NodeTemplate {
                ne: c,
                nw: b,
                se: f,
                sw: e,
            })
            .jump(store);
        let y = store
            .create_interior(NodeTemplate {
                ne: e,
                nw: d,
                se: h,
                sw: g,
            })
            .jump(store);
        let z = store
            .create_interior(NodeTemplate {
                ne: f,
                nw: e,
                se: i,
                sw: h,
            })
            .jump(store);

        // when calculating a-i, we jumped 2^(n-3)
        // when calculating w-z, we jumped 2^(n-3)
        // this makes the total jump 2^(n-2) as desired
        let final_jump = store.create_interior(NodeTemplate {
            ne: x,
            nw: w,
            se: z,
            sw: y,
        });

        // add the jump to the store
        store.add_step(*self, jump_size, final_jump);

        final_jump
    }

    /// Returns the center subnode of the node `2^(cutoff-2)` ticks into the future.
    ///
    /// # Panics
    ///
    /// For a level `n` node, panics if `n < cutoff` or `cutoff < 2`.
    pub fn step(&self, store: &mut Store, level_cutoff: u8) -> Node {
        assert!(self.level >= level_cutoff);
        assert!(level_cutoff >= 2);

        let step_size = 1 << (level_cutoff - 2);

        // check if the result has been calculated previously
        if let Some(step) = store.step(*self, step_size) {
            return step;
        }

        match self.level.cmp(&level_cutoff) {
            std::cmp::Ordering::Less => unreachable!(),
            std::cmp::Ordering::Equal => {
                // when level == level_cutoff, a step is equivalent to a jump
                let jump = self.jump(store);
                store.add_step(*self, step_size, jump);
                jump
            }
            std::cmp::Ordering::Greater => {
                // +---+---+---+---+---+---+---+---+
                // |   |   |   |   |   |   |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   | A | A | B | B | C | C |   |
                // +---+---+---+---+---+---+---+---+
                // |   | A | A | B | B | C | C |   |
                // +---+---+---+---+---+---+---+---+
                // |   | D | D | E | E | F | F |   |
                // +---+---+---+---+---+---+---+---+
                // |   | D | D | E | E | F | F |   |
                // +---+---+---+---+---+---+---+---+
                // |   | G | G | H | H | I | I |   |
                // +---+---+---+---+---+---+---+---+
                // |   | G | G | H | H | I | I |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   |   |   |   |   |   |   |
                // +---+---+---+---+---+---+---+---+
                let a = self.nw(store).center_subnode(store);
                let b = self.north_subsubnode(store);
                let c = self.ne(store).center_subnode(store);
                let d = self.west_subsubnode(store);
                let e = self.center_subnode(store).center_subnode(store);
                let f = self.east_subsubnode(store);
                let g = self.sw(store).center_subnode(store);
                let h = self.south_subsubnode(store);
                let i = self.se(store).center_subnode(store);

                // +---+---+---+---+---+---+---+---+
                // |   |   |   |   |   |   |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   |   |   |   |   |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   | W | W | X | X |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   | W | W | X | X |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   | Y | Y | Z | Z |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   | Y | Y | Z | Z |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   |   |   |   |   |   |   |
                // +---+---+---+---+---+---+---+---+
                // |   |   |   |   |   |   |   |   |
                // +---+---+---+---+---+---+---+---+
                let w = store
                    .create_interior(NodeTemplate {
                        ne: b,
                        nw: a,
                        se: e,
                        sw: d,
                    })
                    .step(store, level_cutoff);
                let x = store
                    .create_interior(NodeTemplate {
                        ne: c,
                        nw: b,
                        se: f,
                        sw: e,
                    })
                    .step(store, level_cutoff);
                let y = store
                    .create_interior(NodeTemplate {
                        ne: e,
                        nw: d,
                        se: h,
                        sw: g,
                    })
                    .step(store, level_cutoff);
                let z = store
                    .create_interior(NodeTemplate {
                        ne: f,
                        nw: e,
                        se: i,
                        sw: h,
                    })
                    .step(store, level_cutoff);

                let final_step = store.create_interior(NodeTemplate {
                    ne: x,
                    nw: w,
                    se: z,
                    sw: y,
                });

                // add the step to the store
                store.add_step(*self, step_size, final_step);

                final_step
            }
        }
    }

    /// Steps a level two node one tick into the future.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is not two.
    fn step_level_2(&self, store: &mut Store) -> Node {
        assert_eq!(self.level, 2);

        // TODO: make this more efficient using bitfields

        let mut nodes = Vec::with_capacity(4);
        for (x, y) in itertools::iproduct!(-1..=0, -1..=0) {
            let mut neighbors = 0;
            for (dx, dy) in itertools::iproduct!(-1..=1, -1..=1).filter(|&c| c != (0, 0)) {
                if self.get_cell(store, x + dx, y + dy).is_alive() {
                    neighbors += 1;
                }
            }
            if neighbors == 3 || (neighbors == 2 && self.get_cell(store, x, y).is_alive()) {
                nodes.push(store.create_leaf(Cell::Alive))
            } else {
                nodes.push(store.create_leaf(Cell::Dead))
            }
        }

        // (-1, -1)
        let nw = nodes[0];
        // (-1, 0)
        let sw = nodes[1];
        // (0, -1)
        let ne = nodes[2];
        // (0, 0)
        let se = nodes[3];

        store.create_interior(NodeTemplate { ne, nw, se, sw })
    }

    /// Given two horizontally adjacent level `n` nodes, compute the level `n-1`
    /// node between them `2^(n-3)` ticks in the future.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   | * | * |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   | * | * |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// ```
    fn horiz_jump(store: &mut Store, e: Node, w: Node) -> Node {
        assert!(e.level >= 2);
        assert_eq!(e.level, w.level);

        let ne = e.nw(store);
        let nw = w.ne(store);
        let se = e.sw(store);
        let sw = w.se(store);

        store
            .create_interior(NodeTemplate { ne, nw, se, sw })
            .jump(store)
    }

    /// Given two vertically adjacent level `n` nodes, compute the level `n-1`
    /// node between them `2^(n-3)` ticks in the future.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// |   | * | * |   |
    /// +---+---+---+---+
    /// |   | * | * |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// ```
    fn vert_jump(store: &mut Store, n: Node, s: Node) -> Node {
        assert!(n.level >= 2);
        assert_eq!(n.level, s.level);

        let ne = n.se(store);
        let nw = n.sw(store);
        let se = s.ne(store);
        let sw = s.nw(store);

        store
            .create_interior(NodeTemplate { ne, nw, se, sw })
            .jump(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod level_2 {
        use super::*;

        #[test]
        fn block() {
            let mut store = Store::new();

            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            // |   | * | * |   |
            // +---+---+---+---+
            // |   | * | * |   |
            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            let block = store
                .create_empty(2)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            // +---+---+
            // | * | * |
            // +---+---+
            // | * | * |
            // +---+---+
            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(block.step_level_2(&mut store), expected);
        }

        #[test]
        fn vert_blinker_ne() {
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            let mut store = Store::new();
            let blinker = store
                .create_empty(2)
                .set_cell(&mut store, 0, -2, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            // +---+---+
            // | * | * |
            // +---+---+
            // |   |   |
            // +---+---+
            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive);

            assert_eq!(blinker.step_level_2(&mut store), expected);
        }

        #[test]
        fn vert_blinker_nw() {
            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            let mut store = Store::new();
            let blinker = store
                .create_empty(2)
                .set_cell(&mut store, -1, -2, Cell::Alive)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive);

            // +---+---+
            // | * | * |
            // +---+---+
            // |   |   |
            // +---+---+
            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive);

            assert_eq!(blinker.step_level_2(&mut store), expected);
        }

        #[test]
        fn vert_blinker_sw() {
            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            let mut store = Store::new();
            let blinker = store
                .create_empty(2)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, -1, 1, Cell::Alive);

            // +---+---+
            // |   |   |
            // +---+---+
            // | * | * |
            // +---+---+
            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(blinker.step_level_2(&mut store), expected);
        }

        #[test]
        fn vert_blinker_se() {
            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            let mut store = Store::new();
            let blinker = store
                .create_empty(2)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive)
                .set_cell(&mut store, 0, 1, Cell::Alive);

            // +---+---+
            // |   |   |
            // +---+---+
            // | * | * |
            // +---+---+
            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(blinker.step_level_2(&mut store), expected);
        }
    }

    mod jump {
        use super::*;

        #[test]
        fn se_glider() {
            let mut store = Store::new();

            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            // | * | * | * |   |
            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            let glider_cells = vec![(-2, 0), (-1, 0), (0, 0), (0, -1), (-1, -2)];

            // returns glider cells offset by the given deltas
            let offset_glider = |dx: i64, dy: i64| -> Vec<(i64, i64)> {
                (&glider_cells)
                    .clone()
                    .into_iter()
                    .map(|(x, y)| (x + dx, y + dy))
                    .collect()
            };

            for level in 4..8 {
                // make a glilder
                let mut glider = store.create_empty(level);
                for &(x, y) in &glider_cells {
                    glider = glider.set_cell(&mut store, x, y, Cell::Alive);
                }

                // jumping decreases the level of the node by one
                // jump size is 2^(level-2), glider's speed is c/4 orthogonally
                // meaning we need to offset by 2^(level-4) in each direction
                let mut expected = store.create_empty(level - 1);
                let offset = 1 << (level - 4);
                for (x, y) in offset_glider(offset, offset) {
                    expected = expected.set_cell(&mut store, x, y, Cell::Alive);
                }

                assert_eq!(glider.jump(&mut store), expected);
            }
        }
    }

    mod step {
        use super::*;

        #[test]
        fn se_glider() {
            let mut store = Store::new();

            // +---+---+---+---+
            // |   | * |   |   |
            // +---+---+---+---+
            // |   |   | * |   |
            // +---+---+---+---+
            // | * | * | * |   |
            // +---+---+---+---+
            // |   |   |   |   |
            // +---+---+---+---+
            let glider_cells = vec![(-2, 0), (-1, 0), (0, 0), (0, -1), (-1, -2)];

            // returns glider cells offset by the given deltas
            let offset_glider = |dx: i64, dy: i64| -> Vec<(i64, i64)> {
                (&glider_cells)
                    .clone()
                    .into_iter()
                    .map(|(x, y)| (x + dx, y + dy))
                    .collect()
            };

            for cutoff in 4..=8 {
                // make a glilder
                let mut glider = store.create_empty(8);
                for &(x, y) in &glider_cells {
                    glider = glider.set_cell(&mut store, x, y, Cell::Alive);
                }

                // stepping decreases the level of the node by one
                // step size is 2^(cutoff-2), glider's speed is c/4 orthogonally
                // meaning we need to offset by 2^(cutoff-4) in each direction
                let mut expected = store.create_empty(7);
                let offset = 1 << (cutoff - 4);
                for (x, y) in offset_glider(offset, offset) {
                    expected = expected.set_cell(&mut store, x, y, Cell::Alive);
                }

                assert_eq!(glider.step(&mut store, cutoff), expected);
            }
        }
    }
}