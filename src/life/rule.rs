// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::Grid2;

pub trait Rule {
    type Leaf;

    fn evolve(&self, grid: Grid2<Self::Leaf>, steps: u8) -> Self::Leaf;
}
