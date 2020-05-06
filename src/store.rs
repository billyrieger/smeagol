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
pub struct Store {
    rule: Rule,
    id_lookup: HashMap<Node, NodeId>,
    nodes: SlotMap<NodeId, Node>,
    steps: SecondaryMap<NodeId, NodeId>,
    jumps: SecondaryMap<NodeId, NodeId>,
}

impl Store {
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

    pub fn recurse<B, F, G>(
        &mut self,
        children: Grid2<NodeId>,
        base_case: B,
        f: F,
        g: G,
    ) -> Option<NodeId>
    where
        B: Fn(Grid2<Leaf>) -> Leaf,
        F: Fn(&mut Self, Grid2<NodeId>) -> Option<NodeId>,
        G: Fn(&mut Self, Grid2<NodeId>) -> Option<NodeId>,
    {
        let Grid2(nodes) = children.try_map(|id| self.get_node(id))?;
        match nodes {
            [Node::Leaf(a), Node::Leaf(b), Node::Leaf(c), Node::Leaf(d)] => {
                let leaves = Grid2([a, b, c, d]);
                self.make_leaf(base_case(leaves))
            }

            [Node::Branch(a), Node::Branch(b), Node::Branch(c), Node::Branch(d)] => {
                let branches = Grid2([a, b, c, d]);
                let grandchildren = branches.map(|branch| branch.children).flatten();
                let shrunk = grandchildren
                    .shrink(|x| f(self, x))?
                    .shrink(|x| g(self, x))?;
                self.make_branch(shrunk)
            }

            _ => None,
        }
    }

    pub fn jump(&mut self, id: NodeId) -> Option<NodeId> {
        let children = self.get_node(id)?.children()?;
        self.jump_helper(children)
    }

    fn jump_helper(&mut self, children: Grid2<NodeId>) -> Option<NodeId> {
        let rule = self.rule;
        let base_case = |leaves: Grid2<Leaf>| leaves.jump(rule);
        self.recurse(children, base_case, Self::jump_helper, Self::jump_helper)
    }

    fn get_id(&mut self, node: Node) -> NodeId {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.id_lookup.insert(node, id);
            id
        })
    }
}
