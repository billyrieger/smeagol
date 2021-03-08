use std::hash::Hash;

use bimap::maps::{HashKind, VecKind};
use bimap::Generic;
use fxhash::FxBuildHasher;

type BiMap<T> = Generic<usize, T, VecKind, HashKind<FxBuildHasher>>;

#[derive(Debug)]
pub struct Arena<T>
where
    T: Eq + Hash,
{
    bimap: BiMap<T>,
    next_free: usize,
}

impl<T> Arena<T>
where
    T: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            bimap: BiMap::new(),
            next_free: 0,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.bimap.get_left(&index)
    }

    pub fn insert(&mut self, value: T) -> usize {
        if let Some(&index) = self.bimap.get_right(&value) {
            index
        } else {
            let index = self.next_free;
            self.bimap.insert(index, value);
            self.find_next_free();
            index
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.bimap.retain_right(|_, x| f(x));
        self.next_free = 0;
        self.find_next_free();
    }

    fn find_next_free(&mut self) {
        while self.bimap.contains_left(&self.next_free) {
            self.next_free += 1;
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

        assert_eq!(arena.get(0), Some(&'a'));
        assert_eq!(arena.get(1), None);
        assert_eq!(arena.get(2), None);
        assert_eq!(arena.get(3), Some(&'d'));

        assert_eq!(arena.insert('e'), 1);
        assert_eq!(arena.insert('f'), 2);
        assert_eq!(arena.insert('a'), 0);
        assert_eq!(arena.insert('g'), 4);
    }
}
