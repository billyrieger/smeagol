use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Copy, Debug)]
pub enum Slot<T> {
    Vacant,
    Occupied(T),
}

impl<T> Slot<T> {
    fn is_vacant(&self) -> bool {
        match self {
            Slot::Vacant => true,
            Slot::Occupied(_) => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Arena<T> {
    values: Vec<Slot<T>>,
    lookup: HashMap<T, usize>,
    next_free: usize,
}

impl<T> Arena<T>
where
    T: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            values: vec![Slot::Vacant],
            lookup: HashMap::new(),
            next_free: 0,
        }
    }

    pub fn get(&self, index: usize) -> Slot<T> {
        self.values[index]
    }

    pub fn insert(&mut self, value: T) -> usize {
        if let Some(&index) = self.lookup.get(&value) {
            index
        } else {
            let index = self.next_free;
            self.values[index] = Slot::Occupied(value);
            self.lookup.insert(value, index);
            self.find_next_free();
            index
        }
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        Self::retain_helper(&mut self.values, &mut self.lookup, f);
        self.next_free = 0;
        self.find_next_free();
    }

    fn retain_helper<F>(values: &mut Vec<Slot<T>>, lookup: &mut HashMap<T, usize>, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        lookup.retain(|value, index| {
            if f(value) {
                true
            } else {
                values[*index] = Slot::Vacant;
                false
            }
        })
    }

    fn find_next_free(&mut self) {
        let maybe_next_free: Option<usize> = self
            .values
            .iter()
            .enumerate()
            .skip(self.next_free)
            .skip_while(|(_, slot)| !slot.is_vacant())
            .map(|(index, _)| index)
            .nth(0);

        if let Some(index) = maybe_next_free {
            self.next_free = index;
        } else {
            self.next_free = self.values.len();
            self.values.push(Slot::Vacant);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut arena = Arena::<char>::new();
        assert_eq!(arena.insert('a'), 0);
        assert_eq!(arena.insert('b'), 1);
        assert_eq!(arena.insert('c'), 2);
        assert_eq!(arena.insert('d'), 3);

        arena.retain(|&c| c == 'a' || c == 'd');
    }
}
