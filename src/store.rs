// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{life::Rule, Grid2};

use generational_arena::{Arena, Index};
use itertools::izip;

const MAX_SIDE_LEN: u64 = 1 << 63;
const LEAF_SIDE_LEN: u64 = 1 << 3;

// const MIN_COORD: i64 = -((MAX_SIDE_LEN / 2) as i64);
// const MAX_COORD: i64 = ((MAX_SIDE_LEN / 1) - 1) as i64;
const MIN_LEAF_COORD: i64 = -((LEAF_SIDE_LEN >> 1) as i64);
const MAX_LEAF_COORD: i64 = ((LEAF_SIDE_LEN / 1) - 1) as i64;

pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

#[derive(Clone, Copy, Debug)]
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
    pub const MIN_COORD: i64 = -4;
    pub const MAX_COORD: i64 = 3;

    pub const DEAD: Self = Self::new(0);
    pub const ALIVE: Self = Self::new(!0);

    pub const fn new(alive: u64) -> Self {
        Self { alive }
    }

    fn localize(&self, pos: Position) -> Option<usize> {
        let (min, max): (i64, i64) = (Self::MIN_COORD, Self::MAX_COORD);

        let x_is_oob: bool = !(min <= pos.x && pos.x <= max);
        let y_is_oob: bool = !(min <= pos.y && pos.y <= max);

        if x_is_oob || y_is_oob {
            None
        } else {
            Some((8 * (max - pos.y) + (max - pos.x)) as usize)
        }
    }

    pub fn population(&self) -> u128 {
        u128::from(self.alive.count_ones())
    }

    pub fn get_cell(&self, pos: Position) -> Option<Cell> {
        self.localize(pos).map(|index| {
            if self.alive & (1 << index) == 0 {
                Cell::Dead
            } else {
                Cell::Alive
            }
        })
    }

    fn set_cells(self, coords: &mut [Position], cell: Cell) -> Option<Self> {
        coords.iter().flat_map(|&pos| self.localize(pos)).fold(
            self,
            |mut leaf: Leaf, index: usize| {
                match cell {
                    Cell::Dead => todo!(),
                    Cell::Alive => {
                        // leaf.alive[row_idx] = leaf.alive[row_idx] | (1 << col_idx);
                        todo!()
                    }
                }
            },
        );
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

    pub fn get_cells(&self, id: Id, coords: &mut [Position]) -> Option<Vec<Cell>> {
        let mut accumulator: Vec<Cell> = Vec::with_capacity(coords.len());
        self.get_cells_helper(id, coords, Position::ORIGIN, &mut accumulator)?;
        Some(accumulator)
    }

    fn get_cells_helper(
        &self,
        id: Id,
        coords: &mut [Position],
        center: Position,
        accumulator: &mut Vec<Cell>,
    ) -> Option<()> {
        if coords.is_empty() {
            return Some(());
        }

        let node = self.get_node(id)?;
        match node {
            Node::Leaf(leaf) => {
                for &mut pos in coords {
                    accumulator.push(leaf.get_cell(pos.relative_to(center))?);
                }
                Some(())
            }

            Node::Branch(branch) => {
                let delta = (branch.side_len / 4) as i64;
                let (dx, dy) = (delta, delta);

                let is_western = |p: &Position| p.x >= 0;
                let is_northern = |p: &Position| p.y >= 0;

                let ids: [Id; 4] = branch.children.unpack();

                let centers: [Position; 4] = [
                    center.offset(-dx, -dy),
                    center.offset(dx, -dy),
                    center.offset(-dx, dy),
                    center.offset(dx, dy),
                ];

                let mut parts: [&mut [Position]; 4] =
                    self.partition(coords, is_western, is_northern).unpack();

                for (&id, &center, part) in izip!(ids.iter(), centers.iter(), parts.iter_mut()) {
                    self.get_cells_helper(id, part, center, accumulator)?;
                }

                Some(())
            }
        }
    }

    fn set_cells_helper(
        &mut self,
        id: Id,
        coords: &mut [Position],
        center: Position,
        cell: Cell,
    ) -> Option<(Node, Id)> {
        let node = self.get_node(id)?;

        if coords.is_empty() {
            return Some((node, id));
        }

        match node {
            Node::Leaf(leaf) => {
                // for &mut pos in coords {
                //     accumulator.push(leaf.get_cell(pos.relative_to(center))?);
                // }
                None
            }

            Node::Branch(branch) => {
                let delta = (branch.side_len / 4) as i64;
                let (dx, dy) = (delta, delta);

                let is_western = |p: &Position| p.x >= 0;
                let is_northern = |p: &Position| p.y >= 0;

                let ids: [Id; 4] = branch.children.unpack();

                let centers: [Position; 4] = [
                    center.offset(-dx, -dy),
                    center.offset(dx, -dy),
                    center.offset(-dx, dy),
                    center.offset(dx, dy),
                ];

                let parts: [&mut [Position]; 4] =
                    self.partition(coords, is_western, is_northern).unpack();

                let (nw_node, nw_id) = self.set_cells_helper(ids[0], parts[0], centers[0], cell)?;
                let (ne_node, ne_id) = self.set_cells_helper(ids[1], parts[1], centers[1], cell)?;
                let (sw_node, sw_id) = self.set_cells_helper(ids[2], parts[2], centers[2], cell)?;
                let (se_node, se_id) = self.set_cells_helper(ids[3], parts[3], centers[3], cell)?;

                let new_population = 0
                    + nw_node.population()
                    + ne_node.population()
                    + sw_node.population()
                    + se_node.population();

                let new_branch = Branch {
                    children: Grid2::pack([nw_id, ne_id, sw_id, se_id]),
                    side_len: branch.side_len,
                    population: new_population,
                };
                None
            }
        }
    }

    fn partition<'list, F, G, T>(
        &self,
        list: &'list mut [T],
        horiz_pred: F,
        vert_pred: G,
    ) -> Grid2<&'list mut [T]>
    where
        F: Fn(&T) -> bool,
        G: Fn(&T) -> bool,
    {
        // a note on itertools::partition
        // elements that satisfy the predicate are placed before the elements that don't
        let split_index = itertools::partition(list.iter_mut(), |t| horiz_pred(&t));
        let (west_coords, east_coords) = list.split_at_mut(split_index);

        let split_index = itertools::partition(east_coords.iter_mut(), |t| vert_pred(&t));
        let (se_coords, ne_coords) = east_coords.split_at_mut(split_index);

        let split_index = itertools::partition(west_coords.iter_mut(), |t| vert_pred(&t));
        let (sw_coords, nw_coords) = west_coords.split_at_mut(split_index);

        Grid2::pack([nw_coords, ne_coords, sw_coords, se_coords])
    }
}
