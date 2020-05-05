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
        let w = closure(Grid2([a, b, d, e]))?;
        let x = closure(Grid2([b, c, e, f]))?;
        let y = closure(Grid2([d, e, g, h]))?;
        let z = closure(Grid2([e, f, h, i]))?;

        Some(Grid2([w, x, y, z]))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid4<T>(pub [T; 16]);

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
        let r = closure(Grid2([a, b, e, f]))?;
        let s = closure(Grid2([b, c, f, g]))?;
        let t = closure(Grid2([c, d, g, h]))?;
        let u = closure(Grid2([e, f, i, j]))?;
        let v = closure(Grid2([f, g, j, k]))?;
        let w = closure(Grid2([g, h, k, l]))?;
        let x = closure(Grid2([i, j, m, n]))?;
        let y = closure(Grid2([j, k, n, o]))?;
        let z = closure(Grid2([k, l, o, p]))?;

        Some(Grid3([r, s, t, u, v, w, x, y, z]))
    }
}
