use crate::node::Index;
use crate::node::Level;
use crate::node::Node;
use crate::node::NodeId;
use packed_simd::u16x16;

pub struct NodeTemplate {
    pub nw: NodeId,
    pub ne: NodeId,
    pub sw: NodeId,
    pub se: NodeId,
}

#[derive(Clone)]
pub struct Store {
    indices: hashbrown::HashMap<Node, NodeId>,
    nodes: Vec<Node>,
    steps: Vec<Option<NodeId>>,
    jumps: Vec<Option<NodeId>>,
    step_log_2: u8,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            indices: hashbrown::HashMap::new(),
            nodes: vec![],
            steps: vec![],
            jumps: vec![],
            step_log_2: 0,
        }
    }

    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.index.0 as usize]
    }

    pub fn create_leaf(&mut self, grid: u16x16) -> NodeId {
        let node = Node::Leaf { grid };
        self.add_node(node)
    }

    pub fn create_interior(&mut self, template: NodeTemplate) -> NodeId {
        let level = template.nw.level(self);
        let new_level = Level(level.0 + 1);

        let population = template.nw.population(self)
            + template.ne.population(self)
            + template.sw.population(self)
            + template.se.population(self);

        let node = Node::Interior {
            nw: template.nw,
            ne: template.ne,
            sw: template.sw,
            se: template.se,
            level: new_level,
            population,
        };

        self.add_node(node)
    }

    pub fn create_empty(&mut self, level: Level) -> NodeId {
        if level == Level(4) {
            self.create_leaf(u16x16::splat(0))
        } else {
            let empty = self.create_empty(Level(level.0 - 1));
            self.create_interior(NodeTemplate {
                nw: empty,
                ne: empty,
                sw: empty,
                se: empty,
            })
        }
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        if let Some(id) = self.indices.get(&node) {
            *id
        } else {
            let id = NodeId {
                index: Index(self.nodes.len() as u32),
            };
            self.indices.insert(node, id);
            self.nodes.push(node);
            self.steps.push(None);
            self.jumps.push(None);
            id
        }
    }
}

impl Store {
    pub fn step_log_2(&self) -> u8 {
        self.step_log_2
    }

    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        self.step_log_2 = step_log_2;
    }

    pub fn get_step(&self, id: NodeId) -> Option<NodeId> {
        self.steps[id.index.0 as usize]
    }

    pub fn add_step(&mut self, id: NodeId, step: NodeId) {
        self.steps[id.index.0 as usize] = Some(step);
    }

    pub fn get_jump(&self, id: NodeId) -> Option<NodeId> {
        self.jumps[id.index.0 as usize]
    }

    pub fn add_jump(&mut self, id: NodeId, jump: NodeId) {
        self.jumps[id.index.0 as usize] = Some(jump);
    }
}
