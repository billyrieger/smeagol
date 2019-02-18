mod create;

use crate::node::Node;

/// A template to create a node from four children nodes.
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

/// A structure to contain and create nodes.
#[derive(Clone, Debug)]
pub struct Store {
    nodes: Vec<Node>,
    populations: Vec<u128>,
    indices: hashbrown::HashMap<Node, usize>,
    level_2_steps: hashbrown::HashMap<Node, Node>,
    steps: hashbrown::HashMap<(Node, u64), usize>,
}

/// Methods to create a store.
impl Store {
    /// Creates a new empty store.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::node::Store::new();
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            populations: vec![],
            indices: hashbrown::HashMap::default(),
            level_2_steps: hashbrown::HashMap::default(),
            steps: hashbrown::HashMap::default(),
        }
    }
}

/// Internal methods.
impl Store {
    /// Returns the stored population of the given node.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is less than 2, since those nodes are not stored.
    pub(crate) fn population(&self, node: &Node) -> u128 {
        self.populations[node.index()]
    }

    /// Returns the index of the given node.
    ///
    /// # Panics
    ///
    /// Panics if the level of the node is less than 2, since those nodes are not stored.
    pub fn node(&self, index: usize) -> Node {
        self.nodes[index]
    }

    pub fn step(&self, node: Node, step_size: u64) -> Option<Node> {
        self.steps
            .get(&(node, step_size))
            .map(|&index| self.nodes[index])
    }

    pub fn add_step(&mut self, node: Node, step_size: u64, result: Node) {
        self.steps.insert((node, step_size), result.index());
    }

    pub fn level_2_step(&self, node: Node) -> Option<Node> {
        self.level_2_steps.get(&node).cloned()
    }

    pub fn add_level_2_step(&mut self, node: Node, result: Node) {
        self.level_2_steps.insert(node, result);
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}
