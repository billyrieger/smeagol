use crate::{node::NodeBase, Node, Store};

impl Node {
    pub fn ne(&self, store: &Store) -> Node {
        assert!(self.level > 0);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { ne_index, .. } => store.node(ne_index),
        }
    }

    pub fn nw(&self, store: &Store) -> Node {
        assert!(self.level > 0);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { nw_index, .. } => store.node(nw_index),
        }
    }

    pub fn se(&self, store: &Store) -> Node {
        assert!(self.level > 0);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { se_index, .. } => store.node(se_index),
        }
    }

    pub fn sw(&self, store: &Store) -> Node {
        assert!(self.level > 0);
        match self.base {
            NodeBase::Leaf { .. } => unreachable!(),
            NodeBase::Interior { sw_index, .. } => store.node(sw_index),
        }
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
            let node = store.create_empty(1).set_cell(&mut store, 0, -1, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.ne(&mut store), expected);
        }

        #[test]
        fn nw() {
            let mut store = Store::new();
            let node = store.create_empty(1).set_cell(&mut store, -1, -1, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.nw(&mut store), expected);
        }

        #[test]
        fn se() {
            let mut store = Store::new();
            let node = store.create_empty(1).set_cell(&mut store, 0, 0, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.se(&mut store), expected);
        }

        #[test]
        fn sw() {
            let mut store = Store::new();
            let node = store.create_empty(1).set_cell(&mut store, -1, 0, Cell::Alive);

            let expected = store.create_leaf(Cell::Alive);

            assert_eq!(node.sw(&mut store), expected);
        }
    }
}
