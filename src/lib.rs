// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(iter_partition_in_place)]
#![allow(dead_code, unused_variables)]

mod life;
mod util;

use life::{Clover, Leaf, Rule, B3S23};
use util::Grid2;

use std::fmt;

use indexmap::{indexmap, indexset, IndexMap, IndexSet};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coords {
    pub x: i64,
    pub y: i64,
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
        assert!(Leaf::level() <= level);
        assert!(level <= Branch::MAX_LEVEL);
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BranchData {
    is_empty: bool,
    jump: Option<Id>,
}

impl BranchData {
    fn new() -> Self {
        Self {
            is_empty: true,
            jump: None,
        }
    }

    fn non_empty() -> Self {
        Self {
            is_empty: false,
            jump: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Branch {
    children: Grid2<Id>,
    is_empty: bool,
    jump: Option<Id>,
}

impl Branch {
    const MIN_LEVEL: usize = 4;
    const MAX_LEVEL: usize = 63;
    const MIN_COORD: i64 = -(1 << (Self::MAX_LEVEL - 1));
    const MAX_COORD: i64 = (1 << (Self::MAX_LEVEL - 1)) - 1;

    fn new(level: usize) -> Self {
        Self {
            children: Grid2::repeat(Id::new(0, level - 1)),
            is_empty: true,
            jump: None,
        }
    }

    fn level(&self) -> usize {
        self.children.nw.level() + 1
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    fn is_empty(&self) -> bool {
        match self {
            Node::Leaf(leaf) => leaf.is_empty(),
            Node::Branch(branch) => branch.is_empty,
        }
    }

    fn level(&self) -> usize {
        match self {
            Node::Leaf(leaf) => Leaf::level(),
            Node::Branch(branch) => branch.level(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Universe<R = B3S23> {
    base: IndexSet<Leaf>,
    levels: Vec<IndexMap<Grid2<Id>, BranchData>>,
    rule: R,
}

impl Universe {
    fn new() -> Self {
        let base = indexset! { Leaf::empty() };
        let mut levels: Vec<IndexMap<Grid2<Id>, BranchData>> = vec![];

        for level in Branch::MIN_LEVEL..=Branch::MAX_LEVEL {
            let empty = Grid2::repeat(Id::new(0, level - 1));
            levels.push(indexmap! { empty => BranchData::new(), });
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

    fn create_branch(&mut self, ids: Grid2<Id>) -> Id {
        let children: Grid2<Node> = ids.map(|id| self.get(id));
        let is_empty = children.nw.is_empty()
            && children.ne.is_empty()
            && children.sw.is_empty()
            && children.se.is_empty();

        let mut data = BranchData::new();
        data.is_empty = is_empty;

        let level = children.nw.level() + 1;
        let index = self.levels[level - Leaf::level() - 1]
            .insert_full(ids, data)
            .0;

        Id::new(index, level)
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
        let (children, data) = self.levels[id.level() - Leaf::level() - 1]
            .get_index(id.index())
            .expect("invalid index");
        Branch {
            children: *children,
            is_empty: data.is_empty,
            jump: data.jump,
        }
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

            let nw = self.set_cells(branch.children.nw, center_nw, coords_nw);
            let ne = self.set_cells(branch.children.ne, center_ne, coords_ne);
            let sw = self.set_cells(branch.children.sw, center_sw, coords_sw);
            let se = self.set_cells(branch.children.se, center_se, coords_se);
            self.create_branch(Grid2::new([nw, ne, sw, se]))
        }
    }

    fn idle(&mut self, id: Id) -> Id {
        let parent = self.get_branch(id);

        if parent.level() == Leaf::level() + 1 {
            let leaves: Grid2<Leaf> = parent.children.map(|id| self.get_leaf(id));
            self.create_leaf(Clover::new(leaves).center())
        } else {
            let grid: Grid2<Grid2<Id>> = parent.children.map(|id| self.get_branch(id).children);
            self.create_branch([grid.nw.se, grid.ne.sw, grid.sw.ne, grid.se.nw].into())
        }
    }

    fn evolve(&mut self, id: Id, generations: u64) -> Id {
        let parent = self.get_branch(id);

        if generations == 0 || parent.is_empty {
            return self.idle(id);
        }

        if parent.level() == Leaf::level() + 1 {
            let leaves: Grid2<Leaf> = parent.children.map(|id| self.get_leaf(id));
            let mut clover = Clover::new(leaves);
            for _ in 0..generations {
                clover = self.rule.step(clover);
            }
            self.create_leaf(clover.center())
        } else {
            let grid: Grid2<Grid2<Id>> = parent.children.map(|id| self.get_branch(id).children);

            let [a, b, c, d] = [grid.nw.nw, grid.nw.ne, grid.ne.nw, grid.ne.ne];
            let [e, f, g, h] = [grid.nw.sw, grid.nw.se, grid.ne.sw, grid.ne.se];
            let [i, j, k, l] = [grid.sw.nw, grid.sw.ne, grid.se.nw, grid.se.ne];
            let [m, n, o, p] = [grid.sw.sw, grid.sw.se, grid.se.sw, grid.se.se];

            let northwest = parent.children.nw;
            let northeast = parent.children.ne;
            let southwest = parent.children.sw;
            let southeast = parent.children.se;

            let west = self.create_branch([e, f, i, j].into());
            let east = self.create_branch([g, h, k, l].into());
            let north = self.create_branch([b, c, f, g].into());
            let south = self.create_branch([j, k, n, o].into());

            let center = self.create_branch([f, g, j, k].into());

            // q r s
            // t u v
            // w x y
            let q = self.evolve(northwest, 0);
            let r = self.evolve(north, 0);
            let s = self.evolve(northeast, 0);
            let t = self.evolve(east, 0);
            let u = self.evolve(center, 0);
            let v = self.evolve(west, 0);
            let w = self.evolve(southwest, 0);
            let x = self.evolve(south, 0);
            let y = self.evolve(southeast, 0);

            let nw_partial = self.create_branch([q, r, t, u].into());
            let ne_partial = self.create_branch([r, s, u, v].into());
            let sw_partial = self.create_branch([t, u, w, x].into());
            let se_partial = self.create_branch([u, v, x, y].into());

            let nw = self.evolve(nw_partial, generations);
            let ne = self.evolve(ne_partial, generations);
            let sw = self.evolve(sw_partial, generations);
            let se = self.evolve(se_partial, generations);

            self.create_branch([nw, ne, sw, se].into())
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
        let mut coords: Vec<Coords> = vec![(4, 4), (5, 4), (6, 4), (6, 3), (5, 2)]
            .into_iter()
            .map(|(x, y)| Coords::new(x, y))
            .collect();
        let blinker = universe.set_cells(
            Id::new(0, Branch::MAX_LEVEL),
            Coords::new(0, 0),
            &mut coords,
        );

        let next = universe.evolve(blinker, 4);
        let next = universe.evolve(next, 4);
        let next = universe.evolve(next, 4);
        for &leaf in &universe.base {
            println!("{}\n", leaf);
        }
        dbg!(next);
    }
}
