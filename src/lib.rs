mod node;
mod store;

pub use self::{
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
            Cell::Dead => true,
        }
    }
}
