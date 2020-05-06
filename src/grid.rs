use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid2<T>(pub [T; 4]);

impl<T> Grid2<T>
where
    T: Copy,
{
    pub fn map<F, U>(&self, mut f: F) -> Grid2<U>
    where
        F: FnMut(T) -> U,
        U: Copy,
    {
        let [a, b, c, d] = self.0;
        Grid2([f(a), f(b), f(c), f(d)])
    }

    pub fn try_map<F, U>(&self, mut f: F) -> Option<Grid2<U>>
    where
        F: FnMut(T) -> Option<U>,
        U: Copy,
    {
        let [a, b, c, d] = self.0;
        Some(Grid2([f(a)?, f(b)?, f(c)?, f(d)?]))
    }
}

impl<T> Grid2<Grid2<T>>
where
    T: Copy,
{
    pub fn flatten(&self) -> Grid4<T> {
        // a b c d
        // e f g h
        // i j k l
        // m n o p
        let [[a, b, e, f], [c, d, g, h], [i, j, m, n], [k, l, o, p]] = self.map(|grid| grid.0).0;
        Grid4([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid3<T>(pub [T; 9]);

impl<T> Grid3<T>
where
    T: Copy,
{
    pub fn shrink<F, U>(&self, mut func: F) -> Option<Grid2<U>>
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
        let w = func(Grid2([a, b, d, e]))?;
        let x = func(Grid2([b, c, e, f]))?;
        let y = func(Grid2([d, e, g, h]))?;
        let z = func(Grid2([e, f, h, i]))?;

        Some(Grid2([w, x, y, z]))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid4<T>(pub [T; 16]);

impl<T> Grid4<T>
where
    T: Copy,
{
    pub fn shrink<F, U>(&self, mut func: F) -> Option<Grid3<U>>
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
        let r = func(Grid2([a, b, e, f]))?;
        let s = func(Grid2([b, c, f, g]))?;
        let t = func(Grid2([c, d, g, h]))?;
        let u = func(Grid2([e, f, i, j]))?;
        let v = func(Grid2([f, g, j, k]))?;
        let w = func(Grid2([g, h, k, l]))?;
        let x = func(Grid2([i, j, m, n]))?;
        let y = func(Grid2([j, k, n, o]))?;
        let z = func(Grid2([k, l, o, p]))?;

        Some(Grid3([r, s, t, u, v, w, x, y, z]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_map() {
        let map = |x: u32| if x % 2 == 1 { Some(x.pow(3)) } else { None };

        let odds = Grid2([1, 3, 5, 7]);
        assert_eq!(odds.try_map(map), Some(Grid2([1, 27, 125, 343])));

        let evens = Grid2([2, 4, 6, 8]);
        assert_eq!(evens.try_map(map), None);
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
        let sum = |grid: Grid2<u32>| -> Option<u32> { Some(grid.0.iter().sum()) };
        let ones = Grid4([1; 16]);

        let fours = ones.shrink(sum).unwrap();
        assert_eq!(fours, Grid3([4; 9]));

        let sixteens = fours.shrink(sum).unwrap();
        assert_eq!(sixteens, Grid2([16; 4]));
    }
}
