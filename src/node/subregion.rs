use crate::{node::NodeBase, Node, NodeTemplate, Store};

impl Node {
    pub fn ne(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { ne_index, .. } => store.node(ne_index),
        }
    }

    pub fn nw(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { nw_index, .. } => store.node(nw_index),
        }
    }

    pub fn se(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { se_index, .. } => store.node(se_index),
        }
    }

    pub fn sw(&self, store: &Store) -> Node {
        assert!(self.level >= 1);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { sw_index, .. } => store.node(sw_index),
        }
    }

    pub fn center_subnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 2);

        let ne = self.ne(store).sw(store);
        let nw = self.nw(store).se(store);
        let se = self.se(store).nw(store);
        let sw = self.sw(store).ne(store);

        store.create_interior(NodeTemplate { ne, nw, se, sw })
    }

    pub fn north_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let e = self.ne(store);
        let w = self.nw(store);

        Self::centered_horiz(store, e, w)
    }

    pub fn south_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let e = self.se(store);
        let w = self.sw(store);

        Self::centered_horiz(store, e, w)
    }

    pub fn east_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let n = self.ne(store);
        let s = self.se(store);

        Self::centered_vert(store, n, s)
    }

    pub fn west_subsubnode(&self, store: &mut Store) -> Node {
        assert!(self.level >= 3);

        let n = self.nw(store);
        let s = self.sw(store);

        Self::centered_vert(store, n, s)
    }

    fn centered_horiz(store: &mut Store, e: Node, w: Node) -> Node {
        assert_eq!(e.level, w.level);
        assert!(e.level >= 2);

        let ne = e.nw(store).sw(store);
        let nw = w.ne(store).se(store);
        let se = e.sw(store).nw(store);
        let sw = w.se(store).ne(store);

        store.create_interior(NodeTemplate { ne, nw, se, sw })
    }

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
