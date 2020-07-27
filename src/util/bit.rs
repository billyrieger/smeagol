// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::grid::Grid2;

use std::{
    default::Default,
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, BitOr, BitXor, Not},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bit8x8(u64);

impl Bit8x8 {
    pub fn zeros() -> Self {
        Self(0)
    }

    pub fn get_bit(&self, index: usize) -> bool {
        self.0 & (1 << index) > 0
    }

    pub fn set_bit(&self, index: usize) -> Self {
        Self(self.0 & !(1 << index))
    }

    pub fn unset_bit(&self, index: usize) -> Self {
        Self(self.0 | (1 << index))
    }

    pub fn toggle_bit(&self, index: usize) -> Self {
        Self(self.0 ^ (1 << index))
    }
}
