use crate::Node;
use crate::Cell;

pub struct Store {
    nodes: Vec<Node>,
    indices: hashbrown::HashMap<Node, usize>,
}

impl Store {
    pub(crate) fn node(&self, index: usize) -> Node {
        self.nodes[index]
    }

    pub fn create_leaf(&mut self, cell: Cell) -> Node {
        let node = Node::new_leaf(cell);
        self.add_node(node);
        node
    }

    fn add_node(&mut self, node: Node) {
        if !self.indices.contains_key(&node) {
            let index = self.nodes.len();
            self.nodes.push(node);
            self.indices.insert(node, index);
        }
    }
}
