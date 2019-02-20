use crate::node::*;

impl NodeId {
    pub fn expand(self, store: &mut Store) -> NodeId {
        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior { nw, ne, sw, se, level, .. } => {
                let empty = store.create_empty(Level(level.0 - 1));

                let nw = store.create_interior(NodeTemplate {
                    nw: empty,
                    ne: empty,
                    sw: empty,
                    se: nw,
                });

                let ne = store.create_interior(NodeTemplate {
                    nw: empty,
                    ne: empty,
                    sw: ne,
                    se: empty,
                });

                let sw = store.create_interior(NodeTemplate {
                    nw: empty,
                    ne: sw,
                    sw: empty,
                    se: empty,
                });

                let se = store.create_interior(NodeTemplate {
                    nw: se,
                    ne: empty,
                    sw: empty,
                    se: empty,
                });

                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
    }

    pub fn nw(self, store: &Store) -> NodeId {
        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior { nw, .. } => nw,
        }
    }

    pub fn ne(self, store: &Store) -> NodeId {
        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior { ne, .. } => ne,
        }
    }

    pub fn sw(self, store: &Store) -> NodeId {
        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior { sw, .. } => sw,
        }
    }

    pub fn se(self, store: &Store) -> NodeId {
        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior { se, .. } => se,
        }
    }

    pub fn center_subnode(self, store: &mut Store) -> NodeId {
        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                ..
            } => {
                if level == Level(5) {
                    let nw_grid = store.node(nw).unwrap_leaf();
                    let ne_grid = store.node(ne).unwrap_leaf();
                    let sw_grid = store.node(sw).unwrap_leaf();
                    let se_grid = store.node(se).unwrap_leaf();
                    store.create_leaf(center(nw_grid, ne_grid, sw_grid, se_grid))
                } else {
                    let template = NodeTemplate {
                        nw: nw.se(store),
                        ne: ne.sw(store),
                        sw: sw.ne(store),
                        se: se.nw(store),
                    };
                    store.create_interior(template)
                }
            }
        }
    }

    pub fn north_subsubnode(self, store: &mut Store) -> NodeId {
        let w = self.nw(store);
        let e = self.ne(store);
        centered_horiz(store, w, e)
    }

    pub fn south_subsubnode(self, store: &mut Store) -> NodeId {
        let w = self.sw(store);
        let e = self.se(store);
        centered_horiz(store, w, e)
    }

    pub fn west_subsubnode(self, store: &mut Store) -> NodeId {
        let n = self.nw(store);
        let s = self.sw(store);
        centered_vert(store, n, s)
    }

    pub fn east_subsubnode(self, store: &mut Store) -> NodeId {
        let n = self.ne(store);
        let s = self.se(store);
        centered_vert(store, n, s)
    }
}

fn centered_horiz(store: &mut Store, w: NodeId, e: NodeId) -> NodeId {
    match (store.node(w), store.node(e)) {
        (Node::Leaf { .. }, Node::Leaf { .. }) => panic!(),
        (
            Node::Interior {
                level,
                ne: w_ne,
                se: w_se,
                ..
            },
            Node::Interior {
                nw: e_nw, sw: e_sw, ..
            },
        ) => {
            if level == Level(5) {
                let nw_grid = store.node(w_ne).unwrap_leaf();
                let ne_grid = store.node(e_nw).unwrap_leaf();
                let sw_grid = store.node(w_se).unwrap_leaf();
                let se_grid = store.node(e_sw).unwrap_leaf();
                store.create_leaf(center(nw_grid, ne_grid, sw_grid, se_grid))
            } else {
                let nw = w_ne.se(store);
                let ne = e_nw.sw(store);
                let sw = w_se.ne(store);
                let se = e_sw.nw(store);
                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
        _ => unreachable!(),
    }
}

fn centered_vert(store: &mut Store, n: NodeId, s: NodeId) -> NodeId {
    match (store.node(n), store.node(s)) {
        (Node::Leaf { .. }, Node::Leaf { .. }) => panic!(),
        (
            Node::Interior {
                level,
                sw: n_sw,
                se: n_se,
                ..
            },
            Node::Interior {
                nw: s_nw, ne: s_ne, ..
            },
        ) => {
            if level == Level(5) {
                let nw_grid = store.node(n_sw).unwrap_leaf();
                let ne_grid = store.node(n_se).unwrap_leaf();
                let sw_grid = store.node(s_nw).unwrap_leaf();
                let se_grid = store.node(s_ne).unwrap_leaf();
                store.create_leaf(center(nw_grid, ne_grid, sw_grid, se_grid))
            } else {
                let nw = n_sw.se(store);
                let ne = n_se.sw(store);
                let sw = s_nw.ne(store);
                let se = s_ne.nw(store);
                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;

    #[test]
    fn center_subnode_level_5() {
        let mut store = Store::new();

        let nw = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
        ));

        let ne = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
        ));

        let sw = store.create_leaf(u16x16::new(
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_1111_1111,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        let se = store.create_leaf(u16x16::new(
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b1111_1111_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        let expected_center_subnode = store.create_leaf(u16x16::splat(0b1111_1111_1111_1111));

        let level_5 = store.create_interior(NodeTemplate { nw, ne, sw, se });
        let center_subnode = level_5.center_subnode(&mut store);

        assert_eq!(center_subnode, expected_center_subnode);
    }

    #[test]
    fn north_subsubnode_level_6() {
        let mut store = Store::new();

        let mut level_6 = store.create_empty(Level(6));
        for x in -8..8 {
            for y in -24..8 {
                level_6 = level_6.set_cell_alive(&mut store, Position { x, y });
            }
        }

        let north_subsubnode = level_6.north_subsubnode(&mut store);
        let expected_north_subsubnode = store.create_leaf(u16x16::splat(0b1111_1111_1111_1111));

        assert_eq!(north_subsubnode, expected_north_subsubnode);
    }

    #[test]
    fn south_subsubnode_level_6() {
        let mut store = Store::new();

        let mut level_6 = store.create_empty(Level(6));
        for x in -8..8 {
            for y in 8..24 {
                level_6 = level_6.set_cell_alive(&mut store, Position { x, y });
            }
        }

        let south_subsubnode = level_6.south_subsubnode(&mut store);
        let expected_south_subsubnode = store.create_leaf(u16x16::splat(0b1111_1111_1111_1111));

        assert_eq!(south_subsubnode, expected_south_subsubnode);
    }

    #[test]
    fn west_subsubnode_level_6() {
        let mut store = Store::new();

        let mut level_6 = store.create_empty(Level(6));
        for x in -24..-8 {
            for y in -8..8 {
                level_6 = level_6.set_cell_alive(&mut store, Position { x, y });
            }
        }

        let west_subsubnode = level_6.west_subsubnode(&mut store);
        let expected_west_subsubnode = store.create_leaf(u16x16::splat(0b1111_1111_1111_1111));

        assert_eq!(west_subsubnode, expected_west_subsubnode);
    }

    #[test]
    fn east_subsubnode_level_6() {
        let mut store = Store::new();

        let mut level_6 = store.create_empty(Level(6));
        for x in 8..24 {
            for y in -8..8 {
                level_6 = level_6.set_cell_alive(&mut store, Position { x, y });
            }
        }

        let east_subsubnode = level_6.east_subsubnode(&mut store);
        let expected_east_subsubnode = store.create_leaf(u16x16::splat(0b1111_1111_1111_1111));

        assert_eq!(east_subsubnode, expected_east_subsubnode);
    }
}
