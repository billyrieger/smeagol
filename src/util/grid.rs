// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::Result;

/// Four values arranged in a square.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid2<T> {
    /// The value in the top-left corner.
    pub nw: T,
    /// The value in the top-right corner.
    pub ne: T,
    /// The value in the bottom-left corner.
    pub sw: T,
    /// The value in the bottom-right corner.
    pub se: T,
}

/// ## Packing and unpacking
///
/// A `Grid2<T>` is nothing more than four values of type `T`, so it's simple to convert a
/// `Grid2<T>` into a `[T; 4]` and vice versa as long as a consistent ordering is used. I chose
/// `[northwest, northeast, southwest, southeast]` since it feels like reading in English: left to
/// right, top to bottom. Use [`pack`][Grid2::pack] to create a `Grid2<T>` and
/// [`unpack`][Grid2::unpack] to destructure it.
impl<T> Grid2<T> {
    /// Creates a `Grid2<T>` from a `[T; 4]` in order `[nw, ne, sw, se]`.
    pub fn pack([nw, ne, sw, se]: [T; 4]) -> Self {
        Self { nw, ne, sw, se }
    }

    /// Creates a `[T; 4]` from a `Grid2<T>` in order `[nw, ne, sw, se]`.
    pub fn unpack(self) -> [T; 4] {
        [self.nw, self.ne, self.sw, self.se]
    }
}

/// ## Creating a `Grid2<T>` from a cloneable `T`
///
/// In the special case that the grid values are all the same, it's more convenient to give a
/// single value than declare each value separately.
impl<T> Grid2<T>
where
    T: Clone,
{
    /// Creates a new `Grid2<T>` with all four values initialized to the given value.
    pub fn repeat(val: T) -> Self
    where
        T: Clone,
    {
        // don't clone a fourth time - use the value itself
        Self::pack([val.clone(), val.clone(), val.clone(), val])
    }
}

/// ## Functional programming.
///
/// TODO
impl<T> Grid2<T> {
    /// Creates a new grid by mapping each grid value to a new one.
    ///
    /// An analogous method in the standard library would be [`std::result::Result::map`].
    pub fn map<F, U>(self, f: F) -> Grid2<U>
    where
        F: Fn(T) -> U,
    {
        Grid2 {
            nw: f(self.nw),
            ne: f(self.ne),
            sw: f(self.sw),
            se: f(self.se),
        }
    }

    /// Creates a new grid by mapping each grid value to a new one using the given function.
    ///
    /// An analogous method in the standard library would be [`std::result::Result::map`].
    pub fn try_map<B, F>(self, f: F) -> Result<Grid2<B>>
    where
        F: Fn(T) -> Result<B>,
    {
        Ok(Grid2 {
            nw: f(self.nw)?,
            ne: f(self.ne)?,
            sw: f(self.sw)?,
            se: f(self.se)?,
        })
    }

    pub fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, T) -> B,
    {
        let init = f(init, self.nw);
        let init = f(init, self.ne);
        let init = f(init, self.sw);
        f(init, self.se)
    }

    pub fn try_fold<B, F>(self, init: B, mut f: F) -> Option<B>
    where
        F: FnMut(B, T) -> Option<B>,
    {
        let init = f(init, self.nw)?;
        let init = f(init, self.ne)?;
        let init = f(init, self.sw)?;
        f(init, self.se)
    }
}

impl<T> Grid2<Grid2<T>> {
    pub fn flatten(self) -> Grid4<T> {
        // a b c d
        // e f g h
        // i j k l
        // m n o p

        // Northwest quadrant:
        // a b
        // e f
        let [a, b, e, f] = self.nw.unpack();

        // Northeast quadrant:
        // c d
        // g h
        let [c, d, g, h] = self.ne.unpack();

        // Southwest quadrant:
        // i j
        // m n
        let [i, j, m, n] = self.sw.unpack();

        // Southeast quadrant:
        // k l
        // o p
        let [k, l, o, p] = self.se.unpack();

        // Put everything together in alphabetical order.
        Grid4([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
    }
}

pub struct Grid3<T>([T; 9]);

impl<T> Grid3<T>
where
    T: Copy,
{
    pub fn shrink<F>(self, mut func: F) -> Result<Grid2<T>>
    where
        F: FnMut(Grid2<T>) -> Result<T>,
    {
        // a b c
        // d e f
        // g h i
        let [a, b, c, d, e, f, g, h, i] = self.0;

        // a b
        // d e
        let northwest = func(Grid2::pack([a, b, d, e]))?;

        // b c
        // e f
        let northeast = func(Grid2::pack([b, c, e, f]))?;

        // d e
        // g h
        let southwest = func(Grid2::pack([d, e, g, h]))?;

        // e g
        // h i
        let southeast = func(Grid2::pack([e, f, h, i]))?;

        Ok(Grid2::pack([northwest, northeast, southwest, southeast]))
    }
}

pub struct Grid4<T>([T; 16]);

impl<T> Grid4<T>
where
    T: Copy,
{
    pub fn shrink<F>(self, mut func: F) -> Result<Grid3<T>>
    where
        F: FnMut(Grid2<T>) -> Result<T>,
    {
        // a b c d
        // e f g h
        // i j k l
        // m n o p
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = self.0;

        // a b
        // e f
        let northwest = func(Grid2::pack([a, b, e, f]))?;

        // b c
        // f g
        let north = func(Grid2::pack([b, c, f, g]))?;

        // c d
        // g h
        let northeast = func(Grid2::pack([b, c, f, g]))?;

        // e f
        // i j
        let west = func(Grid2::pack([e, f, i, j]))?;

        // f g
        // j k
        let center = func(Grid2::pack([f, g, j, k]))?;

        // g h
        // k l
        let east = func(Grid2::pack([g, h, k, l]))?;

        // i j
        // m n
        let southwest = func(Grid2::pack([i, j, m, n]))?;

        // j k
        // n o
        let south = func(Grid2::pack([j, k, n, o]))?;

        // k l
        // o p
        let southeast = func(Grid2::pack([k, l, o, p]))?;

        Ok(Grid3([
            northwest, north, northeast, west, center, east, southwest, south, southeast,
        ]))
    }
}
