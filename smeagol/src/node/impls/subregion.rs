use crate::{
    node::{self, NodeBase},
    Cell, node::{Node, NodeTemplate, Store
}};

/// Methods for extracting subregions of a node.
///
/// # Examples
///
/// ```
/// let mut store = smeagol::node::Store::new();
/// let node = store.create_random_filled(4, 0.5);
///
/// let ne = node.ne(&store);
/// let nw = node.nw(&store);
/// let se = node.se(&store);
/// let sw = node.sw(&store);
///
/// assert_eq!(
///     node,
///     store.create_interior(smeagol::node::NodeTemplate { ne, nw, se, sw })
/// );
/// ```
impl Node {
    pub fn expand(&self, store: &mut Store) -> Node {
        assert!(self.level >= 1);
        let border = store.create_empty(self.level - 1);

        let self_ne = self.ne(store);
        let ne = store.create_interior(NodeTemplate {
            ne: border,
            nw: border,
            se: border,
            sw: self_ne,
        });

        let self_nw = self.nw(store);
        let nw = store.create_interior(NodeTemplate {
            ne: border,
            nw: border,
            se: self_nw,
            sw: border,
        });

        let self_se = self.se(store);
        let se = store.create_interior(NodeTemplate {
            ne: border,
            nw: self_se,
            se: border,
            sw: border,
        });

        let self_sw = self.sw(store);
        let sw = store.create_interior(NodeTemplate {
            ne: self_sw,
            nw: border,
            se: border,
            sw: border,
        });

        store.create_interior(NodeTemplate { ne, nw, se, sw })
    }

    /// Returns the northeast quadrant of the node.  
    ///
    /// # Panics
    ///
    /// Panics if the node is a leaf node.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+
    /// |   | * |
    /// +---+---+
    /// |   |   |
    /// +---+---+
    /// ```
    pub fn ne(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::LevelOne { cells } => {
                store.create_leaf(if cells & node::LEVEL_ONE_NE_MASK > 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                })
            }
            NodeBase::LevelTwo { cells } => {
                store.create_level_one_from_cells(cells.to_be_bytes()[0] & node::LEVEL_ONE_MASK)
            }
            NodeBase::Interior { ne_index, .. } => store.node(ne_index),
        }
    }

    /// Returns the northwest quadrant of the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is a leaf node.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+
    /// | * |   |
    /// +---+---+
    /// |   |   |
    /// +---+---+
    /// ```
    pub fn nw(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::LevelOne { cells } => {
                store.create_leaf(if cells & node::LEVEL_ONE_NW_MASK > 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                })
            }
            NodeBase::LevelTwo { cells } => store
                .create_level_one_from_cells((cells.to_be_bytes()[0] >> 2) & node::LEVEL_ONE_MASK),
            NodeBase::Interior { nw_index, .. } => store.node(nw_index),
        }
    }

    /// Returns the southeast quadrant of the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is a leaf node.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+
    /// |   |   |
    /// +---+---+
    /// |   | * |
    /// +---+---+
    /// ```
    pub fn se(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::LevelOne { cells } => {
                store.create_leaf(if cells & node::LEVEL_ONE_SE_MASK > 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                })
            }
            NodeBase::LevelTwo { cells } => {
                store.create_level_one_from_cells(cells.to_be_bytes()[1] & node::LEVEL_ONE_MASK)
            }
            NodeBase::Interior { se_index, .. } => store.node(se_index),
        }
    }

    /// Returns the southwest quadrant of the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is a leaf node.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+
    /// |   |   |
    /// +---+---+
    /// | * |   |
    /// +---+---+
    /// ```
    pub fn sw(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::LevelOne { cells } => {
                store.create_leaf(if cells & node::LEVEL_ONE_SW_MASK > 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                })
            }
            NodeBase::LevelTwo { cells } => store
                .create_level_one_from_cells((cells.to_be_bytes()[1] >> 2) & node::LEVEL_ONE_MASK),
            NodeBase::Interior { sw_index, .. } => store.node(sw_index),
        }
    }

    /// Returns the center subnode of the node.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is less than 2.
    ///
    /// ```txt
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// |   | * | * |   |
    /// +---+---+---+---+
    /// |   | * | * |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// ```
    pub fn center_subnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 2);

        let ne = self.ne(store).sw(store);
        let nw = self.nw(store).se(store);
        let se = self.se(store).nw(store);
        let sw = self.sw(store).ne(store);

        store.create_interior(NodeTemplate { ne, nw, se, sw })
    }

    /// Returns the north subsubnode of the node.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is less than 3.
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
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// ```
    pub fn north_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let e = self.ne(store);
        let w = self.nw(store);

        Self::centered_horiz(store, e, w)
    }

    /// Returns the south subsubnode of the node.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is less than 3.
    ///
    /// # Diagram
    ///  
    /// ```txt
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
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
    pub fn south_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let e = self.se(store);
        let w = self.sw(store);

        Self::centered_horiz(store, e, w)
    }

    /// Returns the east subsubnode of the node.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is less than 3.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   | * | * |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   | * | * |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// ```
    pub fn east_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let n = self.ne(store);
        let s = self.se(store);

        Self::centered_vert(store, n, s)
    }

    /// Returns the west subsubnode of the node.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is less than 3.
    ///
    /// # Diagram
    ///
    /// ```txt
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   | * | * |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   | * | * |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   |
    /// +---+---+---+---+---+---+---+---+
    /// ```
    pub fn west_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let n = self.nw(store);
        let s = self.sw(store);

        Self::centered_vert(store, n, s)
    }

    /// Given two horizontally adjacent nodes, returns the subnode halfway
    /// between them.
    ///
    /// # Panics
    ///
    /// Panics if the nodes have different levels or the level of the nodes is
    /// less than 2.
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
    fn centered_horiz(store: &mut Store, e: Node, w: Node) -> Node {
        assert_eq!(e.level, w.level);
        assert!(e.level >= 2);

        let ne = e.nw(store).sw(store);
        let nw = w.ne(store).se(store);
        let se = e.sw(store).nw(store);
        let sw = w.se(store).ne(store);

        store.create_interior(NodeTemplate { ne, nw, se, sw })
    }

    /// Given two vertically adjacent nodes, returns the subnode halfway between
    /// them.
    ///
    /// # Panics
    ///
    /// Panics if the nodes have different levels or the level of the nodes is
    /// less than 2.
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
    fn centered_vert(store: &mut Store, n: Node, s: Node) -> Node {
        assert_eq!(n.level, s.level);
        assert!(n.level >= 2);

        let ne = n.se(store).sw(store);
        let nw = n.sw(store).se(store);
        let se = s.ne(store).nw(store);
        let sw = s.nw(store).ne(store);

        store.create_interior(NodeTemplate { ne, nw, se, sw })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cell;

    mod subnode {
        use super::*;

        #[test]
        fn ne_lvl2() {
            let mut store = Store::new();
            let node = store
                .create_empty(2)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, -2, Cell::Alive)
                .set_cell(&mut store, 1, -1, Cell::Alive)
                .set_cell(&mut store, 1, -2, Cell::Alive);

            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(node.ne(&mut store), expected);
        }

        #[test]
        fn ne() {
            let mut store = Store::new();
            let node = store
                .create_empty(1)
                .set_cell(&mut store, 0, -1, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.ne(&mut store), expected);
        }

        #[test]
        fn nw() {
            let mut store = Store::new();
            let node = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.nw(&mut store), expected);
        }

        #[test]
        fn se() {
            let mut store = Store::new();
            let node = store
                .create_empty(1)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.se(&mut store), expected);
        }

        #[test]
        fn sw() {
            let mut store = Store::new();
            let node = store
                .create_empty(1)
                .set_cell(&mut store, -1, 0, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.sw(&mut store), expected);
        }

        #[test]
        fn center_subnode() {
            let mut store = Store::new();
            let node = store
                .create_empty(2)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(node.center_subnode(&mut store), expected);
        }
    }

    mod subsubnode {
        use super::*;

        #[test]
        fn north_subsubnode() {
            let mut store = Store::new();

            let node = store
                .create_empty(3)
                .set_cell(&mut store, -1, -2, Cell::Alive)
                .set_cell(&mut store, -1, -3, Cell::Alive)
                .set_cell(&mut store, 0, -2, Cell::Alive)
                .set_cell(&mut store, 0, -3, Cell::Alive);

            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(node.north_subsubnode(&mut store), expected);
        }

        #[test]
        fn south_subsubnode() {
            let mut store = Store::new();

            let node = store
                .create_empty(3)
                .set_cell(&mut store, -1, 1, Cell::Alive)
                .set_cell(&mut store, -1, 2, Cell::Alive)
                .set_cell(&mut store, 0, 1, Cell::Alive)
                .set_cell(&mut store, 0, 2, Cell::Alive);

            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(node.south_subsubnode(&mut store), expected);
        }

        #[test]
        fn east_subsubnode() {
            let mut store = Store::new();

            let node = store
                .create_empty(3)
                .set_cell(&mut store, 1, -1, Cell::Alive)
                .set_cell(&mut store, 1, 0, Cell::Alive)
                .set_cell(&mut store, 2, -1, Cell::Alive)
                .set_cell(&mut store, 2, 0, Cell::Alive);

            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(node.east_subsubnode(&mut store), expected);
        }

        #[test]
        fn west_subsubnode() {
            let mut store = Store::new();

            let node = store
                .create_empty(3)
                .set_cell(&mut store, -2, -1, Cell::Alive)
                .set_cell(&mut store, -2, 0, Cell::Alive)
                .set_cell(&mut store, -3, -1, Cell::Alive)
                .set_cell(&mut store, -3, 0, Cell::Alive);

            let expected = store
                .create_empty(1)
                .set_cell(&mut store, -1, -1, Cell::Alive)
                .set_cell(&mut store, -1, 0, Cell::Alive)
                .set_cell(&mut store, 0, -1, Cell::Alive)
                .set_cell(&mut store, 0, 0, Cell::Alive);

            assert_eq!(node.west_subsubnode(&mut store), expected);
        }
    }
}
