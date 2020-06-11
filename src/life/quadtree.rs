// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::prelude::*;

use crate::{
    life::{Cell, Rule},
    util::{Bit8x8, Grid2},
};

use std::{convert::TryFrom, option::NoneError};

use generational_arena::{Arena, Index};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Length(u64);

impl Length {
    pub const MAX: Length = Self(1u64 << 63);
    pub const HALF_MAX: Length = Self(Self::MAX.0 / 2);
}

impl Into<u64> for Length {
    fn into(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for Length {
    type Error = NoneError;

    fn try_from(val: u64) -> Result<Length> {
        if val <= Length::MAX.0 {
            Ok(Length(val))
        } else {
            None?
        }
    }
}

const LEAF_SIDE_LEN: u64 = 1 << 3;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coord(i64);

impl Coord {
    pub const MIN: Coord = Self(-(Length::HALF_MAX.0 as i64));
    pub const MAX: Coord = Self((1i64 << 62) - 1);
}

impl Into<i64> for Coord {
    fn into(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for Coord {
    type Error = NoneError;

    fn try_from(val: i64) -> Result<Coord> {
        if Coord::MIN.0 <= val && val <= Coord::MAX.0 {
            Ok(Coord(val))
        } else {
            None?
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub const ORIGIN: Self = Self::new(0, 0);

    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub const fn offset(&self, dx: i64, dy: i64) -> Position {
        Self::new(self.x + dx, self.y + dy)
    }

    pub const fn relative_to(&self, other: Position) -> Position {
        self.offset(-other.x, -other.y)
    }
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
    pub alive: Bit8x8,
}

impl Leaf {
    const MIN: i64 = -4;
    const MAX: i64 = 3;
    const SIDE_LEN: i64 = 8;

    pub const fn new(alive: Bit8x8) -> Self {
        Self { alive }
    }

    pub fn alive_cells(&self) -> Vec<Position> {
        let mut result = Vec::new();

        if self.alive == Bit8x8::default() {
            return result;
        }

        let mut bits: u64 = self.alive.into();
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
        u128::from(self.alive.0.count_ones())
    }

    pub fn get_cell(&self, pos: Position) -> Option<Cell> {
        if self.check_bounds(pos) {
            None
        } else {
            let index = self.pos_to_idx(pos);
            if self.alive.0 & (1 << index) == 0 {
                Some(Cell::Dead)
            } else {
                Some(Cell::Alive)
            }
        }
    }

    fn _set_cells(&self, _coords: &mut [Position], _cell: Cell) -> Option<Self> {
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
    pub children: Grid2<Id>,
    pub side_len: u64,
    pub population: u128,
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
}

pub struct Trunk<R> {
    rule: R,
    nodes: Arena<Node>,
}

impl<R> Trunk<R>
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
        let empty_leaf = Leaf::new(Bit8x8(0));
        let empty_leaf_id = Id::new(self.nodes.insert(Node::Leaf(empty_leaf)));

        let mut empty_branch = Branch {
            children: Grid2::pack([empty_leaf_id; 4]),
            side_len: LEAF_SIDE_LEN << 1,
            population: 0,
        };

        let mut id = empty_leaf_id;
        let mut side_len = LEAF_SIDE_LEN;
        loop {
            empty_branch = Branch {
                children: Grid2::pack([id; 4]),
                side_len: empty_branch.side_len,
                population: 0,
            };
            id = Id::new(self.nodes.insert(Node::Branch(empty_branch)));
            if side_len == Length::MAX.0 {
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
        let empty = Leaf::new(Bit8x8(0));
        let nw_leaf = Leaf::new(Bit8x8(0x_80_00_00_00_00_00_00_00));
        let ne_leaf = Leaf::new(Bit8x8(0x_01_00_00_00_00_00_00_00));
        let sw_leaf = Leaf::new(Bit8x8(0x_00_00_00_00_00_00_00_80));
        let se_leaf = Leaf::new(Bit8x8(0x_00_00_00_00_00_00_00_01));
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
