// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// A `2x2` square grid of values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid2<T>(pub [T; 4]);

impl<T> Grid2<T>
where
    T: Copy,
{
    pub fn nw(&self) -> T {
        self.0[0]
    }

    pub fn ne(&self) -> T {
        self.0[1]
    }

    pub fn sw(&self) -> T {
        self.0[2]
    }

    pub fn se(&self) -> T {
        self.0[3]
    }

    pub fn map<F, U>(&self, mut f: F) -> Grid2<U>
    where
        F: FnMut(T) -> U,
    {
        let [a, b, c, d] = self.0;
        Grid2([f(a), f(b), f(c), f(d)])
    }

    pub fn try_map<E, F, U>(&self, mut f: F) -> Result<Grid2<U>, E>
    where
        F: FnMut(T) -> Result<U, E>,
    {
        let [a, b, c, d] = self.0;
        Ok(Grid2([f(a)?, f(b)?, f(c)?, f(d)?]))
    }
}

impl<T> Grid2<Grid2<T>>
where
    T: Copy,
{
    pub fn flatten(&self) -> Grid4<T> {
        // a b | c d
        // e f | g h
        // ----+----
        // i j | k l
        // m n | o p
        let [[a, b, e, f], [c, d, g, h], [i, j, m, n], [k, l, o, p]] = self.map(|grid| grid.0).0;
        Grid4([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
    }
}

/// A `3x3` square grid of values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid3<T>(pub [T; 9]);

impl<T> Grid3<T>
where
    T: Copy,
{
    pub fn shrink<E, F, U>(&self, mut map: F) -> Result<Grid2<U>, E>
    where
        F: FnMut(Grid2<T>) -> Result<U, E>,
    {
        // a---b---c
        // | w | x |
        // d---e---f
        // | y | z |
        // g---h---i
        let [a, b, c, d, e, f, g, h, i] = self.0;
        let w = map(Grid2([a, b, d, e]))?;
        let x = map(Grid2([b, c, e, f]))?;
        let y = map(Grid2([d, e, g, h]))?;
        let z = map(Grid2([e, f, h, i]))?;
        Ok(Grid2([w, x, y, z]))
    }
}

/// A `4x4` square grid of values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid4<T>(pub [T; 16]);

impl<T> Grid4<T>
where
    T: Copy,
{
    pub fn center(&self) -> Grid2<T> {
        let [_a, _b, _c, _d, _e, f, g, _h, _i, j, k, _l, _m, _n, _o, _p] = self.0;
        Grid2([f, g, j, k])
    }

    pub fn shrink<E, F, U>(&self, mut func: F) -> Result<Grid3<U>, E>
    where
        F: FnMut(Grid2<T>) -> Result<U, E>,
    {
        // a---b---c---d
        // | r | s | t |
        // e---f---g---h
        // | u | v | w |
        // i---j---k---l
        // | x | y | z |
        // m---n---o---p
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = self.0;
        let r = func(Grid2([a, b, e, f]))?;
        let s = func(Grid2([b, c, f, g]))?;
        let t = func(Grid2([c, d, g, h]))?;
        let u = func(Grid2([e, f, i, j]))?;
        let v = func(Grid2([f, g, j, k]))?;
        let w = func(Grid2([g, h, k, l]))?;
        let x = func(Grid2([i, j, m, n]))?;
        let y = func(Grid2([j, k, n, o]))?;
        let z = func(Grid2([k, l, o, p]))?;
        Ok(Grid3([r, s, t, u, v, w, x, y, z]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn try_map() {
        let odd_cube = |x: u32| if x % 2 == 1 { Ok(x.pow(3)) } else { Err(()) };

        let odds = Grid2([1, 3, 5, 7]);
        assert_eq!(odds.try_map(odd_cube), Ok(Grid2([1, 27, 125, 343])));

        let not_odds = Grid2([1, 4, 3, 8]);
        assert_eq!(not_odds.try_map(odd_cube), Err(()));
    }

    #[test]
    fn flatten() {
        // 0 0 1 1
        // 0 0 1 1
        // 2 2 3 3
        // 2 2 3 3
        let zeros = Grid2([0; 4]);
        let ones = Grid2([1; 4]);
        let twos = Grid2([2; 4]);
        let threes = Grid2([3; 4]);
        let nested = Grid2([zeros, ones, twos, threes]);
        let flattened = nested.flatten();

        assert_eq!(
            flattened,
            Grid4([0, 0, 1, 1, 0, 0, 1, 1, 2, 2, 3, 3, 2, 2, 3, 3])
        );
    }

    #[test]
    fn shrink() {
        let sum = |grid: Grid2<u32>| -> Result<u32> { Ok(grid.0.iter().sum()) };
        let ones = Grid4([1; 16]);

        let fours = ones.shrink(sum).unwrap();
        assert_eq!(fours, Grid3([4; 9]));

        let sixteens = fours.shrink(sum).unwrap();
        assert_eq!(sixteens, Grid2([16; 4]));
    }
}
