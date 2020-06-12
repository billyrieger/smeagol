// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(u8);

impl Level {
    pub const MAX: Level = Self(63);
}

impl From<Level> for u8 {
    fn from(lvl: Level) -> u8 {
        lvl.0
    }
}

impl TryFrom<u8> for Level {
    type Error = Error;

    fn try_from(n: u8) -> Result<Level> {
        if Level(n) <= Level::MAX {
            Ok(Level(n))
        } else {
            Err(Error)
        }
    }
}
