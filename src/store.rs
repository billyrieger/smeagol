// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    node::{Branch, Leaf, Level, Node},
    util::{Grid2, Grid4},
    Cell, Error, Result, Rule,
};

use std::{collections::HashMap, convert::TryFrom, fmt::Write};

use either::Either;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id {
    index: usize,
}

impl Id {
    fn new(index: usize) -> Self {
        Self { index }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, dx: i64, dy: i64) -> Position {
        Self::new(self.x + dx, self.y + dy)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Data {
    node: Node,
    idle: Option<Id>,
    jump: Option<Id>,
}

#[derive(Clone, Default)]
pub struct Store {
    rule: Rule,
    id_lookup: HashMap<Node, Id>,
    empties: Vec<Id>,
    node_data: Vec<Data>,
}

pub struct AliveCells<'a> {
    store: &'a Store,
    unexplored: Vec<(Id, Position)>,
    current: Vec<Position>,
    center: Position,
}

impl<'a> AliveCells<'a> {
    pub fn new(store: &'a Store, root: Id) -> Self {
        Self {
            store,
            unexplored: vec![(root, Position::new(0, 0))],
            current: vec![],
            center: Position::new(0, 0),
        }
    }
}

impl<'a> Iterator for AliveCells<'a> {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        self.current
            .pop()
            .or_else(|| {
                while self.current.is_empty() {
                    let (next_id, center) = self.unexplored.pop()?;
                    self.center = center;

                    let data = self.store.get_data(next_id).ok()?;
                    if data.node.population() == 0 {
                        continue;
                    }

                    match data.node {
                        Node::Leaf(leaf) => self.current.extend(leaf.alive_cells()),
                        Node::Branch(branch) => {
                            let [nw, ne, sw, se] = branch.children.0;
                            let offset = i64::try_from(data.node.level().side_len() / 4).unwrap();
                            let (dx, dy) = (offset, offset);
                            self.unexplored.push((se, center.offset(dx, dy)));
                            self.unexplored.push((sw, center.offset(-dx, dy)));
                            self.unexplored.push((ne, center.offset(dx, -dy)));
                            self.unexplored.push((nw, center.offset(-dx, -dy)));
                        },
                    }
                }
                self.current.pop()
            })
            .map(|pos| pos.offset(self.center.x, self.center.y))
    }
}

impl Store {
    pub fn new(rule: Rule) -> Self {
        Self {
            rule,
            id_lookup: HashMap::new(),
            node_data: vec![],
            empties: vec![],
        }
    }

    pub fn initialize(&mut self) -> Result<Id> {
        let empty = Node::Leaf(Leaf::dead());

        let mut current_id = self.get_id(empty);
        let mut current_level = empty.level();

        self.empties = vec![current_id; 4];

        while let Ok(next_level) = current_level.increment() {
            let next_branch = self.make_branch(Grid2([current_id; 4]))?;
            let next_id = self.get_id(Node::Branch(next_branch));

            current_id = next_id;
            current_level = next_level;

            self.empties.push(current_id);
            if self.empties.len() > 5 {
                break;
            }
        }

        Ok(current_id)
    }

    pub fn debug(&self, id: Id) -> Result<String> {
        let mut buffer = String::new();
        let level = self.get_data(id)?.node.level();
        let side_len = i64::try_from(level.side_len()).unwrap();
        let index_range = (-side_len / 2)..(side_len / 2);
        for y in index_range.clone() {
            for x in index_range.clone() {
                write!(
                    buffer,
                    "{}",
                    match self.get_cell(id, x, y)? {
                        Cell::Dead => '.',
                        Cell::Alive => '#',
                    }
                )?;
            }
            writeln!(buffer)?;
        }
        Ok(buffer)
    }

    pub fn get_cell(&self, id: Id, x: i64, y: i64) -> Result<Cell> {
        let data = self.get_data(id)?;

        let level = data.node.level();
        let offset = i64::try_from(level.side_len() / 4).unwrap();
        let (dx, dy) = (offset, offset);

        match data.node {
            Node::Leaf(leaf) => Ok(leaf.get_cell(x, y)),
            Node::Branch(branch) => {
                let [northwest, northeast, southwest, southeast] = branch.children.0;
                match (x < 0, y < 0) {
                    (true, true) => self.get_cell(northwest, x + dx, y + dy),
                    (false, true) => self.get_cell(northeast, x - dx, y + dy),
                    (true, false) => self.get_cell(southwest, x + dx, y - dy),
                    (false, false) => self.get_cell(southeast, x - dx, y - dy),
                }
            }
        }
    }

    pub fn alive_cells(&self, id: Id) -> AliveCells<'_> {
        AliveCells::new(self, id)
    }

    pub fn set_cell(&mut self, id: Id, x: i64, y: i64, cell: Cell) -> Result<Id> {
        let data = self.get_data(id)?;

        let level = data.node.level();
        let offset = i64::try_from(level.side_len() / 4).unwrap();
        let (dx, dy) = (offset, offset);

        match data.node {
            Node::Leaf(leaf) => Ok(self.get_id(Node::Leaf(leaf.set_cell(x, y, cell)))),
            Node::Branch(branch) => {
                let offset = i64::try_from(level.side_len() / 4).unwrap();
                let [mut nw, mut ne, mut sw, mut se] = branch.children.0;
                match (x < 0, y < 0) {
                    (true, true) => {
                        nw = self.set_cell(nw, x + dx, y + dy, cell)?;
                    }
                    (false, true) => {
                        ne = self.set_cell(ne, x - dx, y + dy, cell)?;
                    }
                    (true, false) => {
                        sw = self.set_cell(sw, x + dx, y - dy, cell)?;
                    }
                    (false, false) => {
                        se = self.set_cell(se, x - dx, y - dy, cell)?;
                    }
                };
                let new_branch = self.make_branch(Grid2([nw, ne, sw, se]))?;
                Ok(self.get_id(Node::Branch(new_branch)))
            }
        }
    }

    pub fn set_cells<I>(&mut self, id: Id, coords: I, cell: Cell) -> Result<Id>
    where
        I: IntoIterator<Item = Position>,
    {
        // Itertools::collect_vec
        let mut coords: Vec<_> = coords.into_iter().collect();
        self.set_helper(id, &mut coords, cell)
    }

    fn set_helper(&mut self, id: Id, coords: &mut [Position], cell: Cell) -> Result<Id> {
        let data = self.get_data(id)?;
        match data.node {
            Node::Leaf(mut leaf) => {
                for point in coords {
                    leaf = leaf.set_cell(point.x, point.y, cell);
                }
                Ok(self.get_id(Node::Leaf(leaf)))
            }
            Node::Branch(branch) => {
                let offset = i64::try_from(data.node.level().side_len() / 4).unwrap();
                // a note in itertools::partition
                // elements that satisfy the predicate are placed before the elements that don't
                let split_index = itertools::partition(coords.iter_mut(), |p| p.x >= 0);
                let (east_coords, west_coords) = coords.split_at_mut(split_index);

                let split_index = itertools::partition(east_coords.iter_mut(), |p| p.y >= 0);
                let (se_coords, ne_coords) = east_coords.split_at_mut(split_index);

                let split_index = itertools::partition(west_coords.iter_mut(), |p| p.y >= 0);
                let (sw_coords, nw_coords) = west_coords.split_at_mut(split_index);

                for p in nw_coords.iter_mut() {
                    p.x += offset;
                    p.y += offset;
                }

                for p in ne_coords.iter_mut() {
                    p.x -= offset;
                    p.y += offset;
                }

                for p in sw_coords.iter_mut() {
                    p.x += offset;
                    p.y -= offset;
                }

                for p in se_coords.iter_mut() {
                    p.x -= offset;
                    p.y -= offset;
                }

                let [nw_id, ne_id, sw_id, se_id] = branch.children.0;

                let new_nw_id = self.set_helper(nw_id, nw_coords, cell)?;
                let new_ne_id = self.set_helper(ne_id, ne_coords, cell)?;
                let new_sw_id = self.set_helper(sw_id, sw_coords, cell)?;
                let new_se_id = self.set_helper(se_id, se_coords, cell)?;

                let branch =
                    self.make_branch(Grid2([new_nw_id, new_ne_id, new_sw_id, new_se_id]))?;
                Ok(self.get_id(Node::Branch(branch)))
            }
        }
    }

    fn get_data(&self, id: Id) -> Result<&Data> {
        self.node_data.get(id.index()).ok_or(Error::IdNotFound(id))
    }

    fn get_data_mut(&mut self, id: Id) -> Result<&mut Data> {
        self.node_data
            .get_mut(id.index())
            .ok_or(Error::IdNotFound(id))
    }

    fn get_id(&mut self, node: Node) -> Id {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let data = Data {
                node,
                idle: None,
                jump: None,
            };
            let new_id = Id::new(self.node_data.len());
            self.node_data.push(data);
            self.id_lookup.insert(node, new_id);
            new_id
        })
    }

    fn make_branch(&mut self, children: Grid2<Id>) -> Result<Branch> {
        let data = children.try_map(|id| self.get_data(id))?;
        let level = data.0[0].node.level().increment()?;
        let population = data.0.iter().map(|data| data.node.population()).sum();
        Ok(Branch {
            children,
            level,
            population,
        })
    }

    fn empty(&self, level: Level) -> Id {
        self.empties[level.0 as usize]
    }

    fn step(&mut self, id: Id, step: u64) -> Result<Id> {
        match self.get_data(id)?.node {
            Node::Leaf(_) => todo!(),
            Node::Branch(branch) => self.evolve(branch.children, step),
        }
    }

    fn evolve(&mut self, grid: Grid2<Id>, steps: u64) -> Result<Id> {
        let rule = self.rule;

        let branch = self.make_branch(grid)?;
        let branch_id = self.get_id(Node::Branch(branch));
        let branch_data = self.get_data(branch_id).unwrap();

        if branch.population == 0 {
            let empty_id = self.empty(Level(branch.level.0 - 1));
            self.get_data_mut(branch_id)?.idle = Some(empty_id);
            return Ok(empty_id);
        }

        let max_steps = branch.level.max_steps();
        let half_max = max_steps / 2;
        assert!(steps <= max_steps, format!("step too large"));

        if steps == 0 {
            if let Some(idle) = branch_data.idle {
                return Ok(idle);
            }
        }

        if steps == max_steps {
            if let Some(jump) = branch_data.jump {
                return Ok(jump);
            }
        }

        let children: Either<Grid2<Leaf>, Grid2<Branch>> = branch
            .children
            .try_map(|id| self.get_data(id))?
            .map(|data| data.node)
            .classify()?;

        let result: Node = match children {
            Either::Left(leaf_grid) => Node::Leaf(Leaf::evolve_leaves(leaf_grid, steps, rule)),

            Either::Right(branch_grid) => {
                let grandchildren: Grid4<Id> = branch_grid.map(|branch| branch.children).flatten();

                let (first_step, second_step) = if steps < half_max {
                    (0, steps)
                } else {
                    (steps - half_max, half_max)
                };

                let new_children: Grid2<Id> = grandchildren
                    .shrink(|ids| self.evolve(ids, first_step))?
                    .shrink(|ids| self.evolve(ids, second_step))?;

                Node::Branch(self.make_branch(new_children)?)
            }
        };

        let id = self.get_id(result);

        if steps == 0 {
            self.get_data_mut(branch_id)?.idle = Some(id);
        } else if steps == max_steps {
            self.get_data_mut(branch_id)?.jump = Some(id);
        }

        Ok(self.get_id(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store() {
        let mut store = Store::new(Rule::new(&[3], &[2, 3]));

        let root = store.initialize().unwrap();

        // let coords = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]

        let coords = vec![(-1, -2), (0, -1), (-2, 0), (-1, 0), (0, 0)]
            .into_iter()
            .map(|(x, y)| Position { x, y });
        let root = store.set_cells(root, coords, Cell::Alive).unwrap();

        let alive: Vec<Position> = store.alive_cells(root).collect();
        dbg!(alive);

        let four = store.step(root, 4).unwrap();

        let alive: Vec<Position> = store.alive_cells(four).collect();
        dbg!(alive);

        let eight = store.step(root, 8).unwrap();

        let alive: Vec<Position> = store.alive_cells(eight).collect();
        dbg!(alive);
    }
}
