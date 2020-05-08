// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    grid::Grid2,
    grid::Grid4,
    node::{Branch, Id, Leaf, Level, Node},
    Error, Result, Rule,
};
use either::Either;
use slotmap::DenseSlotMap;
use std::collections::HashMap;

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
    node_data: DenseSlotMap<Id, Data>,
}

impl Store {
    pub fn new(rule: Rule) -> Self {
        Self {
            rule,
            id_lookup: HashMap::new(),
            node_data: DenseSlotMap::with_key(),
        }
    }

    fn get_data(&self, id: Id) -> Result<&Data> {
        self.node_data.get(id).ok_or(Error::IdNotFound(id))
    }

    fn get_data_mut(&mut self, id: Id) -> Result<&mut Data> {
        self.node_data.get_mut(id).ok_or(Error::IdNotFound(id))
    }

    fn make_id(&mut self, node: Node) -> Id {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let data = Data {
                node,
                idle: None,
                jump: None,
            };
            let new_id = self.node_data.insert(data);
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

    fn idle(&mut self, input: Branch) -> Result<Id> {
        let input_id = self.make_id(Node::Branch(input));

        if let Some(saved_idle) = self.get_data(input_id)?.idle {
            return Ok(saved_idle);
        }

        let result_node = match self.classify_children(input.children)? {
            Either::Left(leaf_grid) => {
                Node::Leaf(Leaf::apply(leaf_grid, self.rule, Leaf::idle, Leaf::idle))
            }
            Either::Right(branch_grid) => {
                let grandchildren: Grid4<Id> = branch_grid.map(|branch| branch.children).flatten();
                Node::Branch(self.make_branch(grandchildren.center())?)
            }
        };

        let result_id = self.make_id(result_node);
        let mut saved_data = self.get_data_mut(input_id)?;
        saved_data.idle = Some(result_id);
        Ok(result_id)
    }

    fn jump(&mut self, input: Branch) -> Result<Id> {
        let input_id = self.make_id(Node::Branch(input));

        if let Some(saved_jump) = self.get_data(input_id)?.jump {
            return Ok(saved_jump);
        }

        let result_node = match self.classify_children(input.children)? {
            Either::Left(leaf_grid) => {
                Node::Leaf(Leaf::apply(leaf_grid, self.rule, Leaf::jump, Leaf::jump))
            }

            Either::Right(branch_grid) => {
                let grandchildren: Grid4<Id> = branch_grid.map(|branch| branch.children).flatten();
                let complete = grandchildren
                    .shrink(|ids| self.make_branch(ids).and_then(|b| self.jump(b)))?
                    .shrink(|ids| self.make_branch(ids).and_then(|b| self.jump(b)))?;
                Node::Branch(self.make_branch(complete)?)
            }
        };

        let result_id = self.make_id(result_node);
        let mut saved_data = self.get_data_mut(input_id)?;
        saved_data.jump = Some(result_id);
        Ok(result_id)
    }

    fn classify_children(&self, id_grid: Grid2<Id>) -> Result<Either<Grid2<Leaf>, Grid2<Branch>>> {
        id_grid
            .try_map(|id| self.get_data(id))?
            .map(|data| data.node)
            .classify()
    }

    pub fn evolve(&mut self, branch: Branch, steps: u64) -> Result<Id> {
        let rule = self.rule;

        let max_steps = branch.level.max_steps();
        assert!(steps <= max_steps, format!("step too large"));

        match self.classify_children(branch.children)? {
            Either::Left(leaf_grid) => {
                let (idle_fn, step_fn, jump_fn) = (Leaf::idle, Leaf::step, Leaf::jump);
                let new_leaf = match steps {
                    0 => Ok(Leaf::apply(leaf_grid, rule, idle_fn, idle_fn)),
                    1 => Ok(Leaf::apply(leaf_grid, rule, idle_fn, step_fn)),
                    2 => Ok(Leaf::apply(leaf_grid, rule, idle_fn, jump_fn)),
                    3 => Ok(Leaf::apply(leaf_grid, rule, step_fn, jump_fn)),
                    4 => Ok(Leaf::apply(leaf_grid, rule, jump_fn, jump_fn)),
                    _ => Err(Error::StepOverflow {
                        step: steps,
                        level: Level(3),
                    }),
                }?;
                Ok(self.make_id(Node::Leaf(new_leaf)))
            }

            Either::Right(branch_grid) => {
                let grandchildren = branch_grid.map(|branch| branch.children).flatten();

                let half_max = max_steps / 2;

                let complete = if steps < half_max {
                    grandchildren
                        .shrink(|ids| self.make_branch(ids).and_then(|b| self.idle(b)))?
                        .shrink(|ids| self.make_branch(ids).and_then(|b| self.evolve(b, steps)))?
                } else {
                    let step = steps - half_max;
                    grandchildren
                        .shrink(|ids| self.make_branch(ids).and_then(|b| self.evolve(b, step)))?
                        .shrink(|ids| self.make_branch(ids).and_then(|b| self.jump(b)))?
                };

                let new_branch = self.make_branch(complete)?;
                Ok(self.make_id(Node::Branch(new_branch)))
            }
        }
    }
}
