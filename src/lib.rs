// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(iter_partition_in_place)]
#![allow(dead_code, unused_variables)]

mod life;

use life::{Clover, Leaf, Rule, B3S23};

use std::fmt;

use indexmap::{indexmap, indexset, IndexMap, IndexSet};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coords {
    x: i64,
    y: i64,
}

impl Coords {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    fn offset(&self, dx: i64, dy: i64) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    fn relative_to(&self, other: Self) -> Self {
        self.offset(-other.x, -other.y)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Branch {
    nw: Id,
    ne: Id,
    sw: Id,
    se: Id,
}

impl Branch {
    const MIN_LEVEL: usize = 4;
    const MAX_LEVEL: usize = 63;
    const MIN_COORD: i64 = -(1 << (Self::MAX_LEVEL - 1));
    const MAX_COORD: i64 = (1 << (Self::MAX_LEVEL - 1)) - 1;

    fn new(nw: Id, ne: Id, sw: Id, se: Id) -> Self {
        debug_assert_eq!(nw.level(), ne.level());
        debug_assert_eq!(nw.level(), sw.level());
        debug_assert_eq!(nw.level(), se.level());
        Self { nw, ne, sw, se }
    }

    fn repeat(id: Id) -> Self {
        Self::new(id, id, id, id)
    }

    const fn level(&self) -> usize {
        self.nw.level() + 1
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    data: u64,
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Id")
            .field("index", &self.index())
            .field("level", &self.level())
            .finish()
    }
}

impl Id {
    fn new(index: usize, level: usize) -> Self {
        debug_assert!(Leaf::level() <= level);
        debug_assert!(level <= Branch::MAX_LEVEL);
        let data = ((index as u64) << 8) | ((level as u64) & 0xFF);
        Self { data }
    }

    const fn index(&self) -> usize {
        (self.data >> 8) as usize
    }

    const fn level(&self) -> usize {
        (self.data & 0xFF) as usize
    }
}

#[derive(Clone, Debug)]
struct Data;

#[derive(Clone, Debug)]
pub struct Universe<R = B3S23> {
    base: IndexSet<Leaf>,
    levels: Vec<IndexMap<Branch, Data>>,
    rule: R,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Universe {
    fn new() -> Self {
        let base = indexset! { Leaf::empty() };
        let mut levels: Vec<IndexMap<Branch, Data>> = vec![];

        for level in Branch::MIN_LEVEL..=Branch::MAX_LEVEL {
            let prev = Id::new(0, level - 1);
            let empty = Branch::repeat(prev);
            levels.push(indexmap! { empty => Data, });
        }

        Self {
            base,
            levels,
            rule: B3S23,
        }
    }

    fn create_leaf(&mut self, leaf: Leaf) -> Id {
        let index = self.base.insert_full(leaf).0;
        Id::new(index, Leaf::level())
    }

    fn create_branch(&mut self, branch: Branch) -> Id {
        let index = self.levels[branch.level() - Leaf::level() - 1]
            .insert_full(branch, Data)
            .0;

        Id::new(index, branch.level())
    }

    fn get(&self, id: Id) -> Node {
        if id.level() == Leaf::level() {
            Node::Leaf(self.get_leaf(id))
        } else {
            Node::Branch(self.get_branch(id))
        }
    }

    fn get_leaf(&self, id: Id) -> Leaf {
        *self.base.get_index(id.index()).expect("invalid index")
    }

    fn get_branch(&self, id: Id) -> Branch {
        *self.levels[id.level() - Leaf::level() - 1]
            .get_index(id.index())
            .expect("invalid index")
            .0
    }

    fn set_cells(&mut self, id: Id, center: Coords, coords: &mut [Coords]) -> Id {
        if coords.is_empty() {
            return id;
        }

        if id.level() == Leaf::level() {
            let mut leaf = self.get_leaf(id);
            for pos in coords {
                leaf = leaf.set_cell(pos.relative_to(center));
            }
            self.create_leaf(leaf)
        } else {
            let branch = self.get_branch(id);
            let delta: i64 = 1 << (branch.level() - 2);

            let center_nw = center.offset(-delta, -delta);
            let center_ne = center.offset(delta, -delta);
            let center_sw = center.offset(-delta, delta);
            let center_se = center.offset(delta, delta);

            let (coords_north, coords_south) = partition(coords, |pos| pos.y < center.y);
            let (coords_nw, coords_ne) = partition(coords_north, |pos| pos.x < center.x);
            let (coords_sw, coords_se) = partition(coords_south, |pos| pos.x < center.x);

            let nw = self.set_cells(branch.nw, center_nw, coords_nw);
            let ne = self.set_cells(branch.ne, center_ne, coords_ne);
            let sw = self.set_cells(branch.sw, center_sw, coords_sw);
            let se = self.set_cells(branch.se, center_se, coords_se);
            let branch = Branch::new(nw, ne, sw, se);
            self.create_branch(branch)
        }
    }

    fn evolve(&mut self, id: Id, generations: u64) -> Id {
        let parent = self.get_branch(id);
        match (
            self.get(parent.nw),
            self.get(parent.ne),
            self.get(parent.sw),
            self.get(parent.se),
        ) {
            (Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)) => {
                let mut clover = Clover::new(nw, ne, sw, se);
                assert!(generations <= 4); // log_2 16 is the max for 16x16 grid
                for _ in 0..generations {
                    clover = self.rule.step(clover);
                }
                self.create_leaf(clover.center())
            }

            (Node::Branch(nw), Node::Branch(ne), Node::Branch(sw), Node::Branch(se)) => {
                let [a, b, c, d] = [nw.nw, nw.ne, ne.nw, ne.ne];
                let [e, f, g, h] = [nw.sw, nw.se, ne.sw, ne.se];
                let [i, j, k, l] = [sw.nw, sw.ne, se.nw, se.ne];
                let [m, n, o, p] = [sw.sw, sw.se, se.sw, se.se];

                let east = self.create_branch(Branch::new(e, f, i, j));
                let west = self.create_branch(Branch::new(g, h, k, l));
                let north = self.create_branch(Branch::new(b, c, f, g));
                let south = self.create_branch(Branch::new(j, k, n, o));

                todo!()
            }
            _ => panic!("invalid branch"),
        }
    }
}

fn partition<F>(coords: &mut [Coords], predicate: F) -> (&mut [Coords], &mut [Coords])
where
    F: FnMut(&Coords) -> bool,
{
    let split = coords.iter_mut().partition_in_place(predicate);
    coords.split_at_mut(split)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut universe = Universe::new();
        let mut coords: Vec<Coords> = vec![(7, 4), (8, 4), (9, 4)]
            .into_iter()
            .map(|(x, y)| Coords::new(x, y))
            .collect();
        let blinker = universe.set_cells(
            Id::new(0, Branch::MAX_LEVEL),
            Coords::new(0, 0),
            &mut coords,
        );

        let next = universe.evolve(blinker, 1);
        dbg!(&universe);
        dbg!(next);
    }
}
