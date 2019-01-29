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
