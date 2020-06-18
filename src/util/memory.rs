// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Copy, Debug)]
enum Slot<T> {
    Occupied(T),
    Vacant,
}

impl<T> Slot<T>
where
    T: Copy,
{
    fn occupy(&mut self, value: T) {
        *self = Slot::Occupied(value);
    }

    fn vacate(&mut self) {
        *self = Slot::Vacant;
    }
}

#[derive(Clone, Debug)]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    lookup: HashMap<T, usize>,
    next_free: usize,
}

impl<T> Arena<T>
where
    T: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            slots: vec![],
            lookup: HashMap::new(),
            next_free: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn len(&self) -> usize {
        self.lookup.len()
    }

    pub fn get(&self, index: usize) -> Option<T> {
        self.slots
            .get(index)
            .and_then(|slot| match slot {
                Slot::Vacant => None,
                Slot::Occupied(value) => Some(value),
            })
            .copied()
    }

    pub fn register(&mut self, value: T) -> usize {
        if let Some(&index) = self.lookup.get(&value) {
            return index;
        }

        assert!(self.next_free <= self.slots.len());

        // Ensure that `self.next_free` points inside `self.slots`. If it's out of bounds, extend
        // the vector by adding an empty slot to the end.
        if self.next_free == self.slots.len() {
            self.slots.push(Slot::Vacant);
        }

        let insertion_index = self.next_free;
        self.slots[insertion_index].occupy(value);
        self.lookup.insert(value, insertion_index);

        // Increment `self.next_free` in a while loop to find the next vacant slot. The loop is
        // broken under either of two conditions:
        //
        // * A vacant slot is found, in which case `Slot::Occupied(_)` fails to match.
        //
        // * No vacant slot is found, in which case `Some(_)` fails to match.
        while let Some(Slot::Occupied(_)) = self.slots.get(self.next_free) {
            self.next_free += 1;
        }

        insertion_index
    }

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&T) -> bool,
    {
        let mut last_seen_vacant: usize = self.slots.len();

        for (index, slot) in self.slots.iter_mut().enumerate().rev() {
            match slot {
                Slot::Vacant => {
                    last_seen_vacant = index;
                }
                Slot::Occupied(value) => {
                    if !predicate(value) {
                        slot.vacate();
                        last_seen_vacant = index;
                    }
                }
            }
        }

        self.next_free = last_seen_vacant;

        self.lookup.retain(|key, _| predicate(key));
    }
}
