// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! SMEAGOL

// SIMD in the standard library.
#![feature(portable_simd)]
// MaybeUninit methods on arrays and slices.
#![feature(
    maybe_uninit_array_assume_init,
    maybe_uninit_uninit_array,
    maybe_uninit_write_slice
)]
// everything else
#![feature(generic_const_exprs, array_zip, try_blocks, split_array, array_try_map)]
#![allow(incomplete_features)]

pub mod bitgrid;
pub mod quad;
// mod life;
pub mod util;
