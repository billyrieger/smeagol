// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    rule::{Leaf, Rule},
    Grid2,
};

use generational_arena::{Arena, Index};

const MAX_SIDE_LEN: u64 = 1 << 63;
const LEAF_SIDE_LEN: u64 = 1 << 3;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Id {
    index: Index,
}

impl Id {
    fn new(index: Index) -> Self {
        Self { index }
    }
}

#[derive(Clone, Copy, Debug)]
struct Branch {
    children: Grid2<Id>,
    side_len: u64,
    population: u128,
}

enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

pub struct Store<R: Rule> {
    rule: R,
    nodes: Arena<Node>,
}

impl<R> Store<R>
where
    R: Rule,
{
    pub fn new(rule: R) -> Self {
        Self {
            rule,
            nodes: Arena::new(),
        }
    }

    pub fn init(&mut self) -> Id {
        let empty_leaf = Leaf::new([0; 8]);
        let empty_leaf_id = Id::new(self.nodes.insert(Node::Leaf(empty_leaf)));

        let mut empty_branch = Branch {
            children: Grid2([empty_leaf_id; 4]),
            side_len: LEAF_SIDE_LEN << 1,
            population: 0,
        };

        let mut id = empty_leaf_id;
        let mut side_len = LEAF_SIDE_LEN;
        loop {
            empty_branch = Branch {
                children: Grid2([id; 4]),
                side_len: empty_branch.side_len,
                population: 0,
            };
            id = Id::new(self.nodes.insert(Node::Branch(empty_branch)));
            if side_len == MAX_SIDE_LEN {
                return id;
            } else {
                side_len <<= 1;
            }
        }
    }
}
