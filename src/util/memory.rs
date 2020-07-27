// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
    pub fn with_value(value: T) -> Self {
        let mut arena = Self {
            slots: Vec::new(),
            lookup: HashMap::new(),
            next_free: 0,
        };
        arena.register(value);
        arena
    }

    pub fn with_value_and_capacity(value: T, capacity: usize) -> Self {
        let mut arena = Self {
            slots: vec![Slot::Vacant; capacity],
            lookup: HashMap::new(),
            next_free: 0,
        };
        arena.register(value);
        arena
    }

    pub fn get(&self, index: usize) -> Option<T> {
        self.slots.get(index).and_then(|&slot| match slot {
            Slot::Occupied(value) => Some(value),
            Slot::Vacant => None,
        })
    }

    /// This is the only way to add values to the arena.
    pub fn register(&mut self, value: T) -> usize {
        // Check if the arena already contains the value, and return its index if so.
        if let Some(&index) = self.lookup.get(&value) {
            return index;
        }

        // Ensure that `self.next_free` points inside `self.slots`. If it's out of bounds, extend
        // the vector by adding an empty slot to the end.
        if self.next_free == self.slots.len() {
            self.slots.push(Slot::Vacant);
        }

        // Insert the value into the arena, and add the value to index lookup map.
        self.slots[self.next_free].occupy(value);
        self.lookup.insert(value, self.next_free);

        // Store the index where the value was inserted.
        let insertion_index = self.next_free;

        // Increment `self.next_free` in a while loop to find the next vacant slot. The loop is
        // broken under either of two conditions:
        //
        // 1. A vacant slot is found, in which case `Slot::Occupied(_)` fails to match.
        //    `self.next_free` will be a valid index pointing to a vacant slot.
        //
        // 2. No vacant slot is found, in which case `Some(_)` fails to match. `self.next_free`
        //    will be equal to `self.slots.len()`, an out-of-bounds index, but this is checked the
        //    next time `self.register()` is called.
        while let Some(Slot::Occupied(_)) = self.slots.get(self.next_free) {
            self.next_free += 1;
        }

        // Return the saved index.
        insertion_index
    }

    /// This is the only way to remove values from the arena.
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&T) -> bool,
    {
        let mut earliest_vacant: usize = self.next_free;

        for (index, slot) in self
            .slots
            .iter_mut() // Item = &mut Slot<T>
            .enumerate() // Item = (usize, &mut Slot<T>)
            .skip(1)
            .rev()
        {
            match slot {
                Slot::Vacant => {
                    earliest_vacant = index;
                }
                Slot::Occupied(value) => {
                    if !predicate(value) {
                        slot.vacate();
                        earliest_vacant = index;
                    }
                }
            }
        }

        self.next_free = earliest_vacant;

        self.lookup.retain(|key, _| predicate(key));
        self.lookup.shrink_to_fit();
    }
}
