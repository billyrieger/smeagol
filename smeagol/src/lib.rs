//! Library for simulating Conway's Game of Life using the HashLife algorithm.
//!
//! # Examples
//!
//! ```
//! // load an RLE pattern
//! let mut gosper_glider_gun = smeagol::Life::from_rle_pattern(
//!     b"
//! 24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8b
//! o3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!
//! ",
//! ).unwrap();
//!
//! // advance 1024 generations into the future
//! gosper_glider_gun.step(1024);
//!
//! // save the result
//! gosper_glider_gun.save_png(std::env::temp_dir().join("out.png"));
//! ```

mod life;
mod node;
mod store;

pub use self::{
    life::Life,
    node::Node,
    store::{NodeTemplate, Store},
};

/// A single cell in a Life board.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    /// An alive cell.
    Alive,
    /// A dead cell.
    Dead,
}

impl Cell {
    /// Returns true for `Cell::Alive` and false for `Cell::Dead`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(smeagol::Cell::Alive.is_alive());
    /// assert!(!smeagol::Cell::Dead.is_alive());
    /// ```
    pub fn is_alive(self) -> bool {
        match self {
            Cell::Alive => true,
            Cell::Dead => false,
        }
    }
}
