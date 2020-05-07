// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{grid::Grid2, Bool8x8, Rule};
use slotmap::new_key_type;

/// A measure of the size of a `Node`.
///
/// A node with level `Level(n)` represents a `2^n` by `2^n` square grid of dead or alive cells.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(pub u8);

impl Level {
    /// The maximum possible level, `Level(63)`.
    ///
    /// This ensures that the population of a node can be stored in a `u128`.
    pub const MAX_LEVEL: Self = Self(63);

    /// Attempts to increment the `Level`, returning `None` if the result would be too large.
    ///
    /// ```
    /// # use smeagol::node::Level;
    /// assert_eq!(Level(5).increment(), Some(Level(6)));
    /// assert_eq!(Level::MAX_LEVEL.increment(), None);
    /// ```
    pub fn increment(self) -> Option<Self> {
        if self < Self::MAX_LEVEL {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }

    pub fn max_steps(&self) -> u64 {
        1u64 << (self.0 - 2)
    }
}

new_key_type! {
    pub struct Id;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn children(&self) -> Option<Grid2<Id>> {
        match self {
            Self::Leaf(_) => None,
            Self::Branch(branch) => Some(branch.children),
        }
    }

    /// Returns the level of the `Node`.
    pub fn level(&self) -> Level {
        match self {
            Self::Leaf(_) => Level(3),
            Self::Branch(branch) => branch.level,
        }
    }

    /// Returns the number of alive cells in the `Node`.
    pub fn population(&self) -> u128 {
        match self {
            Self::Leaf(leaf) => u128::from(leaf.alive.0.count_ones()),
            Self::Branch(branch) => branch.population,
        }
    }
}

/// An 8 by 8 grid of dead or alive cells in a cellular automaton.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    pub alive: Bool8x8,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Branch {
    pub children: Grid2<Id>,
    pub level: Level,
    pub population: u128,
}

impl Leaf {
    /// Creates a new `Leaf` with the given alive cells.
    fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    /// Advances the leaf by 0 generations.
    fn idle(&self, _rule: Rule) -> Self {
        *self
    }

    /// Advances the leaf by 1 generation.
    fn step(&self, rule: Rule) -> Self {
        let alive = rule.step(self.alive);
        Self { alive }
    }

    /// Advances the leaf by 2 generations.
    fn jump(&self, rule: Rule) -> Self {
        self.step(rule).step(rule)
    }
}

impl Grid2<Leaf> {
    /// Advances the leaves by 0 generations.
    pub fn idle_idle(&self, rule: Rule) -> Leaf {
        self.apply(rule, Leaf::idle, Leaf::idle)
    }

    /// Advances the leaves by 1 generation.
    pub fn idle_step(&self, rule: Rule) -> Leaf {
        self.apply(rule, Leaf::idle, Leaf::step)
    }

    /// Advances the leaves by 2 generations.
    pub fn idle_jump(&self, rule: Rule) -> Leaf {
        self.apply(rule, Leaf::idle, Leaf::jump)
    }

    /// Advances the leaves by 3 generations.
    pub fn step_jump(&self, rule: Rule) -> Leaf {
        self.apply(rule, Leaf::step, Leaf::jump)
    }

    /// Advances the leaves by 4 generations.
    pub fn jump_jump(&self, rule: Rule) -> Leaf {
        self.apply(rule, Leaf::jump, Leaf::jump)
    }

    fn apply<F, G>(&self, rule: Rule, first: F, second: G) -> Leaf
    where
        F: Fn(&Leaf, Rule) -> Leaf,
        G: Fn(&Leaf, Rule) -> Leaf,
    {
        let first = |leaf| first(&leaf, rule);
        let second = |leaf| second(&leaf, rule);

        let debug = |leaf: Leaf| println!("{:016x}", leaf.alive.0);

        println!("input:");
        debug(self.0[0]);
        debug(self.0[1]);
        debug(self.0[2]);
        debug(self.0[3]);

        let Grid2([nw, ne, sw, se]) = *self;

        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . a a a a b b | b b c c c c . .
        // . . a a a a b b | b b c c c c . .
        // . . a a a a b b | b b c c c c . .
        // . . a a a a b b | b b c c c c . .
        // . . d d d d e e | e e f f f f . .
        // . . d d d d e e | e e f f f f . .
        // ----------------+----------------
        // . . d d d d e e | e e f f f f . .
        // . . d d d d e e | e e f f f f . .
        // . . g g g g h h | h h i i i i . .
        // . . g g g g h h | h h i i i i . .
        // . . g g g g h h | h h i i i i . .
        // . . g g g g h h | h h i i i i . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        let a = first(nw);
        let b = first(Self::join_horiz(nw, ne));
        let c = first(ne);
        let d = first(Self::join_vert(nw, sw));
        let e = first(Self::center(nw, ne, sw, se));
        println!("center:");
        debug(e);
        let f = first(Self::join_vert(ne, se));
        let g = first(sw);
        let h = first(Self::join_horiz(sw, se));
        let i = first(se);

        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . w w w w | x x x x . . . .
        // . . . . w w w w | x x x x . . . .
        // . . . . w w w w | x x x x . . . .
        // . . . . w w w w | x x x x . . . .
        // ----------------+----------------
        // . . . . y y y y | z z z z . . . .
        // . . . . y y y y | z z z z . . . .
        // . . . . y y y y | z z z z . . . .
        // . . . . y y y y | z z z z . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        let w = second(Self::join_centers(a, b, d, e));
        let x = second(Self::join_centers(b, c, e, f));
        let y = second(Self::join_centers(d, e, g, h));
        let z = second(Self::join_centers(e, f, h, i));

        Self::join_centers(w, x, y, z)
    }

    fn center(nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf) -> Leaf {
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // ----------------+----------------
        // . . . . c c c c | d d d d . . . .
        // . . . . c c c c | d d d d . . . .
        // . . . . c c c c | d d d d . . . .
        // . . . . c c c c | d d d d . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        let a = nw.alive.up(4).left(4) & Bool8x8::NORTHWEST;
        let b = ne.alive.up(4).right(4) & Bool8x8::NORTHEAST;
        let c = sw.alive.down(4).left(4) & Bool8x8::SOUTHWEST;
        let d = se.alive.down(4).right(4) & Bool8x8::SOUTHEAST;
        Leaf::new(a | b | c | d)
    }

    fn join_horiz(left: Leaf, right: Leaf) -> Leaf {
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        // . . . . a a a a | b b b b . . . .
        let a = left.alive.left(4) & Bool8x8::WEST;
        let b = right.alive.right(4) & Bool8x8::EAST;
        Leaf::new(a | b)
    }

    fn join_vert(top: Leaf, bottom: Leaf) -> Leaf {
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // a a a a a a a a
        // a a a a a a a a
        // a a a a a a a a
        // a a a a a a a a
        // ---------------
        // b b b b b b b b
        // b b b b b b b b
        // b b b b b b b b
        // b b b b b b b b
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        let a = top.alive.up(4) & Bool8x8::NORTH;
        let b = bottom.alive.down(4) & Bool8x8::SOUTH;
        Leaf::new(a | b)
    }

    fn join_centers(nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf) -> Leaf {
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . a a a a . . | . . b b b b . .
        // . . a a a a . . | . . b b b b . .
        // . . a a a a . . | . . b b b b . .
        // . . a a a a . . | . . b b b b . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // ----------------+----------------
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        // . . c c c c . . | . . d d d d . .
        // . . c c c c . . | . . d d d d . .
        // . . c c c c . . | . . d d d d . .
        // . . c c c c . . | . . d d d d . .
        // . . . . . . . . | . . . . . . . .
        // . . . . . . . . | . . . . . . . .
        let combined = Bool8x8::FALSE
            | nw.alive.up(2).left(2) & Bool8x8::NORTHWEST
            | ne.alive.up(2).right(2) & Bool8x8::NORTHEAST
            | sw.alive.down(2).left(2) & Bool8x8::SOUTHWEST
            | se.alive.down(2).right(2) & Bool8x8::SOUTHEAST;
        Leaf::new(combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glider() {
        let life = Rule::new(&[3], &[2, 3]);

        //      +-----------------+-----------------+
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x01 | . . . . . . . # | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | # . . . . . . . | 0x80
        //      +-----------------+-----------------+
        // 0x03 | . . . . . . # # | # . . . . . . . | 0x80
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        // 0x00 | . . . . . . . . | . . . . . . . . | 0x00
        //      +-----------------+-----------------+
        let nw_start = Bool8x8(0x0000_0000_0000_0100);
        let ne_start = Bool8x8(0x0000_0000_0000_0080);
        let sw_start = Bool8x8(0x0300_0000_0000_0000);
        let se_start = Bool8x8(0x8000_0000_0000_0000);
        let start = Grid2([nw_start, ne_start, sw_start, se_start]).map(Leaf::new);

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x10 | . . . # . . . .
        // 0x08 | . . . . # . . .
        // 0x38 | . . # # # . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let idle_leaf = Leaf::new(Bool8x8(0x0000_1008_3800_0000));
        assert_eq!(idle_leaf, Grid2::center(start.0[0], start.0[1], start.0[2], start.0[3]));

        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        // 0x08 | . . . . # . . .
        // 0x04 | . . . . . # . .
        // 0x1C | . . . # # # . .
        // 0x00 | . . . . . . . .
        // 0x00 | . . . . . . . .
        let jump_leaf = Leaf::new(Bool8x8(0x0000_0008_041C_0000));
        assert_eq!(idle_leaf.alive.down(1).right(1), jump_leaf.alive);

        let idled = start.idle_idle(life);
        let jumped = start.jump_jump(life);
        assert_eq!(idled, idle_leaf);
        assert_eq!(jumped, jump_leaf);
    }
}
