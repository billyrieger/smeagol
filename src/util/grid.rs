// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::prelude::*;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grid2<T> {
    pub nw: T,
    pub ne: T,
    pub sw: T,
    pub se: T,
}

impl<T> Grid2<T> {
    pub fn pack([nw, ne, sw, se]: [T; 4]) -> Self {
        Self { nw, ne, sw, se }
    }

    pub fn unpack(self) -> [T; 4] {
        [self.nw, self.ne, self.sw, self.se]
    }

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
