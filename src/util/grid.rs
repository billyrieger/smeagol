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
/// `Grid2<T>` into a `[T; 4]` and vice versa. Well, as long as a consistent ordering is used. I
/// chose `[northwest, northeast, southwest, southeast]` since it feels like reading in English:
/// left to right, top to bottom. Use [`pack`][Grid2::pack] to create a `Grid2<T>` and
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
