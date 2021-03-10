// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Grid2<T> {
    pub nw: T,
    pub ne: T,
    pub sw: T,
    pub se: T,
}

impl<T> Grid2<T> {
    pub fn new([nw, ne, sw, se]: [T; 4]) -> Self {
        Self { nw, ne, sw, se }
    }

    pub fn repeat(t: T) -> Self
    where
        T: Copy,
    {
        Self::new([t; 4])
    }

    pub fn map<F, U>(self, mut f: F) -> Grid2<U>
    where
        F: FnMut(T) -> U,
    {
        Grid2 {
            nw: f(self.nw),
            ne: f(self.ne),
            sw: f(self.sw),
            se: f(self.se),
        }
    }
}

impl<T> From<[T; 4]> for Grid2<T> {
    fn from(array: [T; 4]) -> Grid2<T> {
        Self::new(array)
    }
}
