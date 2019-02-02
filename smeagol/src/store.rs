mod create;

use crate::Node;

/// A template to create a node from four children nodes.
///
/// Used by [`Store::create_interior`](Store::create_interior).
pub struct NodeTemplate {
    /// The northeast child node.
    pub ne: Node,
    /// The northwest child node.
    pub nw: Node,
    /// The southeast child node.
    pub se: Node,
    /// The southwest child node.
    pub sw: Node,
}

#[derive(Clone, Debug)]
pub struct Store {
    nodes: Vec<Node>,
    populations: hashbrown::HashMap<Node, u128>,
    indices: hashbrown::HashMap<Node, usize>,
    level_2_steps: hashbrown::HashMap<Node, usize>,
    steps: hashbrown::HashMap<(u64, Node), usize>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            populations: hashbrown::HashMap::default(),
            indices: hashbrown::HashMap::default(),
            level_2_steps: hashbrown::HashMap::default(),
            steps: hashbrown::HashMap::default(),
        }
    }

    pub(crate) fn population(&mut self, node: &Node) -> u128 {
        if let Some(pop) = self.populations.get(node).cloned() {
            pop
        } else {
            let pop = if node.level() == 0 {
                if node.contains_alive_cells() {
                    1
                } else {
                    0
                }
            } else {
                if node.contains_alive_cells() {
                    self.population(&node.ne(&self)) + 
                    self.population(&node.nw(&self)) + 
                    self.population(&node.se(&self)) + 
                    self.population(&node.sw(&self))
                } else {
                    0
                }
            };
            self.populations.insert(*node, pop);
            pop
        }
    }

    pub(crate) fn node(&self, index: usize) -> Node {
        self.nodes[index]
    }

    pub(crate) fn step(&self, node: Node, step_size: u64) -> Option<Node> {
        self.steps
            .get(&(step_size, node))
            .map(|&index| self.nodes[index])
    }

    pub(crate) fn add_step(&mut self, node: Node, step_size: u64, result: Node) {
        self.steps.insert((step_size, node), self.indices[&result]);
    }

    pub(crate) fn level_2_step(&self, node: Node) -> Option<Node> {
        self.level_2_steps
            .get(&node)
            .map(|&index| self.nodes[index])
    }

    pub(crate) fn add_level_2_step(&mut self, node: Node, result: Node) {
        self.level_2_steps.insert(node, self.indices[&result]);
    }
}
