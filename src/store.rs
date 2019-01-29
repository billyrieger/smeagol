use crate::{Cell, Node};

pub struct Store {
    nodes: Vec<Node>,
    indices: hashbrown::HashMap<Node, usize>,
}

impl Store {
    pub(crate) fn node(&self, index: usize) -> Node {
        self.nodes[index]
    }

    fn add_node(&mut self, node: Node) {
        if !self.indices.contains_key(&node) {
            let index = self.nodes.len();
            self.nodes.push(node);
            self.indices.insert(node, index);
        }
    }
}
