use crate::{node::Leaf, Rule};
use std::{
    hash::{Hash, Hasher},
    mem::{transmute_copy, MaybeUninit},
};
use tinyvec::{Array, ArrayVec};

#[derive(Clone, Copy)]
pub struct Grid<T, const N: usize>([T; N]);

pub type Grid2<T> = Grid<T, 4>;
pub type Grid3<T> = Grid<T, 9>;
pub type Grid4<T> = Grid<T, 16>;

impl<T> Grid2<Grid2<T>>
where
    T: Copy,
{
    pub fn flatten(&self) -> Grid4<T> {
        let [w, x, y, z] = self.0;

        let [a, b, e, f] = w.0;
        let [c, d, g, h] = x.0;
        let [i, j, m, n] = y.0;
        let [k, l, o, p] = z.0;

        Grid([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
    }
}

impl<T> Grid4<T>
where
    T: Copy,
{
    pub fn shrink<F, U>(&self, mut closure: F) -> Option<Grid3<U>>
    where
        F: FnMut(Grid2<T>) -> Option<U>,
        U: Copy,
    {
        // a b c d
        // e f g h
        // i j k l
        // m n o p
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = self.0;

        // r s t
        // u v w
        // x y z
        let r = closure(Grid([a, b, e, f]))?;
        let s = closure(Grid([b, c, f, g]))?;
        let t = closure(Grid([c, d, g, h]))?;
        let u = closure(Grid([e, f, i, j]))?;
        let v = closure(Grid([f, g, j, k]))?;
        let w = closure(Grid([g, h, k, l]))?;
        let x = closure(Grid([i, j, m, n]))?;
        let y = closure(Grid([j, k, n, o]))?;
        let z = closure(Grid([k, l, o, p]))?;

        Some(Grid([r, s, t, u, v, w, x, y, z]))
    }
}

impl<T> Grid3<T>
where
    T: Copy,
{
    pub fn shrink<F, U>(&self, mut closure: F) -> Option<Grid2<U>>
    where
        F: FnMut(Grid2<T>) -> Option<U>,
        U: Copy,
    {
        // a b c
        // d e f
        // g h i
        let [a, b, c, d, e, f, g, h, i] = self.0;

        // w x
        // y z
        let w = closure(Grid([a, b, d, e]))?;
        let x = closure(Grid([b, c, e, f]))?;
        let y = closure(Grid([d, e, g, h]))?;
        let z = closure(Grid([e, f, h, i]))?;

        Some(Grid([w, x, y, z]))
    }
}

impl<T, const N: usize> Grid<T, N>
where
    T: Copy,
{
    pub fn map<F, U>(&self, mut closure: F) -> Grid<U, N>
    where
        F: FnMut(T) -> U,
        U: Copy,
    {
        let mut grid = [MaybeUninit::uninit(); N];
        for (&t, u) in self.0.iter().zip(grid.iter_mut()) {
            *u = MaybeUninit::new(closure(t));
        }
        let grid = unsafe { transmute_copy::<[MaybeUninit<U>; N], [U; N]>(&grid) };
        Grid(grid)
    }

    pub fn try_map<F, U>(self, closure: F) -> Option<Grid<U, N>>
    where
        F: Fn(T) -> Option<U>,
        U: Copy,
    {
        let mut grid = [MaybeUninit::uninit(); N];
        for (&t, u) in self.0.iter().zip(grid.iter_mut()) {
            *u = MaybeUninit::new(closure(t)?);
        }
        let grid = unsafe { transmute_copy::<[MaybeUninit<U>; N], [U; N]>(&grid) };
        Some(Grid(grid))
    }
}

pub trait SquareArray: Array + Default {
    const SIDE_LEN: usize;
}

impl<T> SquareArray for [T; 4]
where
    T: Default,
{
    const SIDE_LEN: usize = 2;
}

impl<T> SquareArray for [T; 9]
where
    T: Default,
{
    const SIDE_LEN: usize = 3;
}

impl<T> SquareArray for [T; 16]
where
    T: Default,
{
    const SIDE_LEN: usize = 4;
}

#[derive(Clone, Copy, Default)]
pub struct Gridd<A: SquareArray>(ArrayVec<A>);

pub type Grid2x2<T> = Gridd<[T; 4]>;
pub type Grid3x3<T> = Gridd<[T; 9]>;
pub type Grid4x4<T> = Gridd<[T; 16]>;

impl<T> Grid2x2<Grid2x2<T>>
where
    T: Copy + Default,
{
    pub fn flatten(&self) -> Grid4x4<T> {
        todo!()
        // match self.map(|grid| grid.unpack()).unpack() {
        //     &[&[a, b, e, f], &[c, d, g, h], &[i, j, m, n], &[k, l, o, p]] => {
        //         Grid::pack(&[a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
        //     }
        //     _ => unreachable!(),
        // }
    }
}

impl<A> Gridd<A>
where
    A: SquareArray,
    A::Item: Copy,
{
    pub fn pack(items: &[A::Item]) -> Self {
        assert_eq!(items.len(), A::CAPACITY);
        Self(items.iter().copied().collect())
    }

    pub fn unpack(&self) -> &[A::Item] {
        self.0.as_slice()
    }

    pub fn map<B, F>(self, f: F) -> Gridd<B>
    where
        B: SquareArray,
        F: Fn(A::Item) -> B::Item,
    {
        assert_eq!(A::SIDE_LEN, B::SIDE_LEN);
        let array = self.0.into_iter().map(|x| f(x)).collect();
        Gridd(array)
    }

    pub fn try_map<B, F>(self, f: F) -> Option<Gridd<B>>
    where
        B: SquareArray,
        F: Fn(A::Item) -> Option<B::Item>,
    {
        assert_eq!(A::SIDE_LEN, B::SIDE_LEN);
        let array = self.0.into_iter().map(|x| f(x)).collect::<Option<_>>()?;
        Some(Gridd(array))
    }

    pub fn shrink<B, F>(self, mut f: F) -> Option<Gridd<B>>
    where
        B: SquareArray,
        F: FnMut(Grid2x2<A::Item>) -> Option<B::Item>,
    {
        assert_eq!(A::SIDE_LEN, B::SIDE_LEN + 1);
        let array = (0..B::CAPACITY)
            .map(|i| {
                let (row, col) = (i / B::SIDE_LEN, i % B::SIDE_LEN);
                f(self.subgrid(row, col)?)
            })
            .collect::<Option<_>>()?;
        Some(Gridd(array))
    }

    fn subgrid(&self, row: usize, col: usize) -> Option<Grid2x2<A::Item>> {
        let a = self.get(row, col)?;
        let b = self.get(row, col + 1)?;
        let c = self.get(row + 1, col)?;
        let d = self.get(row + 1, col + 1)?;
        Some(Gridd::pack(&[a, b, c, d]))
    }

    fn get(&self, row: usize, col: usize) -> Option<A::Item> {
        if row < A::SIDE_LEN && col < A::SIDE_LEN {
            self.0.get(row * A::SIDE_LEN + col).copied()
        } else {
            None
        }
    }
}

impl<A> Eq for Gridd<A>
where
    A: SquareArray,
    A::Item: Eq,
{
}

impl<A> Hash for Gridd<A>
where
    A: SquareArray,
    A::Item: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<A> PartialEq for Gridd<A>
where
    A: SquareArray,
    A::Item: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Grid2x2<Leaf> {
    pub fn jump(&self, _rule: Rule) -> Leaf {
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

    //     fn join_horizontal(left: Leaf, right: Leaf) -> Leaf {
    //         todo!()
    //         // let mask_left = Bool8x8(0xFF00_FF00_FF00_FF00);
    //         // let mask_right = Bool8x8(0x00FF00_00FF_00FF_00FF);
    //         // Leaf::new(
    //         //     Bool8x8::FALSE | left.alive.left(4) & mask_left | right.alive.right(4) & mask_right,
    //         // )
    //     }

    //     fn join_vertical(top: Leaf, bottom: Leaf) -> Leaf {
    //         todo!()
    //         // let mask_top = Bool8x8(0xFFFF_FFFF_0000_0000);
    //         // let mask_bottom = Bool8x8(0x0000_0000_FFFF_FFFF);
    //         // Leaf::new(Bool8x8::FALSE | top.alive.up(4) & mask_top | bottom.alive.down(4) & mask_bottom)
    //     }

    //     fn north(&self) -> Leaf {
    //         Self::join_horizontal(self.0[0], self.0[1])
    //     }

    //     fn south(&self) -> Leaf {
    //         Self::join_horizontal(self.0[2], self.0[3])
    //     }

    //     fn east(&self) -> Leaf {
    //         Self::join_vertical(self.0[0], self.0[2])
    //     }

    //     fn west(&self) -> Leaf {
    //         Self::join_vertical(self.0[1], self.0[3])
    //     }

    //     fn center(&self) -> Leaf {
    //         todo!()
    //         // let mask_nw = Bool8x8(0xF0F0_F0F0_0000_0000);
    //         // let mask_ne = Bool8x8(0x0F0F_0F0F_0000_0000);
    //         // let mask_sw = Bool8x8(0x0000_0000_F0F0_F0F0);
    //         // let mask_se = Bool8x8(0x0000_0000_0F0F_0F0F);

    //         // let center = Bool8x8::FALSE
    //         //     | self.0[0].alive.up(4).left(4) & mask_nw
    //         //     | self.0[1].alive.up(4).right(4) & mask_ne
    //         //     | self.0[2].alive.down(4).left(4) & mask_sw
    //         //     | self.0[3].alive.down(4).right(4) & mask_se;

    //         // Leaf::new(center)
    //     }
}
