use crate::node::{Leaf, Rule};
use std::{
    hash::{Hash, Hasher},
    ops::Index,
};
use tinyvec::{Array, ArrayVec};

#[derive(Clone, Copy, Default)]
pub struct Grid<A: Array>(ArrayVec<A>);

pub type Grid2x2<T> = Grid<[T; 4]>;
pub type Grid3x3<T> = Grid<[T; 9]>;
pub type Grid4x4<T> = Grid<[T; 16]>;

impl<A> Hash for Grid<A>
where
    A: Array,
    A::Item: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<A> Eq for Grid<A>
where
    A: Array,
    A::Item: Eq,
{
}

impl<A> PartialEq for Grid<A>
where
    A: Array,
    A::Item: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A> Grid<A>
where
    A: Array + Default,
{
    pub fn side_len() -> usize {
        (A::CAPACITY as f32).sqrt() as usize
    }

    pub fn pack(items: &[A::Item]) -> Self
    where
        A::Item: Clone,
    {
        let mut array = ArrayVec::default();
        array.extend_from_slice(items);
        Self(array)
    }

    pub fn unpack(&self) -> &[A::Item] {
        self.0.as_slice()
    }

    pub fn reduce<B, F>(self, mut f: F) -> Option<Grid<B>>
    where
        B: Array + Default,
        F: FnMut(Grid2x2<A::Item>) -> Option<B::Item>,
        A::Item: Copy,
    {
        let side_len = Grid::<A>::side_len();
        let new_side_len = Grid::<B>::side_len();
        assert_eq!(side_len, new_side_len + 1);

        let array = (0..(new_side_len * new_side_len))
            .map(|i| {
                let (row, col) = (i / new_side_len, i % new_side_len);
                let a = self[(row, col)];
                let b = self[(row, col + 1)];
                let c = self[(row + 1, col)];
                let d = self[(row + 1, col + 1)];
                f(Grid::pack(&[a, b, c, d]))
            })
            .collect::<Option<_>>()?;
        Some(Grid(array))
    }

    pub fn map<B, F>(self, f: F) -> Grid<B>
    where
        B: Array + Default,
        F: Fn(A::Item) -> B::Item,
    {
        assert_eq!(Grid::<A>::side_len(), Grid::<B>::side_len());
        Grid(self.0.into_iter().map(|x| f(x)).collect())
    }

    pub fn try_map<B, F>(self, f: F) -> Option<Grid<B>>
    where
        B: Array + Default,
        F: Fn(A::Item) -> Option<B::Item>,
    {
        assert_eq!(Grid::<A>::side_len(), Grid::<B>::side_len());
        let array = self.0.into_iter().map(|x| f(x)).collect::<Option<_>>()?;
        Some(Grid(array))
    }
}

impl<A> Index<(usize, usize)> for Grid<A>
where
    A: Array + Default,
{
    type Output = A::Item;

    fn index(&self, (row, col): (usize, usize)) -> &A::Item {
        let side_len = Self::side_len();
        &self.0[row * side_len + col]
    }
}

impl Grid2x2<Leaf> {
    pub fn jump(&self, rule: Rule) -> Leaf {
        todo!()
        // let a = self.0[0].jump(rule);
        // let b = self.north().jump(rule);
        // let c = self.0[1].jump(rule);
        // let d = self.west().jump(rule);
        // let e = self.center().jump(rule);
        // let f = self.east().jump(rule);
        // let g = self.0[2].jump(rule);
        // let h = self.south().jump(rule);
        // let i = self.0[3].jump(rule);

        // let mask_center = Bool8x8(0x0000_3C3C_3C3C_0000);
        // let combine_jumps = |nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf| {
        //     Leaf::new(
        //         Bool8x8::FALSE
        //             | (nw.alive & mask_center).up(2).left(2)
        //             | (ne.alive & mask_center).up(2).right(2)
        //             | (sw.alive & mask_center).down(2).left(2)
        //             | (se.alive & mask_center).down(2).right(2),
        //     )
        // };

        // let w = combine_jumps(a, b, d, e).jump(rule);
        // let x = combine_jumps(b, c, e, f).jump(rule);
        // let y = combine_jumps(d, e, g, h).jump(rule);
        // let z = combine_jumps(e, f, h, i).jump(rule);

        // combine_jumps(w, x, y, z)
    }

    fn join_horizontal(left: Leaf, right: Leaf) -> Leaf {
        todo!()
        // let mask_left = Bool8x8(0xFF00_FF00_FF00_FF00);
        // let mask_right = Bool8x8(0x00FF00_00FF_00FF_00FF);
        // Leaf::new(
        //     Bool8x8::FALSE | left.alive.left(4) & mask_left | right.alive.right(4) & mask_right,
        // )
    }

    fn join_vertical(top: Leaf, bottom: Leaf) -> Leaf {
        todo!()
        // let mask_top = Bool8x8(0xFFFF_FFFF_0000_0000);
        // let mask_bottom = Bool8x8(0x0000_0000_FFFF_FFFF);
        // Leaf::new(Bool8x8::FALSE | top.alive.up(4) & mask_top | bottom.alive.down(4) & mask_bottom)
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
        todo!()
        // let mask_nw = Bool8x8(0xF0F0_F0F0_0000_0000);
        // let mask_ne = Bool8x8(0x0F0F_0F0F_0000_0000);
        // let mask_sw = Bool8x8(0x0000_0000_F0F0_F0F0);
        // let mask_se = Bool8x8(0x0000_0000_0F0F_0F0F);

        // let center = Bool8x8::FALSE
        //     | self.0[0].alive.up(4).left(4) & mask_nw
        //     | self.0[1].alive.up(4).right(4) & mask_ne
        //     | self.0[2].alive.down(4).left(4) & mask_sw
        //     | self.0[3].alive.down(4).right(4) & mask_se;

        // Leaf::new(center)
    }
}
