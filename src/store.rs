// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    grid::Grid2,
    node::{Branch, Leaf, Node, NodeId},
    Rule,
};
use slotmap::{SecondaryMap, SlotMap};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Quadtree {
    rule: Rule,
    id_lookup: HashMap<Node, NodeId>,
    nodes: SlotMap<NodeId, Node>,
    steps: SecondaryMap<NodeId, NodeId>,
    jumps: SecondaryMap<NodeId, NodeId>,
}

impl Quadtree {
    pub fn make_leaf(&mut self, leaf: Leaf) -> Option<NodeId> {
        Some(self.get_id(Node::Leaf(leaf)))
    }

    pub fn make_branch(&mut self, children: Grid2<NodeId>) -> Option<NodeId> {
        let Grid2(nodes) = children.try_map(|id| self.get_node(id))?;
        let level = nodes[0].level().increment()?;
        let population = nodes.iter().map(Node::population).sum();
        let node = Node::Branch(Branch {
            children,
            level,
            population,
        });
        Some(self.get_id(node))
    }

    pub fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    pub fn recurse<F, G>(&mut self, children: Grid2<NodeId>, f: F, g: G) -> Option<NodeId>
    where
        F: Fn(&mut Self, Grid2<NodeId>) -> Option<NodeId>,
        G: Fn(&mut Self, Grid2<NodeId>) -> Option<NodeId>,
    {
        let Grid2(nodes) = children.try_map(|id| self.get_node(id))?;
        match nodes {
            [Node::Leaf(w), Node::Leaf(x), Node::Leaf(y), Node::Leaf(z)] => {
                let grid2x2 = Grid2([w, x, y, z]);
                self.make_leaf(grid2x2.jump(self.rule))
            }

            [Node::Branch(w), Node::Branch(x), Node::Branch(y), Node::Branch(z)] => {
                let grandchildren = Grid2([w, x, y, z]).map(|branch| branch.children).flatten();
                let shrunk = grandchildren
                    .shrink(|x| f(self, x))?
                    .shrink(|x| g(self, x))?;
                self.make_branch(shrunk)
            }

            _ => None,
        }
    }

    pub fn jump(&mut self, children: Grid2<NodeId>) -> Option<NodeId> {
        self.recurse(children, Self::jump, Self::jump)
    }

    fn get_id(&mut self, node: Node) -> NodeId {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.id_lookup.insert(node, id);
            id
        })
    }
}
