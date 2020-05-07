// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    grid::Grid2,
    node::{Branch, Id, Leaf, Node},
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

//     fn idle(&mut self, _ids: Grid2<Id>) -> Option<Id> {
//         todo!()
//     }

//     fn jump(&mut self, id: Id) -> Option<Id> {
//         use Node::*;

//         self.jumps
//             .get(id)
//             .copied()
//             .or_else(|| match self.get_node(id)? {
//                 Leaf(_) => None,
//                 Branch(branch) => match branch.children.try_map(|id| self.get_node(id))? {
//                     Grid2([Leaf(a), Leaf(b), Leaf(c), Leaf(d)]) => {
//                         let leaves = Grid2([a, b, c, d]);
//                         todo!();
//                     }

//                     Grid2([Branch(a), Branch(b), Branch(c), Branch(d)]) => {
//                         todo!();
//                     }

//                     _ => None,
//                 },
//             })
//     }

//     pub fn evolve(&mut self, id: Id, steps: u64) -> Option<Id> {
//         use Node::{Branch, Leaf};

//         match self.get_node(id)? {
//             Leaf(_) => todo!(),
//             Branch(branch) => {
//                 let max_steps = branch.level.max_steps();
//                 let Grid2(child_nodes) = branch.children.try_map(|id| self.get_node(id))?;
//                 match child_nodes {
//                     [Node::Leaf(a), Node::Leaf(b), Node::Leaf(c), Node::Leaf(d)] => {
//                         let child_leaves = Grid2([a, b, c, d]);
//                         let new_leaf = match steps {
//                             0 => Some(child_leaves.idle_idle(self.rule)),
//                             1 => Some(child_leaves.idle_step(self.rule)),
//                             2 => Some(child_leaves.idle_jump(self.rule)),
//                             3 => Some(child_leaves.step_jump(self.rule)),
//                             4 => Some(child_leaves.jump_jump(self.rule)),
//                             _ => None,
//                         }?;
//                         self.make_leaf(new_leaf);
//                         todo!()
//                     }

//                     [Node::Branch(a), Node::Branch(b), Node::Branch(c), Node::Branch(d)] => {
//                         let result = if steps <= max_steps / 2 {
//                             todo!()
//                         } else if steps <= max_steps {
//                             todo!()
//                         } else {
//                             None
//                         }?;
//                         self.make_branch(result);
//                         todo!()
//                     }

//                     _ => todo!(),
//                 }
//             }
//         };
//         todo!()
//     }

    fn get_id(&mut self, node: Node) -> Id {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.id_lookup.insert(node, id);
            id
        })
    }
}
