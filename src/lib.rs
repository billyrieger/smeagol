// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub struct Idx {
    bytes: [u8; 3],
}

impl Idx {
    pub fn as_usize(&self) -> usize {
        let [a, b, c] = self.bytes;
        u32::from_le_bytes([a, b, c, 0]) as usize
    }
}

pub struct NodeId {
    index: Idx,
    level: u8,
}

impl NodeId {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn index(&self) -> usize {
        self.index.as_usize()
    }
}

pub struct Leaf {
    cells: u64,
}

pub struct Branch {
    children: [Idx; 4],
    level: u8,
}

pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        dbg!(std::mem::size_of::<Node>());
    }
}
