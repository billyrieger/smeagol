// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(iter_partition_in_place)]

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
    const fn new(index: usize, level: usize) -> Self {
        let _ = level - Leaf::level();
        let _ = Branch::MAX_LEVEL - level;
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
            Node::Leaf(_) => Leaf::level(),
            Node::Branch(branch) => branch.level(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Universe<R = B3S23> {
    leaves: IndexSet<Leaf>,
    branches: Vec<IndexMap<Grid2<Id>, BranchData>>,
    rule: R,
}

impl Universe {
    fn new() -> Self {
        let leaves = indexset! { Leaf::empty() };
        let mut branches: Vec<IndexMap<Grid2<Id>, BranchData>> = vec![];

        for level in Branch::MIN_LEVEL..=Branch::MAX_LEVEL {
            let empty = Grid2::repeat(Id::new(0, level - 1));
            branches.push(indexmap! { empty => BranchData::new(), });
        }

        Self {
            leaves,
            branches,
            rule: B3S23,
        }
    }

    fn create_leaf(&mut self, leaf: Leaf) -> Id {
        let index = self.leaves.insert_full(leaf).0;
        Id::new(index, Leaf::level())
    }

    fn create_branch(&mut self, ids: [Id; 4]) -> Id {
        let ids: Grid2<Id> = ids.into();
        let children: Grid2<Node> = ids.map(|id| self.get(id));
        let is_empty = children.nw.is_empty()
            && children.ne.is_empty()
            && children.sw.is_empty()
            && children.se.is_empty();

        let mut data = BranchData::new();
        data.is_empty = is_empty;

        let level = children.nw.level() + 1;
        let index = self.branches[level - Leaf::level() - 1]
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
        *self.leaves.get_index(id.index()).expect("invalid index")
    }

    fn get_branch(&self, id: Id) -> Branch {
        let (children, data) = self.branches[id.level() - Branch::MIN_LEVEL]
            .get_index(id.index())
            .expect("invalid index");
        Branch {
            children: *children,
            is_empty: data.is_empty,
            jump: data.jump,
        }
    }

    fn set_cells_relative(&mut self, id: Id, center: Coords, coords: &mut [Coords]) -> Id {
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

            let nw = self.set_cells_relative(branch.children.nw, center_nw, coords_nw);
            let ne = self.set_cells_relative(branch.children.ne, center_ne, coords_ne);
            let sw = self.set_cells_relative(branch.children.sw, center_sw, coords_sw);
            let se = self.set_cells_relative(branch.children.se, center_se, coords_se);
            self.create_branch([nw, ne, sw, se])
        }
    }

    fn surround(&mut self, id: Id) -> Id {
        assert!(id.level() > Leaf::level());

        let parent = self.get_branch(id);

        let empty = Id::new(0, parent.level() - 1);
        let grid: Grid2<Id> = parent.children;

        let nw = self.create_branch([empty, empty, empty, grid.nw]);
        let ne = self.create_branch([empty, empty, grid.ne, empty]);
        let sw = self.create_branch([empty, grid.sw, empty, empty]);
        let se = self.create_branch([grid.se, empty, empty, empty]);

        self.create_branch([nw, ne, sw, se])
    }

    fn idle(&mut self, id: Id) -> Id {
        let parent = self.get_branch(id);

        if parent.level() == Leaf::level() + 1 {
            let leaves: Grid2<Leaf> = parent.children.map(|id| self.get_leaf(id));
            self.create_leaf(Clover::new(leaves).center())
        } else {
            let grid: Grid2<Grid2<Id>> = parent.children.map(|id| self.get_branch(id).children);
            self.create_branch([grid.nw.se, grid.ne.sw, grid.sw.ne, grid.se.nw])
        }
    }

    fn population(&mut self, id: Id) -> u128 {
        match self.get(id) {
            Node::Leaf(leaf) => leaf.population() as u128,
            Node::Branch(branch) => {
                if branch.is_empty {
                    0
                } else {
                    self.population(branch.children.nw)
                        + self.population(branch.children.ne)
                        + self.population(branch.children.sw)
                        + self.population(branch.children.se)
                }
            }
        }
    }

    fn evolve(&mut self, parent_id: Id, generations: u64) -> Id {
        assert!(
            parent_id.level() > Leaf::level(),
            "cannot evolve id {:?}",
            parent_id
        );
        let parent = self.get_branch(parent_id);

        let jump_size: u64 = 1 << (parent.level() - 2);

        if generations == 0 || parent.is_empty {
            return self.idle(parent_id);
        }

        if generations == 1 << (parent.level() - 2) {
            if let Some(jump) = parent.jump {
                return jump;
            }
        }

        if parent.level() == Branch::MIN_LEVEL {
            let leaves: Grid2<Leaf> = parent.children.map(|id| self.get_leaf(id));
            let mut clover = Clover::new(leaves);
            for _ in 0..generations {
                clover = self.rule.step(clover);
            }
            self.create_leaf(clover.center())
        } else {
            let grid: Grid2<Grid2<Id>> = parent.children.map(|id| self.get_branch(id).children);

            // a b | c d
            // e f | g h
            // ----+----
            // i j | k l
            // m n | o p
            let [a, b, e, f]: [Id; 4] = grid.nw.into();
            let [c, d, g, h]: [Id; 4] = grid.ne.into();
            let [i, j, m, n]: [Id; 4] = grid.sw.into();
            let [k, l, o, p]: [Id; 4] = grid.se.into();

            let west = self.create_branch([e, f, i, j]);
            let east = self.create_branch([g, h, k, l]);
            let north = self.create_branch([b, c, f, g]);
            let south = self.create_branch([j, k, n, o]);

            let center = self.create_branch([f, g, j, k]);

            let (step0, step1) = if generations < jump_size / 2 {
                (0, generations)
            } else {
                (generations - jump_size / 2, jump_size / 2)
            };

            // q r s
            // t u v
            // w x y
            let q = self.evolve(parent.children.nw, step0);
            let r = self.evolve(north, step0);
            let s = self.evolve(parent.children.ne, step0);
            let t = self.evolve(west, step0);
            let u = self.evolve(center, step0);
            let v = self.evolve(east, step0);
            let w = self.evolve(parent.children.sw, step0);
            let x = self.evolve(south, step0);
            let y = self.evolve(parent.children.se, step0);

            let nw_partial = self.create_branch([q, r, t, u]);
            let ne_partial = self.create_branch([r, s, u, v]);
            let sw_partial = self.create_branch([t, u, w, x]);
            let se_partial = self.create_branch([u, v, x, y]);

            let nw = self.evolve(nw_partial, step1);
            let ne = self.evolve(ne_partial, step1);
            let sw = self.evolve(sw_partial, step1);
            let se = self.evolve(se_partial, step1);

            let result = self.create_branch([nw, ne, sw, se]);

            if generations == 1 << (parent.level() - 2) {
                let (_, data) = self.branches[parent_id.level() - Branch::MIN_LEVEL]
                    .get_index_mut(parent_id.index())
                    .expect("bad id");
                data.jump = Some(result);
            }

            result
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

pub fn run() {
    let mut universe = Universe::new();
    let mut coords: Vec<Coords> = vec![(0, 0), (1, 0), (0, 1), (0, 2), (-1, 1)]
        // let mut coords: Vec<Coords> = vec![(0, 0), (1, 0), (2, 0), (2, 1), (1, 2)]
        .into_iter()
        .map(|(x, y)| Coords::new(x, y))
        .collect();
    let mut pattern = universe.set_cells_relative(
        Id::new(0, Branch::MAX_LEVEL),
        Coords::new(0, 0),
        &mut coords,
    );
    let mut gen: u64 = 0;

    println!(
        "generation {}, population {}",
        gen,
        universe.population(pattern),
    );

    let mut delta = 1;
    loop {
        pattern = universe.evolve(pattern, delta);
        pattern = universe.surround(pattern);
        gen += delta;
        assert!(pattern.level() == Branch::MAX_LEVEL);
        println!(
            "generation {}, population {}",
            gen,
            universe.population(pattern)
        );
        delta += 1;

        // let n_leaves = universe.leaves.len();
        // let branches: Vec<usize> = universe.branches.iter().map(|map| map.len()).collect();
        // let n_branches: usize = branches.iter().sum();
        // println!(
        //     "leaves: {:?}, branches: {:?} = {:?}",
        //     n_leaves, n_branches, branches
        // );
        // println!("leaves: {}", universe.leaves.len());
        // println!("lvl4  : {}", universe.branches[0].len());
        // println!("lvl5  : {}", universe.branches[1].len());
        // println!("last  : {}", universe.branches.last().unwrap().len());
    }

    // let next = universe.evolve(pattern, 4);
    // let next = universe.evolve(next, 4);
    // let next = universe.evolve(next, 4);
    // for &leaf in &universe.leaves {
    //     println!("{}\n", leaf);
    // }
    // println!("next: {:?}", next);
    // for level in Branch::MIN_LEVEL..=Branch::MAX_LEVEL {
    //     println!(
    //         "level {:?} has {:?} entries",
    //         level,
    //         universe.branches[level - 4].len()
    //     );
    // }
}
