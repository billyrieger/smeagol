/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::node::{Index, Level, Node, NodeId};
use packed_simd::u16x16;

/// A template to create a node from four child nodes.
pub struct NodeTemplate {
    /// The northwest child.
    pub nw: NodeId,
    /// The northeast child.
    pub ne: NodeId,
    /// The southwest child.
    pub sw: NodeId,
    /// The southeast child.
    pub se: NodeId,
}

/// A struct to store nodes and node evolution results.
#[derive(Clone, Debug)]
pub struct Store {
    indices: hashbrown::HashMap<Node, NodeId>,
    nodes: Vec<Node>,
    steps: Vec<Option<NodeId>>,
    jumps: Vec<Option<NodeId>>,
    step_log_2: u8,
}

impl Store {
    /// Creates a new empty store.
    ///
    /// # Examples
    ///
    /// ```
    /// let store = smeagol::node::Store::new();
    /// ```
    pub fn new() -> Self {
        Self {
            indices: hashbrown::HashMap::new(),
            nodes: vec![],
            steps: vec![],
            jumps: vec![],
            step_log_2: 0,
        }
    }

    /// Returns the node corresponding to the given node ID.
    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.index.0 as usize]
    }

    /// Creates a leaf node corresponding to the given 16 by 16 grid.
    pub fn create_leaf(&mut self, grid: u16x16) -> NodeId {
        let node = Node::Leaf { grid };
        self.add_node(node)
    }

    /// Creates an interior node from the given node template.
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

    /// Creates an empty node with the given level.
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

    /// Adds a node to the store, returning a node ID.
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
    /// Returns the current step size log 2.
    pub fn step_log_2(&self) -> u8 {
        self.step_log_2
    }

    /// Sets the step size to be `2^step_log_2`.
    ///
    /// This clears previously calculated steps.
    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        if step_log_2 != self.step_log_2 {
            self.step_log_2 = step_log_2;
            self.steps = vec![None; self.steps.len()]
        }
    }

    /// Gets the step of the given node, if it has been previously calculated.
    pub fn get_step(&self, id: NodeId) -> Option<NodeId> {
        self.steps[id.index.0 as usize]
    }

    /// Sets the step of the given node.
    pub fn add_step(&mut self, id: NodeId, step: NodeId) {
        self.steps[id.index.0 as usize] = Some(step);
    }

    /// Gets the jump of the given node, if it has been previously calculated.
    pub fn get_jump(&self, id: NodeId) -> Option<NodeId> {
        self.jumps[id.index.0 as usize]
    }

    /// Sets the jump of the given node.
    pub fn add_jump(&mut self, id: NodeId, jump: NodeId) {
        self.jumps[id.index.0 as usize] = Some(jump);
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let _store = Store::default();
    }
}
