// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    grid::{Grid2x2, Grid3x3, Grid4x4},
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

    pub fn make_branch(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        let nodes: Grid2x2<_> = children.try_map(|id| self.get_node(id))?;
        if let [a, b, c, d] = nodes.unpack() {
            let level = a.level();
            debug_assert_eq!(level, b.level());
            debug_assert_eq!(level, c.level());
            debug_assert_eq!(level, d.level());
            let population = a.population() + b.population() + c.population() + d.population();
            let node = Node::Branch(Branch {
                children,
                level,
                population,
            });
            Some(self.get_id(node))
        } else {
            None
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    pub fn recurse<F, G>(&mut self, children: Grid2x2<NodeId>, f: F, g: G) -> Option<NodeId>
    where
        F: Fn(&mut Self, Grid2x2<NodeId>) -> Option<NodeId>,
        G: Fn(&mut Self, Grid2x2<NodeId>) -> Option<NodeId>,
    {
        let nodes: Grid2x2<Node> = children.try_map(|id| self.get_node(id))?;
        match nodes.unpack() {
            &[Node::Leaf(w), Node::Leaf(x), Node::Leaf(y), Node::Leaf(z)] => {
                let grid2x2 = Grid2x2::pack(&[w, x, y, z]);
                self.make_leaf(grid2x2.jump(self.rule))
            }

            &[Node::Branch(w), Node::Branch(x), Node::Branch(y), Node::Branch(z)] => {
                let _: Grid2x2<_> = Grid2x2::pack(&[w, x, y, z]).map(|branch| branch.children);
                // let grid4x4 = Grid4x4::flatten(grandchildren);
                let grid4x4 = Grid4x4::default();
                let partial: Grid3x3<_> = grid4x4.shrink(|x| f(self, x))?;
                let grid2x2: Grid2x2<_> = partial.shrink(|x| g(self, x))?;
                self.make_branch(grid2x2)
            }

            _ => None,
        }
    }

    pub fn jump(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        self.recurse(children, Self::jump, Self::jump)
        // let nodes: Grid2x2<_> = children.try_map(|id| self.get_node(id))?;
        // match nodes.unpack() {
        //     [Node::Leaf(w), Node::Leaf(x), Node::Leaf(y), Node::Leaf(z)] => {
        //         let grid2x2 = Grid2x2::pack(&[*w, *x, *y, *z]);
        //         self.make_leaf(grid2x2.jump(self.rule))
        //     }

        //     [Node::Branch(w), Node::Branch(x), Node::Branch(y), Node::Branch(z)] => {
        //         let _: Grid2x2<_> = Grid2x2::pack(&[*w, *x, *y, *z]).map(|branch| branch.children);
        //         // let grid4x4 = Grid4x4::flatten(grandchildren);
        //         let grid4x4 = Grid4x4::default();
        //         let partial: Grid3x3<_> = grid4x4.reduce(|x| self.jump(x))?;
        //         let grid2x2: Grid2x2<_> = partial.reduce(|x| self.jump(x))?;
        //         self.make_inner(grid2x2)
        //     }

        //     _ => None,
        // }
    }

    fn get_id(&mut self, node: Node) -> NodeId {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.id_lookup.insert(node, id);
            id
        })
    }
}
