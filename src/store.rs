// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{life::Rule, util::Grid2};

use arrayvec::ArrayVec;
use generational_arena::{Arena, Index};

const MAX_SIDE_LEN: u64 = 1 << 63;
const LEAF_SIDE_LEN: u64 = 1 << 3;

pub trait NodeVisitor<T = ()> {
    fn visit_leaf(&mut self, leaf: Leaf) -> Option<T>;
    fn visit_branch(&mut self, branch: Branch) -> Option<T>;
}

pub struct AliveCells<'s, R: 's> {
    store: &'s Store<R>,
    root: Id,
    center: Position,
    coords: Vec<Position>,
}

impl<'s, R: 's> NodeVisitor for AliveCells<'s, R> {
    fn visit_leaf(&mut self, leaf: Leaf) -> Option<()> {
        None
    }

    fn visit_branch(&mut self, branch: Branch) -> Option<()> {
        None
    }
}

pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub const ORIGIN: Self = Self::new(0, 0);

    /// Creates a new `Position` from the given `x` and `y` coordinates.
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub const fn offset(&self, dx: i64, dy: i64) -> Position {
        Self::new(self.x + dx, self.y + dy)
    }

    pub const fn relative_to(&self, other: Position) -> Position {
        self.offset(-other.x, -other.y)
    }

    pub fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Dead,
    Alive,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Id {
    index: Index,
}

impl Id {
    fn new(index: Index) -> Self {
        Self { index }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: u64,
}

impl Leaf {
    pub const MIN: i64 = -4;
    pub const MAX: i64 = 3;
    pub const SIDE_LEN: i64 = 8;

    pub const DEAD: Self = Self::new(0);
    pub const ALIVE: Self = Self::new(!0);

    pub const fn new(alive: u64) -> Self {
        Self { alive }
    }

    pub fn alive_cells(&self) -> ArrayVec<[Position; 64]> {
        let mut result = ArrayVec::new();

        if self.alive == 0 {
            return result;
        }

        let mut bits: u64 = self.alive;
        let mut reverse_index: usize = 0;

        while bits > 0 {
            let n_zeros = bits.leading_zeros() as usize;

            bits <<= n_zeros;
            reverse_index += n_zeros;

            result.push(self.idx_to_pos(63 - reverse_index));

            reverse_index += 1;
            bits <<= 1;
        }

        result
    }

    fn check_bounds(&self, pos: Position) -> bool {
        let x_ok = Self::MIN <= pos.x && pos.x <= Self::MAX;
        let y_ok = Self::MIN <= pos.y && pos.y <= Self::MAX;
        x_ok && y_ok
    }

    fn pos_to_idx(&self, pos: Position) -> usize {
        (Self::SIDE_LEN * (Self::MAX - pos.y) + (Self::MAX - pos.x)) as usize
    }

    fn idx_to_pos(&self, index: usize) -> Position {
        let index = index as i64;
        let y = Self::MAX - index / Self::SIDE_LEN;
        let x = Self::MAX - index % Self::SIDE_LEN;
        Position::new(x, y)
    }

    pub fn population(&self) -> u128 {
        u128::from(self.alive.count_ones())
    }

    pub fn get_cell(&self, pos: Position) -> Option<Cell> {
        if self.check_bounds(pos) {
            None
        } else {
            let index = self.pos_to_idx(pos);
            if self.alive & (1 << index) == 0 {
                Some(Cell::Dead)
            } else {
                Some(Cell::Alive)
            }
        }
    }

    fn set_cells(&self, coords: &mut [Position], cell: Cell) -> Option<Self> {
        // coords
        //     .iter()
        //     .flat_map(|&pos| self.get_cell(pos))
        //     .fold(*self, |leaf: Leaf, index: usize| match cell {
        //         Cell::Dead => Leaf::new(leaf.alive & !(1 << index)),
        //         Cell::Alive => Leaf::new(leaf.alive | (1 << index)),
        //     });
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Branch {
    children: Grid2<Id>,
    side_len: u64,
    population: u128,
}

#[derive(Clone, Copy, Debug)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn population(&self) -> u128 {
        match self {
            Node::Leaf(leaf) => leaf.population(),
            Node::Branch(branch) => branch.population,
        }
    }
    pub fn kind(&self) -> NodeKind {
        match self {
            Node::Leaf(_) => NodeKind::Leaf,
            Node::Branch(_) => NodeKind::Branch,
        }
    }
}

pub enum NodeKind {
    Leaf,
    Branch,
}

pub struct Store<R> {
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
        let empty_leaf = Leaf::new(0);
        let empty_leaf_id = Id::new(self.nodes.insert(Node::Leaf(empty_leaf)));

        let mut empty_branch = Branch {
            children: Grid2::repeat(empty_leaf_id),
            side_len: LEAF_SIDE_LEN << 1,
            population: 0,
        };

        let mut id = empty_leaf_id;
        let mut side_len = LEAF_SIDE_LEN;
        loop {
            empty_branch = Branch {
                children: Grid2::repeat(id),
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

    pub fn get_node(&self, id: Id) -> Option<Node> {
        self.nodes.get(id.index).copied()
    }

    pub fn visit<B, F, G>(
        &self,
        init: B,
        root_id: Id,
        visit_leaf: &mut F,
        visit_branch: &mut G,
    ) -> Option<B>
    where
        F: Fn(B, Leaf) -> Option<B>,
        G: Fn(B, Branch) -> Option<B>,
    {
        let root: Node = self.get_node(root_id)?;
        match root {
            Node::Leaf(leaf) => visit_leaf(init, leaf),
            Node::Branch(branch) => visit_branch(init, branch),
        }
    }
}

fn split<F, T>(list: &mut [T], pred: F) -> (&mut [T], &mut [T])
where
    F: Fn(&T) -> bool,
{
    // a note on itertools::partition
    // elements that satisfy the predicate are placed before the elements that don't
    let index: usize = itertools::partition(list.iter_mut(), |t| pred(t));
    list.split_at_mut(index)
}

fn partition(list: &mut [Position]) -> Grid2<&mut [Position]> {
    let (north, south) = split(list, |p| p.y < 0);

    let (nw, ne) = split(north, |p| p.x < 0);
    let (sw, se) = split(south, |p| p.x < 0);

    Grid2::pack([nw, ne, sw, se])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alive_cells() {
        let empty = Leaf::DEAD;
        let nw_leaf = Leaf::new(0x_80_00_00_00_00_00_00_00);
        let ne_leaf = Leaf::new(0x_01_00_00_00_00_00_00_00);
        let sw_leaf = Leaf::new(0x_00_00_00_00_00_00_00_80);
        let se_leaf = Leaf::new(0x_00_00_00_00_00_00_00_01);
        let four_corners = Leaf::new(nw_leaf.alive | ne_leaf.alive | sw_leaf.alive | se_leaf.alive);

        assert_eq!(&*empty.alive_cells(), &[]);
        assert_eq!(&*nw_leaf.alive_cells(), &[Position::new(-4, -4)]);
        assert_eq!(&*ne_leaf.alive_cells(), &[Position::new(3, -4)]);
        assert_eq!(&*sw_leaf.alive_cells(), &[Position::new(-4, 3)]);
        assert_eq!(&*se_leaf.alive_cells(), &[Position::new(3, 3)]);
        assert_eq!(
            &*four_corners.alive_cells(),
            &[
                Position::new(-4, -4),
                Position::new(3, -4),
                Position::new(-4, 3),
                Position::new(3, 3)
            ]
        );
    }
}

//     fn _get_cells_helper(
//         &self,
//         id: Id,
//         coords: &mut [Position],
//         center: Position,
//         buf: &mut Vec<Cell>,
//     ) -> Option<()> {
//         if coords.is_empty() {
//             return Some(());
//         }

//         let node = self.get_node(id)?;
//         match node {
//             Node::Leaf(leaf) => {
//                 for &mut pos in coords {
//                     buf.push(leaf.get_cell(pos.relative_to(center))?);
//                 }
//                 Some(())
//             }

//             Node::Branch(branch) => {
//                 let delta = (branch.side_len / 4) as i64;
//                 let (dx, dy) = (delta, delta);

//                 let is_western = |p: &Position| p.x >= 0;
//                 let is_northern = |p: &Position| p.y >= 0;

//                 let ids: [Id; 4] = branch.children.unpack();

//                 let centers: [Position; 4] = [
//                     center.offset(-dx, -dy),
//                     center.offset(dx, -dy),
//                     center.offset(-dx, dy),
//                     center.offset(dx, dy),
//                 ];

//                 let mut parts: [&mut [Position]; 4] =
//                     self.partition(coords, is_western, is_northern).unpack();

//                 for ((&id, &center), part) in ids.iter().zip(centers.iter()).zip(parts.iter_mut()) {
//                     self._get_cells_helper(id, part, center, buf)?;
//                 }

//                 Some(())
//             }
//         }
