// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    grid::Grid2,
    node::{Branch, Id, Leaf, Level, Node},
    Rule,
};
use slotmap::{SecondaryMap, SlotMap};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Store {
    rule: Rule,
    id_lookup: HashMap<Node, Id>,
    nodes: SlotMap<Id, Node>,
    steps: SecondaryMap<Id, Id>,
    jumps: SecondaryMap<Id, Id>,
}

impl Store {
    pub fn make_leaf(&mut self, leaf: Leaf) -> Option<Id> {
        Some(self.get_id(Node::Leaf(leaf)))
    }

    pub fn make_branch(&mut self, children: Grid2<Id>) -> Option<Id> {
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

    pub fn get_node(&self, id: Id) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    fn idle(&mut self, _ids: Grid2<Id>) -> Option<Id> {
        todo!()
    }

    pub fn evolve(&mut self, ids: Grid2<Id>, steps: u64) -> Option<Id> {
        let Grid2(nodes) = ids.try_map(|id| self.get_node(id))?;
        match nodes {
            [Node::Leaf(a), Node::Leaf(b), Node::Leaf(c), Node::Leaf(d)] => {
                let leaves = Grid2([a, b, c, d]);
                self.make_leaf(leaves.evolve(self.rule, steps))
            }

            [Node::Branch(a), Node::Branch(b), Node::Branch(c), Node::Branch(d)] => {
                let max_steps = a.level.max_steps();
                let branches = Grid2([a, b, c, d]);
                let grandchildren = branches.map(|branch| branch.children).flatten();

                let result = if steps <= max_steps / 2 {
                    grandchildren
                        .shrink(|ids| self.idle(ids))?
                        .shrink(|ids| self.evolve(ids, steps))?
                } else if steps <= max_steps {
                    let leftover_steps = steps - max_steps / 2;
                    grandchildren
                        .shrink(|ids| self.evolve(ids, leftover_steps))?
                        .shrink(|ids| self.evolve(ids, max_steps / 2))?
                } else {
                    None?
                };
                self.make_branch(result)
            }

            _ => None,
        };
        todo!()
    }

    fn get_id(&mut self, node: Node) -> Id {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.id_lookup.insert(node, id);
            id
        })
    }
}
