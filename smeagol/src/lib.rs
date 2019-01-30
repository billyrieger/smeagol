#[cfg(feature = "import-rle")]
#[macro_use]
extern crate nom;

mod life;
mod node;
#[cfg(feature = "import-rle")]
mod rle;
mod store;

pub use self::{
    life::Life,
    node::Node,
    store::{NodeTemplate, Store},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Alive,
    Dead,
}

impl Cell {
    pub fn is_alive(self) -> bool {
        match self {
            Cell::Alive => true,
            Cell::Dead => false,
        }
    }
}
