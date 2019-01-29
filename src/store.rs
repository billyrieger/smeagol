mod create;

use crate::Node;

pub struct NodeTemplate {
    pub ne: Node,
    pub nw: Node,
    pub se: Node,
    pub sw: Node,
}

pub struct Store {
    nodes: Vec<Node>,
    indices: hashbrown::HashMap<Node, usize>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            indices: hashbrown::HashMap::new(),
        }
    }

    pub(crate) fn node(&self, index: usize) -> Node {
        self.nodes[index]
    }
}
