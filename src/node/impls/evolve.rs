/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::node::*;
use packed_simd::u16x16;

#[derive(Clone, Copy, Debug)]
struct Counts {
    low: u16x16,
    mid: u16x16,
    high: u16x16,
}

impl Counts {
    fn new() -> Self {
        Self {
            low: u16x16::splat(0),
            mid: u16x16::splat(0),
            high: u16x16::splat(0),
        }
    }

    fn add(&mut self, neighbors: u16x16) {
        // low bit half adder
        let low_carry = self.low & neighbors;
        self.low ^= neighbors;

        // middle bit half adder
        let mid_carry = self.mid & low_carry;
        self.mid ^= low_carry;

        // high bit saturating add
        self.high |= mid_carry;
    }
}

fn rotate_lanes_up(board: u16x16) -> u16x16 {
    shuffle!(
        board,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]
    )
}

fn rotate_lanes_down(board: u16x16) -> u16x16 {
    shuffle!(
        board,
        [15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
    )
}

fn step_once_u16x16(board: u16x16) -> u16x16 {
    let mut neighbors = Counts::new();

    // +---+---+---+
    // | * | * | * |
    // +---+---+---+
    // | * |   | * |
    // +---+---+---+
    // | * | * | * |
    // +---+---+---+

    // top row
    neighbors.add(rotate_lanes_down(board) >> 1);
    neighbors.add(rotate_lanes_down(board));
    neighbors.add(rotate_lanes_down(board) << 1);

    // middle row
    neighbors.add(board >> 1);
    neighbors.add(board << 1);

    // bottom row
    neighbors.add(rotate_lanes_up(board) >> 1);
    neighbors.add(rotate_lanes_up(board));
    neighbors.add(rotate_lanes_up(board) << 1);

    // 2 is 010 in binary
    let two_neighbors = !neighbors.high & neighbors.mid & !neighbors.low;
    // 3 is 011 in binary
    let three_neighbors = !neighbors.high & neighbors.mid & neighbors.low;

    // if 2 neighbors, the cell doesn't change
    // if 3 neighbors, the cell is alive
    (two_neighbors & board) | three_neighbors
}

fn jump_u16x16(mut board: u16x16) -> u16x16 {
    board = step_once_u16x16(board);
    board = step_once_u16x16(board);
    board = step_once_u16x16(board);
    board = step_once_u16x16(board);
    board
}

fn step_u16x16(mut board: u16x16, step_log_2: u8) -> u16x16 {
    for _ in 0..(1 << step_log_2) {
        board = step_once_u16x16(board);
    }
    board
}

fn horiz_jump_u16x16(w: u16x16, e: u16x16) -> u16x16 {
    let grid = (w << 8) | (e >> 8);
    jump_u16x16(grid)
}

fn vert_jump_u16x16(n: u16x16, s: u16x16) -> u16x16 {
    let n = shuffle!(n, [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7])
        & LEVEL_4_UPPER_HALF_MASK;
    let s = shuffle!(s, [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7])
        & LEVEL_4_LOWER_HALF_MASK;
    let grid = n | s;
    jump_u16x16(grid)
}

fn center_jump_u16x16(
    nw_grid: u16x16,
    ne_grid: u16x16,
    sw_grid: u16x16,
    se_grid: u16x16,
) -> u16x16 {
    let grid = center(nw_grid, ne_grid, sw_grid, se_grid);
    jump_u16x16(grid)
}

fn combine_results_u16x16(
    nw_grid: u16x16,
    ne_grid: u16x16,
    sw_grid: u16x16,
    se_grid: u16x16,
) -> u16x16 {
    let nw_grid = nw_grid << 4;
    let nw_grid = shuffle!(
        nw_grid,
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3]
    ) & LEVEL_4_NW_MASK;

    let ne_grid = ne_grid >> 4;
    let ne_grid = shuffle!(
        ne_grid,
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3]
    ) & LEVEL_4_NE_MASK;

    let sw_grid = sw_grid << 4;
    let sw_grid = shuffle!(
        sw_grid,
        [12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    ) & LEVEL_4_SW_MASK;

    let se_grid = se_grid >> 4;
    let se_grid = shuffle!(
        se_grid,
        [12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    ) & LEVEL_4_SE_MASK;

    nw_grid | ne_grid | sw_grid | se_grid
}

fn horiz_u16x16(w: u16x16, e: u16x16) -> u16x16 {
    (w << 8) | (e >> 8)
}

fn vert_u16x16(n: u16x16, s: u16x16) -> u16x16 {
    let n = shuffle!(n, [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7])
        & LEVEL_4_UPPER_HALF_MASK;
    let s = shuffle!(s, [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7])
        & LEVEL_4_LOWER_HALF_MASK;
    n | s
}

#[allow(clippy::many_single_char_names)]
fn step_level_5(
    store: &mut Store,
    step_log_2: u8,
    nw: NodeId,
    ne: NodeId,
    sw: NodeId,
    se: NodeId,
) -> NodeId {
    let nw_grid = store.node(nw).unwrap_leaf();
    let ne_grid = store.node(ne).unwrap_leaf();
    let sw_grid = store.node(sw).unwrap_leaf();
    let se_grid = store.node(se).unwrap_leaf();

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

    let a = nw_grid;
    let b = horiz_u16x16(nw_grid, ne_grid);
    let c = ne_grid;
    let d = vert_u16x16(nw_grid, sw_grid);
    let e = center(nw_grid, ne_grid, sw_grid, se_grid);
    let f = vert_u16x16(ne_grid, se_grid);
    let g = sw_grid;
    let h = horiz_u16x16(sw_grid, se_grid);
    let i = se_grid;

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

    let w = step_u16x16(combine_results_u16x16(a, b, d, e), step_log_2);
    let x = step_u16x16(combine_results_u16x16(b, c, e, f), step_log_2);
    let y = step_u16x16(combine_results_u16x16(d, e, g, h), step_log_2);
    let z = step_u16x16(combine_results_u16x16(e, f, h, i), step_log_2);

    store.create_leaf(combine_results_u16x16(w, x, y, z))
}

#[allow(clippy::many_single_char_names)]
fn jump_level_5(store: &mut Store, nw: NodeId, ne: NodeId, sw: NodeId, se: NodeId) -> NodeId {
    let nw_grid = store.node(nw).unwrap_leaf();
    let ne_grid = store.node(ne).unwrap_leaf();
    let sw_grid = store.node(sw).unwrap_leaf();
    let se_grid = store.node(se).unwrap_leaf();

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

    let a = jump_u16x16(nw_grid);
    let b = horiz_jump_u16x16(nw_grid, ne_grid);
    let c = jump_u16x16(ne_grid);
    let d = vert_jump_u16x16(nw_grid, sw_grid);;
    let e = center_jump_u16x16(nw_grid, ne_grid, sw_grid, se_grid);
    let f = vert_jump_u16x16(ne_grid, se_grid);;
    let g = jump_u16x16(sw_grid);
    let h = horiz_jump_u16x16(sw_grid, se_grid);
    let i = jump_u16x16(se_grid);

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

    let w = jump_u16x16(combine_results_u16x16(a, b, d, e));
    let x = jump_u16x16(combine_results_u16x16(b, c, e, f));
    let y = jump_u16x16(combine_results_u16x16(d, e, g, h));
    let z = jump_u16x16(combine_results_u16x16(e, f, h, i));

    store.create_leaf(combine_results_u16x16(w, x, y, z))
}

fn horiz_jump(store: &mut Store, w: NodeId, e: NodeId) -> NodeId {
    let nw = w.ne(store);
    let ne = e.nw(store);
    let sw = w.se(store);
    let se = e.sw(store);

    store
        .create_interior(NodeTemplate { nw, ne, sw, se })
        .jump(store)
}

fn vert_jump(store: &mut Store, n: NodeId, s: NodeId) -> NodeId {
    let nw = n.sw(store);
    let ne = n.se(store);
    let sw = s.nw(store);
    let se = s.ne(store);

    store
        .create_interior(NodeTemplate { nw, ne, sw, se })
        .jump(store)
}

impl NodeId {
    /// For a level `n` node, advances the node `2^(n-2)` generations into the future.
    ///
    /// Returns a level `n-1` node.
    #[allow(clippy::many_single_char_names)]
    pub fn jump(self, store: &mut Store) -> NodeId {
        if let Some(jump) = store.get_jump(self) {
            return jump;
        }

        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                population,
            } => {
                if population == 0 {
                    return store.create_empty(Level(level.0 - 1));
                }

                if level == Level(5) {
                    jump_level_5(store, nw, ne, sw, se)
                } else {
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

                    let a = nw.jump(store);
                    let b = horiz_jump(store, nw, ne);
                    let c = ne.jump(store);
                    let d = vert_jump(store, nw, sw);
                    let e = self.center_subnode(store).jump(store);
                    let f = vert_jump(store, ne, se);
                    let g = sw.jump(store);
                    let h = horiz_jump(store, sw, se);
                    let i = se.jump(store);

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
                            nw: a,
                            ne: b,
                            sw: d,
                            se: e,
                        })
                        .jump(store);

                    let x = store
                        .create_interior(NodeTemplate {
                            nw: b,
                            ne: c,
                            sw: e,
                            se: f,
                        })
                        .jump(store);

                    let y = store
                        .create_interior(NodeTemplate {
                            nw: d,
                            ne: e,
                            sw: g,
                            se: h,
                        })
                        .jump(store);

                    let z = store
                        .create_interior(NodeTemplate {
                            nw: e,
                            ne: f,
                            sw: h,
                            se: i,
                        })
                        .jump(store);

                    let jump = store.create_interior(NodeTemplate {
                        nw: w,
                        ne: x,
                        sw: y,
                        se: z,
                    });
                    store.add_jump(self, jump);
                    jump
                }
            }
        }
    }

    /// For a level `n` node, advances the node `step_size` generations into the future.
    ///
    /// The step size is determined by the store.
    ///
    /// Returns a level `n-1` node.
    #[allow(clippy::many_single_char_names)]
    pub fn step(self, store: &mut Store) -> NodeId {
        if let Some(step) = store.get_step(self) {
            return step;
        }

        let step_log_2 = store.step_log_2();

        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                population,
            } => {
                if step_log_2 == level.0 - 2 {
                    let step = self.jump(store);
                    store.add_step(self, step);
                    return step;
                }

                if population == 0 {
                    return store.create_empty(Level(level.0 - 1));
                }

                if level == Level(5) {
                    let step = step_level_5(store, step_log_2, nw, ne, sw, se);
                    store.add_step(self, step);
                    step
                } else {
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

                    let a = nw.center_subnode(store);
                    let b = self.north_subsubnode(store);
                    let c = ne.center_subnode(store);
                    let d = self.west_subsubnode(store);
                    let e = self.center_subnode(store).center_subnode(store);
                    let f = self.east_subsubnode(store);
                    let g = sw.center_subnode(store);
                    let h = self.south_subsubnode(store);
                    let i = se.center_subnode(store);

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
                            nw: a,
                            ne: b,
                            sw: d,
                            se: e,
                        })
                        .step(store);

                    let x = store
                        .create_interior(NodeTemplate {
                            nw: b,
                            ne: c,
                            sw: e,
                            se: f,
                        })
                        .step(store);

                    let y = store
                        .create_interior(NodeTemplate {
                            nw: d,
                            ne: e,
                            sw: g,
                            se: h,
                        })
                        .step(store);

                    let z = store
                        .create_interior(NodeTemplate {
                            nw: e,
                            ne: f,
                            sw: h,
                            se: i,
                        })
                        .step(store);

                    let step = store.create_interior(NodeTemplate {
                        nw: w,
                        ne: x,
                        sw: y,
                        se: z,
                    });
                    store.add_step(self, step);
                    step
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nw_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0010,
            0b0000_0000_0000_0001,
            0b0000_0000_0000_0111,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: glider,
            ne: empty,
            sw: empty,
            se: empty,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_1000_0000,
            0b0000_0000_0100_0000,
            0b0000_0001_1100_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn ne_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0100_0000_0000_0000,
            0b1000_0000_0000_0000,
            0b1110_0000_0000_0000,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: empty,
            ne: glider,
            sw: empty,
            se: empty,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0001_0000_0000,
            0b0000_0010_0000_0000,
            0b0000_0011_1000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn sw_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0111,
            0b0000_0000_0000_0001,
            0b0000_0000_0000_0010,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: empty,
            ne: empty,
            sw: glider,
            se: empty,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0001_1100_0000,
            0b0000_0000_0100_0000,
            0b0000_0000_1000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn se_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b1110_0000_0000_0000,
            0b1000_0000_0000_0000,
            0b0100_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: empty,
            ne: empty,
            sw: empty,
            se: glider,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0011_1000_0000,
            0b0000_0010_0000_0000,
            0b0000_0001_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn nw_glider_step() {
        let mut store = Store::new();
        store.set_step_log_2(2);

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0010,
            0b0000_0000_0000_0001,
            0b0000_0000_0000_0111,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: glider,
            ne: empty,
            sw: empty,
            se: empty,
        });

        let step = level_5.step(&mut store);
        let expected_step = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0001_0000_0000,
            0b0000_0000_1000_0000,
            0b0000_0011_1000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(step, expected_step);
    }
}
