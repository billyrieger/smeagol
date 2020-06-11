// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(try_trait)]

pub mod life;
pub mod util;

pub type Error = std::option::NoneError;
pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use crate::{Error, Result};
}
