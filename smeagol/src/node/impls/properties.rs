use crate::{
    node::{NodeBase, MAX_LEVEL, Node, Store
}};

/// Methods for getting node properties.
impl Node {
    /// Returns the level of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::node::Store::new();
    ///
    /// let leaf = store.create_leaf(smeagol::Cell::Alive);
    /// assert_eq!(leaf.level(), 0);
    ///
    /// let empty = store.create_empty(4);
    /// assert_eq!(empty.level(), 4);
    /// ```
    pub fn level(&self) -> u8 {
        self.level
    }

    /// Returns the number of alive cells in the node.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::node::Store::new();
    ///
    /// let alive_leaf = store.create_leaf(smeagol::Cell::Alive);
    /// assert_eq!(alive_leaf.population(&store), 1);
    ///
    /// let empty = store.create_empty(4);
    /// assert_eq!(empty.population(&store), 0);
    /// ```
    pub fn population(&self, store: &Store) -> u128 {
        match self.base {
            NodeBase::Leaf { alive } => {
                if alive {
                    1
                } else {
                    0
                }
            }
            NodeBase::LevelOne { cells } => u128::from(cells.count_ones()),
            NodeBase::LevelTwo { cells } => u128::from(cells.count_ones()),
            _ => store.population(&self),
        }
    }

    /// Returns the minimum coordinate that can be used with the node.
    ///
    /// For a level `n` node, `n > 0`, the minimum coordinate is `-2^(n-1)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate itertools;
    /// let mut store = smeagol::node::Store::new();
    /// let node = store.create_random_filled(3, 0.5);
    ///
    /// // iterate over every cell in the node
    /// let min = node.min_coord();
    /// let max = node.max_coord();
    /// for (x, y) in itertools::iproduct!(min..=max, min..=max) {
    ///     let cell = node.get_cell(&store, x, y);
    ///     // do something
    /// }
    /// ```
    pub fn min_coord(&self) -> i64 {
        if self.level == 0 {
            0
        } else if self.level < MAX_LEVEL {
            -(1 << (self.level - 1))
        } else {
            i64::min_value()
        }
    }

    /// Returns the maximum coordinate that can be used with the node.
    ///
    /// For a level `n` node, `n > 0`, the maximum coordinate is `2^(n-1) - 1`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate itertools;
    /// let mut store = smeagol::node::Store::new();
    /// let node = store.create_random_filled(3, 0.5);
    ///
    /// // iterate over every cell in the node
    /// let min = node.min_coord();
    /// let max = node.max_coord();
    /// for (x, y) in itertools::iproduct!(min..=max, min..=max) {
    ///     let cell = node.get_cell(&store, x, y);
    ///     // do something
    /// }
    /// ```
    pub fn max_coord(&self) -> i64 {
        if self.level == 0 {
            0
        } else if self.level < MAX_LEVEL {
            (1 << (self.level - 1)) - 1
        } else {
            i64::max_value()
        }
    }

    pub fn min_alive_x(&self, store: &Store) -> Option<i64> {
        if self.population(store) == 0 {
            None
        } else {
            match self.base {
                NodeBase::Leaf { alive } => {
                    if alive {
                        Some(0)
                    } else {
                        None
                    }
                }
                NodeBase::LevelOne { .. } => {
                    let mut mins = Vec::with_capacity(4);
                    mins.extend(self.nw(store).min_alive_x(store).map(|x| x - 1));
                    mins.extend(self.sw(store).min_alive_x(store).map(|x| x - 1));
                    if !mins.is_empty() {
                        mins.into_iter().min()
                    } else {
                        mins.extend(self.ne(store).min_alive_x(store));
                        mins.extend(self.se(store).min_alive_x(store));
                        mins.into_iter().min()
                    }
                }
                _ => {
                    let mut mins = Vec::with_capacity(4);
                    let offset = 1 << (self.level - 2);
                    mins.extend(self.nw(store).min_alive_x(store).map(|x| x - offset));
                    mins.extend(self.sw(store).min_alive_x(store).map(|x| x - offset));
                    if !mins.is_empty() {
                        mins.into_iter().min()
                    } else {
                        mins.extend(self.ne(store).min_alive_x(store).map(|x| x + offset));
                        mins.extend(self.se(store).min_alive_x(store).map(|x| x + offset));
                        mins.into_iter().min()
                    }
                }
            }
        }
    }

    pub fn min_alive_y(&self, store: &Store) -> Option<i64> {
        if self.population(store) == 0 {
            None
        } else {
            match self.base {
                NodeBase::Leaf { alive } => {
                    if alive {
                        Some(0)
                    } else {
                        None
                    }
                }
                NodeBase::LevelOne { .. } => {
                    let mut mins = Vec::with_capacity(4);
                    mins.extend(self.ne(store).min_alive_y(store).map(|y| y - 1));
                    mins.extend(self.nw(store).min_alive_y(store).map(|y| y - 1));
                    if !mins.is_empty() {
                        mins.into_iter().min()
                    } else {
                        mins.extend(self.se(store).min_alive_y(store));
                        mins.extend(self.sw(store).min_alive_y(store));
                        mins.into_iter().min()
                    }
                }
                _ => {
                    let mut mins = Vec::with_capacity(4);
                    let offset = 1 << (self.level - 2);
                    mins.extend(self.ne(store).min_alive_y(store).map(|y| y - offset));
                    mins.extend(self.nw(store).min_alive_y(store).map(|y| y - offset));
                    if !mins.is_empty() {
                        mins.into_iter().min()
                    } else {
                        mins.extend(self.se(store).min_alive_y(store).map(|y| y + offset));
                        mins.extend(self.sw(store).min_alive_y(store).map(|y| y + offset));
                        mins.into_iter().min()
                    }
                }
            }
        }
    }

    pub fn max_alive_x(&self, store: &Store) -> Option<i64> {
        if self.population(store) == 0 {
            None
        } else {
            match self.base {
                NodeBase::Leaf { alive } => {
                    if alive {
                        Some(0)
                    } else {
                        None
                    }
                }
                NodeBase::LevelOne { .. } => {
                    let mut maxs = Vec::with_capacity(4);
                    maxs.extend(self.ne(store).max_alive_x(store));
                    maxs.extend(self.se(store).max_alive_x(store));
                    if !maxs.is_empty() {
                        maxs.into_iter().max()
                    } else {
                        maxs.extend(self.nw(store).max_alive_x(store).map(|x| x - 1));
                        maxs.extend(self.sw(store).max_alive_x(store).map(|x| x - 1));
                        maxs.into_iter().max()
                    }
                }
                _ => {
                    let mut maxs = Vec::with_capacity(4);
                    let offset = 1 << (self.level - 2);
                    maxs.extend(self.ne(store).max_alive_x(store).map(|x| x + offset));
                    maxs.extend(self.se(store).max_alive_x(store).map(|x| x + offset));
                    if !maxs.is_empty() {
                        maxs.into_iter().max()
                    } else {
                        maxs.extend(self.nw(store).max_alive_x(store).map(|x| x - offset));
                        maxs.extend(self.sw(store).max_alive_x(store).map(|x| x - offset));
                        maxs.into_iter().max()
                    }
                }
            }
        }
    }

    pub fn max_alive_y(&self, store: &Store) -> Option<i64> {
        if self.population(store) == 0 {
            None
        } else {
            match self.base {
                NodeBase::Leaf { alive } => {
                    if alive {
                        Some(0)
                    } else {
                        None
                    }
                }
                NodeBase::LevelOne { .. } => {
                    let mut maxs = Vec::with_capacity(4);
                    maxs.extend(self.se(store).max_alive_y(store));
                    maxs.extend(self.sw(store).max_alive_y(store));
                    if !maxs.is_empty() {
                        maxs.into_iter().max()
                    } else {
                        maxs.extend(self.ne(store).max_alive_y(store).map(|y| y - 1));
                        maxs.extend(self.nw(store).max_alive_y(store).map(|y| y - 1));
                        maxs.into_iter().max()
                    }
                }
                _ => {
                    let mut maxs = Vec::with_capacity(4);
                    let offset = 1 << (self.level - 2);
                    maxs.extend(self.se(store).max_alive_y(store).map(|y| y + offset));
                    maxs.extend(self.sw(store).max_alive_y(store).map(|y| y + offset));
                    if !maxs.is_empty() {
                        maxs.into_iter().max()
                    } else {
                        maxs.extend(self.ne(store).max_alive_y(store).map(|y| y - offset));
                        maxs.extend(self.nw(store).max_alive_y(store).map(|y| y - offset));
                        maxs.into_iter().max()
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_max_alive_coord() {
        let mut store = Store::new();

        let empty = store.create_empty(4);
        assert_eq!(empty.min_alive_x(&store), None);
        assert_eq!(empty.min_alive_y(&store), None);
        assert_eq!(empty.max_alive_x(&store), None);
        assert_eq!(empty.max_alive_y(&store), None);

        let node = store.create_empty(4).set_cells_alive(
            &mut store,
            &mut vec![(-1, -1), (-2, 4), (0, 2), (3, 5), (-5, 3)],
        );
        assert_eq!(node.min_alive_x(&store), Some(-5));
        assert_eq!(node.min_alive_y(&store), Some(-1));
        assert_eq!(node.max_alive_x(&store), Some(3));
        assert_eq!(node.max_alive_y(&store), Some(5));
    }
}
