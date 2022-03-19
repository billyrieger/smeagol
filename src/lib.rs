// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! SMEAGOL

// Single Instruction Multiple Data in the standard library.
#![feature(portable_simd)]
// MaybeUninit methods on arrays.
#![feature(maybe_uninit_array_assume_init, maybe_uninit_uninit_array)]
// everything else
#![feature(generic_const_exprs, array_zip, try_blocks, split_array, array_try_map)]
#![allow(incomplete_features)]

pub mod life;
pub mod util;
