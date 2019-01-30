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
    steps: hashbrown::HashMap<(Node, u64), usize>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            indices: hashbrown::HashMap::new(),
            steps: hashbrown::HashMap::new(),
        }
    }

    pub(crate) fn node(&self, index: usize) -> Node {
        self.nodes[index]
    }

    pub(crate) fn step(&self, node: Node, step_size: u64) -> Option<Node> {
        self.steps
            .get(&(node, step_size))
            .map(|&index| self.nodes[index])
    }

    pub(crate) fn add_step(&mut self, node: Node, step_size: u64, result: Node) {
        self.steps.insert((node, step_size), self.indices[&result]);
    }
}
