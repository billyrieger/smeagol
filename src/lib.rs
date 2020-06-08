// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod rule;
pub mod store;

#[derive(Clone, Copy, Debug)]
pub struct Grid2<T>([T; 4]);

impl<T> Grid2<T>
where
    T: Copy,
{
    pub fn unpack(&self) -> [T; 4] {
        self.0
    }

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
}
