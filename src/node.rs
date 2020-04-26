// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::{new_key_type, SecondaryMap, SlotMap};
use std::{
    collections::HashMap,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn level(&self) -> Level {
        match self {
            Self::Leaf(_) => Level(3),
            Self::Branch(branch) => branch.level,
        }
    }

    /// Returns the number of alive cells in the `Node`.
    pub fn population(&self) -> u128 {
        match self {
            Self::Leaf(leaf) => leaf.alive.0.count_ones() as u128,
            Self::Branch(branch) => branch.population,
        }
    }
}

/// An 8 by 8 grid of dead or alive cells in a cellular automaton.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Leaf {
    alive: Bool8x8,
}

impl Leaf {
    /// # Examples
    ///
    /// ```
    /// # use smeagol::node::{Bool8x8, Leaf};
    /// let all_dead = Leaf::new(Bool8x8::FALSE);
    /// let all_alive = Leaf::new(Bool8x8::TRUE);
    ///
    /// let glider = Leaf::new(Bool8x8(0x0000_1008_3800_0000));
    /// ```
    pub const fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    fn step(&self, rule: Rule) -> Self {
        let (alive, dead) = (self.alive, self.alive.not());

        let alive_neighbor_count = Bool8x8::sum(&[
            alive.up(1),
            alive.down(1),
            alive.left(1),
            alive.right(1),
            alive.up(1).left(1),
            alive.up(1).right(1),
            alive.down(1).left(1),
            alive.left(1).right(1),
        ]);

        let result = Bool8x8::FALSE
            | dead & alive_neighbor_count[0] & rule.birth[0]
            | dead & alive_neighbor_count[1] & rule.birth[1]
            | dead & alive_neighbor_count[2] & rule.birth[2]
            | dead & alive_neighbor_count[3] & rule.birth[3]
            | dead & alive_neighbor_count[4] & rule.birth[4]
            | dead & alive_neighbor_count[5] & rule.birth[5]
            | dead & alive_neighbor_count[6] & rule.birth[6]
            | dead & alive_neighbor_count[7] & rule.birth[7]
            | dead & alive_neighbor_count[8] & rule.birth[8]
            | alive & alive_neighbor_count[0] & rule.survival[0]
            | alive & alive_neighbor_count[1] & rule.survival[1]
            | alive & alive_neighbor_count[2] & rule.survival[2]
            | alive & alive_neighbor_count[3] & rule.survival[3]
            | alive & alive_neighbor_count[4] & rule.survival[4]
            | alive & alive_neighbor_count[5] & rule.survival[5]
            | alive & alive_neighbor_count[6] & rule.survival[6]
            | alive & alive_neighbor_count[7] & rule.survival[7]
            | alive & alive_neighbor_count[8] & rule.survival[8];

        Self::new(result)
    }

    fn jump(&self, rule: Rule) -> Self {
        self.step(rule).step(rule)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Branch {
    children: Grid2x2<NodeId>,
    level: Level,
    population: u128,
}

/// A description of how one state of a cellular automaton transitions into the next.
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    birth: [Bool8x8; 9],
    survival: [Bool8x8; 9],
}

impl Rule {
    /// Creates a new `Rule` in B/S notation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use smeagol::node::Rule;
    /// // Conway's Game of Life: B3/S23
    /// let life = Rule::new(&[3], &[2, 3]);
    /// ```
    ///
    /// [B/S notation]: https://www.conwaylife.com/wiki/Rulestring#B.2FS_notation
    pub fn new(birth: &[usize], survival: &[usize]) -> Self {
        Self {
            birth: Self::make_rule(birth),
            survival: Self::make_rule(survival),
        }
    }

    fn make_rule(neighbors: &[usize]) -> [Bool8x8; 9] {
        let mut result = [Bool8x8::FALSE; 9];
        for &i in neighbors.iter().filter(|&&i| i < 9) {
            result[i] = Bool8x8::TRUE;
        }
        result
    }
}

#[derive(Clone, Debug)]
pub struct Quadtree {
    rule: Rule,
    id_lookup: HashMap<Node, NodeId>,
    nodes: SlotMap<NodeId, Node>,
    steps: SecondaryMap<NodeId, NodeId>,
    jumps: SecondaryMap<NodeId, NodeId>,
}

impl Quadtree {
    pub fn make_leaf(&mut self, leaf: Leaf) -> Option<NodeId> {
        Some(self.get_id(Node::Leaf(leaf)))
    }

    pub fn make_inner(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        let [a, b, c, d] = children.try_map(|id| self.get_node(id))?.unpack();
        let level = a.level();
        debug_assert_eq!(level, b.level());
        debug_assert_eq!(level, c.level());
        debug_assert_eq!(level, d.level());
        let population = a.population() + b.population() + c.population() + d.population();
        let node = Node::Branch(Branch {
            children,
            level,
            population,
        });
        Some(self.get_id(node))
    }

    pub fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id).copied()
    }

    pub fn jump(&mut self, children: Grid2x2<NodeId>) -> Option<NodeId> {
        match children.try_map(|id| self.get_node(id))?.unpack() {
            [Node::Leaf(w), Node::Leaf(x), Node::Leaf(y), Node::Leaf(z)] => {
                let grid2x2 = Grid2x2::pack([w, x, y, z]);
                self.make_leaf(grid2x2.jump(self.rule))
            }

            [Node::Branch(w), Node::Branch(x), Node::Branch(y), Node::Branch(z)] => {
                let grandchildren = Grid2x2::pack([w, x, y, z]).map(|branch| branch.children);
                let grid4x4 = Grid4x4::flatten(grandchildren);
                let grid2x2 = grid4x4.reduce(|x| self.jump(x))?.reduce(|x| self.jump(x))?;
                self.make_inner(grid2x2)
            }

            _ => None,
        }
    }

    fn get_id(&mut self, node: Node) -> NodeId {
        self.id_lookup.get(&node).copied().unwrap_or_else(|| {
            let id = self.nodes.insert(node);
            self.id_lookup.insert(node, id);
            id
        })
    }
}

/// A `u64` interpreted as a grid of boolean values.
///
/// # Bit layout
///
/// The following diagram shows the layout of the bits of a `u64` to make a
/// square grid. The most significant bit, `1 << 63`, is in the upper-left corner
/// and the least significant bit, `1 << 0`, is in the bottom-right. Each row of the grid
/// corresponds to one contiguous byte of the `u64`.
///
/// ```text
/// ┌────┬────┬────┬────┬────┬────┬────┬────┐
/// │ 63 │ 62 │ 61 │ 60 │ 59 │ 58 │ 57 │ 56 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 55 │ 54 │ 53 │ 52 │ 51 │ 50 │ 49 │ 48 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 47 │ 46 │ 45 │ 44 │ 43 │ 42 │ 41 │ 40 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 39 │ 38 │ 37 │ 36 │ 35 │ 34 │ 33 │ 32 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 31 │ 30 │ 29 │ 28 │ 27 │ 26 │ 25 │ 24 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 23 │ 22 │ 21 │ 20 │ 19 │ 18 │ 17 │ 16 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │ 15 │ 14 │ 13 │ 12 │ 11 │ 10 │  9 │  8 │
/// ├────┼────┼────┼────┼────┼────┼────┼────┤
/// │  7 │  6 │  5 │  4 │  3 │  2 │  1 │  0 │
/// └────┴────┴────┴────┴────┴────┴────┴────┘
/// ```
///
/// # Examples
///
/// ```
/// # use smeagol::node::Bool8x8;
/// let uppercase_f = Bool8x8::from_rows([0x00, 0x3C, 0x20, 0x38, 0x20, 0x20, 0x20, 0x00]);
/// // alternatively, Bool8x8(0x003C_2038_2020_2000);
/// ```
///
/// ```text
/// ┏━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┓
/// ┃ ░░   ░░   ░░   ░░ ╎ ░░   ░░   ░░   ░░ ┃   0 0 0 0   0 0 0 0  =  0x00
/// ┠                   ╎                   ┨
/// ┃ ░░   ░░   ██   ██ ╎ ██   ██   ░░   ░░ ┃   0 0 1 1   1 1 0 0  =  0x3C
/// ┠                   ╎                   ┨
/// ┃ ░░   ░░   ██   ░░ ╎ ░░   ░░   ░░   ░░ ┃   0 0 1 0   0 0 0 0  =  0x20
/// ┠                   ╎                   ┨
/// ┃ ░░   ░░   ██   ██ ╎ ██   ░░   ░░   ░░ ┃   0 0 1 1   1 0 0 0  =  0x38
/// ┠  ╌  ╌ ╌  ╌ ╌  ╌ ╌   ╌ ╌  ╌ ╌  ╌ ╌  ╌  ┨
/// ┃ ░░   ░░   ██   ░░ ╎ ░░   ░░   ░░   ░░ ┃   0 0 1 0   0 0 0 0  =  0x20
/// ┠                   ╎                   ┨
/// ┃ ░░   ░░   ██   ░░ ╎ ░░   ░░   ░░   ░░ ┃   0 0 1 0   0 0 0 0  =  0x20
/// ┠                   ╎                   ┨
/// ┃ ░░   ░░   ██   ░░ ╎ ░░   ░░   ░░   ░░ ┃   0 0 1 0   0 0 0 0  =  0x20
/// ┠                   ╎                   ┨
/// ┃ ░░   ░░   ░░   ░░ ╎ ░░   ░░   ░░   ░░ ┃   0 0 0 0   0 0 0 0  =  0x00
/// ┗━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┛
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bool8x8(pub u64);

impl Bool8x8 {
    /// The `Bool8x8` where all elements are `false`.
    pub const FALSE: Self = Self(0);

    /// The `Bool8x8` where all elements are `true`.
    pub const TRUE: Self = Self(u64::MAX);

    pub fn from_rows(rows: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(rows))
    }

    /// Shifts the `Bool8x8` to the left by the given number of steps.
    pub fn left(&self, steps: u8) -> Self {
        Self(self.0 << steps)
    }

    /// Shifts the `Bool8x8` to the right by the given number of steps.
    pub fn right(&self, steps: u8) -> Self {
        Self(self.0 >> steps)
    }

    /// Shifts the `Bool8x8` up by the given number of steps.
    pub fn up(&self, steps: u8) -> Self {
        self.left(steps * 8)
    }

    /// Shifts the `Bool8x8` down by the given number of steps.
    pub fn down(&self, steps: u8) -> Self {
        self.right(steps * 8)
    }

    pub fn sum(addends: &[Self]) -> [Self; 9] {
        let [mut a, mut b, mut c, mut d] = [Bool8x8::FALSE; 4];

        // adds `addend` to `sum`, returning the carry
        let half_adder = |sum: &mut Self, addend: Self| {
            *sum ^= addend;
            *sum & addend
        };

        for &addend in addends {
            d |= half_adder(&mut c, half_adder(&mut b, half_adder(&mut a, addend)));
        }

        Self::finish_sum([a, b, c, d])
    }

    // separate function to preserve formatting
    #[rustfmt::skip]
    fn finish_sum(digits: [Self; 4]) -> [Self; 9] {
        let [a, b, c, d] = digits;
        [
            !d & !c & !b & !a, // 0000 = 0
            !d & !c & !b &  a, // 0001 = 1
            !d & !c &  b & !a, // 0010 = 2
            !d & !c &  b &  a, // 0011 = 3
            !d &  c & !b & !a, // 0100 = 4
            !d &  c & !b &  a, // 0101 = 5
            !d &  c &  b & !a, // 0110 = 6
            !d &  c &  b &  a, // 0111 = 7
             d & !c & !b & !a, // 1000 = 8
        ]
    }
}

impl BitAnd for Bool8x8 {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl BitAndAssign for Bool8x8 {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

impl BitOr for Bool8x8 {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for Bool8x8 {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitXor for Bool8x8 {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

impl BitXorAssign for Bool8x8 {
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

impl Not for Bool8x8 {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

/// A 2 by 2 grid of values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid2x2<T>([T; 4]);

impl<T> Grid2x2<T> {
    pub fn pack(grid: [T; 4]) -> Self {
        Self(grid)
    }

    pub fn unpack(self) -> [T; 4] {
        self.0
    }

    pub fn map<F, U>(self, f: F) -> Grid2x2<U>
    where
        F: Fn(T) -> U,
    {
        let [a, b, c, d] = self.unpack();
        Grid2x2::pack([f(a), f(b), f(c), f(d)])
    }

    pub fn try_map<F, U>(self, f: F) -> Option<Grid2x2<U>>
    where
        F: Fn(T) -> Option<U>,
    {
        let [a, b, c, d] = self.unpack();
        Some(Grid2x2([f(a)?, f(b)?, f(c)?, f(d)?]))
    }
}

/// A 3 by 3 grid of values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid3x3<T>([T; 9]);

impl<T> Grid3x3<T> {
    pub fn unpack(self) -> [T; 9] {
        self.0
    }

    pub fn reduce<F>(&self, mut map: F) -> Option<Grid2x2<T>>
    where
        F: FnMut(Grid2x2<T>) -> Option<T>,
        T: Copy,
    {
        let [a, b, c, d, e, f, g, h, i] = self.unpack();

        let w = map(Grid2x2([a, b, d, e]))?;
        let x = map(Grid2x2([b, c, e, f]))?;
        let y = map(Grid2x2([d, e, g, h]))?;
        let z = map(Grid2x2([e, f, h, i]))?;

        Some(Grid2x2::pack([w, x, y, z]))
    }
}

/// A 4 by 4 grid of values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid4x4<T>([T; 16]);

impl<T> Grid4x4<T> {
    pub fn pack(grid: [T; 16]) -> Self {
        Self(grid)
    }

    pub fn flatten(grid: Grid2x2<Grid2x2<T>>) -> Self {
        let [[a, b, e, f], [c, d, g, h], [i, j, m, n], [k, l, o, p]] =
            grid.map(|inner| inner.unpack()).unpack();
        Self::pack([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
    }

    pub fn unpack(self) -> [T; 16] {
        self.0
    }

    pub fn map<F, U>(&self, f: F) -> Grid4x4<U>
    where
        F: Fn(T) -> U,
        T: Copy,
    {
        let [k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z] = self.unpack();

        Grid4x4([
            f(k),
            f(l),
            f(m),
            f(n),
            f(o),
            f(p),
            f(q),
            f(r),
            f(s),
            f(t),
            f(u),
            f(v),
            f(w),
            f(x),
            f(y),
            f(z),
        ])
    }

    pub fn reduce<F>(&self, mut map: F) -> Option<Grid3x3<T>>
    where
        F: FnMut(Grid2x2<T>) -> Option<T>,
        T: Copy,
    {
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = self.unpack();

        let r = map(Grid2x2::pack([a, b, e, f]))?;
        let s = map(Grid2x2::pack([b, c, f, g]))?;
        let t = map(Grid2x2::pack([c, d, g, h]))?;
        let u = map(Grid2x2::pack([e, f, i, j]))?;
        let v = map(Grid2x2::pack([f, g, j, k]))?;
        let w = map(Grid2x2::pack([g, h, k, l]))?;
        let x = map(Grid2x2::pack([i, j, m, n]))?;
        let y = map(Grid2x2::pack([j, k, n, o]))?;
        let z = map(Grid2x2::pack([k, l, o, p]))?;

        Some(Grid3x3([r, s, t, u, v, w, x, y, z]))
    }
}

impl Grid2x2<Leaf> {
    fn jump(&self, rule: Rule) -> Leaf {
        let a = self.0[0].jump(rule);
        let b = self.north().jump(rule);
        let c = self.0[1].jump(rule);
        let d = self.west().jump(rule);
        let e = self.center().jump(rule);
        let f = self.east().jump(rule);
        let g = self.0[2].jump(rule);
        let h = self.south().jump(rule);
        let i = self.0[3].jump(rule);

        let mask_center = Bool8x8(0x0000_3C3C_3C3C_0000);
        let combine_jumps = |nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf| {
            Leaf::new(
                Bool8x8::FALSE
                    | (nw.alive & mask_center).up(2).left(2)
                    | (ne.alive & mask_center).up(2).right(2)
                    | (sw.alive & mask_center).down(2).left(2)
                    | (se.alive & mask_center).down(2).right(2),
            )
        };

        let w = combine_jumps(a, b, d, e).jump(rule);
        let x = combine_jumps(b, c, e, f).jump(rule);
        let y = combine_jumps(d, e, g, h).jump(rule);
        let z = combine_jumps(e, f, h, i).jump(rule);

        combine_jumps(w, x, y, z)
    }

    fn join_horizontal(left: Leaf, right: Leaf) -> Leaf {
        let mask_left = Bool8x8(0xFF00_FF00_FF00_FF00);
        let mask_right = Bool8x8(0x00FF00_00FF_00FF_00FF);
        Leaf::new(
            Bool8x8::FALSE | left.alive.left(4) & mask_left | right.alive.right(4) & mask_right,
        )
    }

    fn join_vertical(top: Leaf, bottom: Leaf) -> Leaf {
        let mask_top = Bool8x8(0xFFFF_FFFF_0000_0000);
        let mask_bottom = Bool8x8(0x0000_0000_FFFF_FFFF);
        Leaf::new(Bool8x8::FALSE | top.alive.up(4) & mask_top | bottom.alive.down(4) & mask_bottom)
    }

    fn north(&self) -> Leaf {
        Self::join_horizontal(self.0[0], self.0[1])
    }

    fn south(&self) -> Leaf {
        Self::join_horizontal(self.0[2], self.0[3])
    }

    fn east(&self) -> Leaf {
        Self::join_vertical(self.0[0], self.0[2])
    }

    fn west(&self) -> Leaf {
        Self::join_vertical(self.0[1], self.0[3])
    }

    fn center(&self) -> Leaf {
        let mask_nw = Bool8x8(0xF0F0_F0F0_0000_0000);
        let mask_ne = Bool8x8(0x0F0F_0F0F_0000_0000);
        let mask_sw = Bool8x8(0x0000_0000_F0F0_F0F0);
        let mask_se = Bool8x8(0x0000_0000_0F0F_0F0F);

        let center = Bool8x8::FALSE
            | self.0[0].alive.up(4).left(4) & mask_nw
            | self.0[1].alive.up(4).right(4) & mask_ne
            | self.0[2].alive.down(4).left(4) & mask_sw
            | self.0[3].alive.down(4).right(4) & mask_se;

        Leaf::new(center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adder_histogram() {
        let buckets = Bool8x8::sum(&[
            Bool8x8(0x00000000F0000000),
            Bool8x8(0x0000000FFF000000),
            Bool8x8(0x000000FFFFF00000),
            Bool8x8(0x00000FFFFFFF0000),
            Bool8x8(0x0000FFFFFFFFF000),
            Bool8x8(0x000FFFFFFFFFFF00),
            Bool8x8(0x00FFFFFFFFFFFFF0),
            Bool8x8(0x0FFFFFFFFFFFFFFF),
        ]);

        assert_eq!(Bool8x8(0x00000000F0000000), buckets[8]);
        assert_eq!(Bool8x8(0x0000000F0F000000), buckets[7]);
        assert_eq!(Bool8x8(0x000000F000F00000), buckets[6]);
        assert_eq!(Bool8x8(0x00000F00000F0000), buckets[5]);
        assert_eq!(Bool8x8(0x0000F0000000F000), buckets[4]);
        assert_eq!(Bool8x8(0x000F000000000F00), buckets[3]);
        assert_eq!(Bool8x8(0x00F00000000000F0), buckets[2]);
        assert_eq!(Bool8x8(0x0F0000000000000F), buckets[1]);
        assert_eq!(Bool8x8(0xF000000000000000), buckets[0]);
    }
}
