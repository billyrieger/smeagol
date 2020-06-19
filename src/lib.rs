// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod life;
pub(crate) mod util;

pub use life::quadtree::Tree;
pub use life::rule::{Leaf, Rule};

#[derive(Clone, Copy, Debug, Default)]
pub struct Error;

pub type Result<T> = std::result::Result<T, Error>;
