// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    life::quadtree::{Branch, Leaf},
    util::{BitSquare, Grid2},
};

#[derive(Clone, Copy, Debug)]
pub enum Slot<T> {
    Occupied(T),
    Vacant,
}

impl<T> Slot<T>
where
    T: Copy,
{
    pub fn occupy(&mut self, value: T) {
        *self = Slot::Occupied(value);
    }

    pub fn vacate(&mut self) {
        *self = Slot::Vacant;
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Slot::Vacant
    }
}

#[derive(Clone, Debug, Default)]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    len: usize,
    next_free: usize,
}

impl<T> Arena<T>
where
    T: Copy,
{
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            len: 0,
            next_free: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: vec![Slot::Vacant; capacity],
            len: 0,
            next_free: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn insert(&mut self, value: T) -> usize {
        assert!(self.next_free <= self.slots.len());

        // Ensure that `self.next_free` points inside `self.slots`. If it's out of bounds, extend
        // the vector by adding an empty slot to the end.
        if self.next_free == self.slots.len() {
            self.slots.push(Slot::Vacant);
        }

        // After ensuring `self.next_free` is a valid index, occupy the slot with the given value
        // and increment the length. Keep track of the insertion index as it's the return value.
        self.slots[self.next_free].occupy(value);
        self.len += 1;
        let insertion_index = self.next_free;

        // Increment `self.next_free` in a while loop to find the next vacant slot. The loop is
        // broken under either of two conditions:
        //
        // * A vacant slot is found, in which case `Slot::Occupied(_)` fails to match.
        // `self.next_free` is the index of the vacant slot and the next call to `try_insert` will
        // succeed.
        //
        // * No vacant slot is found, in which case `Some(_)` fails to match. `self.next_free` is
        // beyond the range of valid indices and the next call to `try_insert` will return an
        // error, unless elements are pruned.
        while let Some(Slot::Occupied(_)) = self.slots.get(self.next_free) {
            self.next_free += 1;
        }

        insertion_index
    }

    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut last_seen_vacant: usize = self.capacity();

        for (index, slot) in self.slots.iter_mut().enumerate().rev() {
            match slot {
                Slot::Vacant => {
                    last_seen_vacant = index;
                }
                Slot::Occupied(value) => {
                    if !predicate(value) {
                        slot.vacate();
                        self.len -= 1;
                        last_seen_vacant = index;
                    }
                }
            }
        }

        self.next_free = last_seen_vacant;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tier {
    index: u8,
}

impl Tier {
    const MAX: Tier = Tier::new(63);

    pub const fn new(index: u8) -> Self {
        Self { index }
    }

    pub const fn decrement(&self) -> Self {
        Self::new(self.index - 1)
    }

    pub const fn increment(&self) -> Self {
        Self::new(self.index + 1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    tier: Tier,
    index: usize,
}

impl Id {
    fn new(tier: Tier, index: usize) -> Self {
        Self { tier, index }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TieredArena<B> {
    leaves: Arena<Leaf<B>>,
    branches: Vec<Arena<Branch>>,
}

impl<B> TieredArena<B>
where
    B: BitSquare,
{
    pub fn new() -> Self {
        let mut result: Self = Default::default();

        let mut prev_id = result.register_leaf(Leaf::dead());
        let mut tier = Tier::new(B::LOG_SIDE_LEN + 1);

        while tier < Tier::MAX {
            let empty = Branch {
                tier,
                children: Grid2::repeat(prev_id),
                population: 0,
            };

            prev_id = result.register_branch(empty);
            tier = tier.increment();
        }

        result
    }

    pub fn register_leaf(&mut self, leaf: Leaf<B>) -> Id {
        let index = self.leaves.insert(leaf);
        Id::new(Tier::new(B::LOG_SIDE_LEN), index)
    }

    pub fn register_branch(&mut self, branch: Branch) -> Id {
        let arena_index = (branch.tier.index - B::LOG_SIDE_LEN) as usize;

        let index = self.branches[arena_index].len();
        self.branches[arena_index].insert(branch);

        Id::new(branch.tier, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena() {
        let mut arena = Arena::<char>::with_capacity(5);

        arena.insert('a');
        arena.insert('b');
        arena.insert('c');
        arena.insert('d');
        arena.insert('e');

        assert_eq!(arena.len(), 5);

        arena.retain(|&c| (c as u8) % 2 == 1);

        assert_eq!(arena.len(), 3);

        arena.insert('X');
        arena.insert('Y');
        arena.insert('Z');
    }
}
