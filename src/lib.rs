// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod life;
pub mod store;

#[derive(Clone, Copy, Debug)]
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

    pub fn repeat(val: T) -> Self
    where
        T: Clone,
    {
        let [nw, ne, sw, se] = [val.clone(), val.clone(), val.clone(), val];
        Self { nw, ne, sw, se }
    }
}
