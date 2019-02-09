use crate::{
    node::{NodeBase, MAX_LEVEL},
    Node, Store,
};

/// Methods for getting node properties.
impl Node {
    /// Returns the level of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
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
    /// let mut store = smeagol::Store::new();
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
    /// let mut store = smeagol::Store::new();
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
    /// let mut store = smeagol::Store::new();
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
}
