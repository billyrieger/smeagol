mod create;

use crate::Node;

pub struct NodeTemplate {
    pub ne: Node,
    pub nw: Node,
    pub se: Node,
    pub sw: Node,
}

#[derive(Clone, Debug)]
pub struct Store {
    nodes: Vec<Node>,
    populations: Vec<u128>,
    indices: hashbrown::HashMap<Node, usize>,
    level_2_steps: hashbrown::HashMap<Node, usize>,
    steps: hashbrown::HashMap<(Node, u64), usize>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            populations: vec![],
            indices: hashbrown::HashMap::default(),
            level_2_steps: hashbrown::HashMap::default(),
            steps: hashbrown::HashMap::default(),
        }
    }

    pub(crate) fn population(&self, node: &Node) -> u128 {
        self.populations[node.index()]
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
        self.steps.insert((node, step_size), result.index());
    }

    pub(crate) fn level_2_step(&self, node: Node) -> Option<Node> {
        self.level_2_steps
            .get(&node)
            .map(|&index| self.nodes[index])
    }

    pub(crate) fn add_level_2_step(&mut self, node: Node, result: Node) {
        self.level_2_steps.insert(node, result.index());
    }
}
