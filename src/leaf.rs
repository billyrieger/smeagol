// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    bool8x8::{Adder, Bool8x8},
    Rule,
};

const CENTER_NW_LEAF: Macrocell<Bool8x8> = Macrocell {
    nw: Bool8x8(0x_00_00_3F_3F_3F_3F_3F_3F),
    ne: Bool8x8(0x_00_00_C0_C0_C0_C0_C0_C0),
    sw: Bool8x8(0x_3F_3F_00_00_00_00_00_00),
    se: Bool8x8(0x_C0_C0_00_00_00_00_00_00),
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Macrocell<T> {
    pub nw: T,
    pub ne: T,
    pub sw: T,
    pub se: T,
}

impl Macrocell<Leaf> {
    /// Foo bar.
    ///
    /// ```text
    /// ┏━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┳━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┯━━━━┓
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┠                   ╎                   ┃                   ╎                   ┨
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┠         ┌─────────────────────────────╂─────────┐         ╎                   ┨
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┠         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┨
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┠ ╌ ╌ ╌ ╌ │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │ ╌ ╌ ╌ ╌   ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┨
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┠         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┨
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┠         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┨
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┠         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┨
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┣━━━━━━━━━┿━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━┿━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┠         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┨
    /// ┃         │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░ ┃ ░░░░░░░ │         ╎                   ┃
    /// ┠         └─────────────────────────────╂─────────┘         ╎                   ┨
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┠                   ╎                   ┃                   ╎                   ┨
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┠ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌   ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┃ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌   ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ╌ ┨
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┠                   ╎                   ┃                   ╎                   ┨
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┠                   ╎                   ┃                   ╎                   ┨
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┠                   ╎                   ┃                   ╎                   ┨
    /// ┃                   ╎                   ┃                   ╎                   ┃
    /// ┗━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┻━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┷━━━━┛
    /// ```
    pub const fn center_nw_leaf(self) -> Leaf {
        let nw = self.nw.alive.and(CENTER_NW_LEAF.nw).up(2).left(2);
        let ne = self.ne.alive.and(CENTER_NW_LEAF.ne).up(2).right(6);
        let sw = self.sw.alive.and(CENTER_NW_LEAF.sw).down(6).left(2);
        let se = self.se.alive.and(CENTER_NW_LEAF.se).down(6).right(6);
        Leaf::new(Bool8x8::FALSE.or(nw).or(ne).or(sw).or(se))
    }
}

const BAR: Macrocell<Bool8x8> = Macrocell {
    nw: Bool8x8(0x_00_00_03_03_03_03_03_03),
    ne: Bool8x8(0x_00_00_FC_FC_FC_FC_FC_FC),
    sw: Bool8x8(0x_03_03_00_00_00_00_00_00),
    se: Bool8x8(0x_FC_FC_00_00_00_00_00_00),
};

const BUTT: Macrocell<Bool8x8> = Macrocell {
    nw: Bool8x8(0x_00_00_00_00_00_00_3F_3F),
    ne: Bool8x8(0x_00_00_00_00_00_00_C0_C0),
    sw: Bool8x8(0x_3F_3F_3F_3F_3F_3F_00_00),
    se: Bool8x8(0x_C0_C0_C0_C0_C0_C0_00_00),
};

const BALL: Macrocell<Bool8x8> = Macrocell {
    nw: Bool8x8(0x_00_00_00_00_00_00_03_03),
    ne: Bool8x8(0x_00_00_00_00_00_00_FC_FC),
    sw: Bool8x8(0x_03_03_03_03_03_03_00_00),
    se: Bool8x8(0x_FC_FC_FC_FC_FC_FC_00_00),
};

#[derive(Clone, Copy, Debug)]
pub struct Leaf {
    alive: Bool8x8,
}

impl Leaf {
    pub const fn new(alive: Bool8x8) -> Self {
        Self { alive }
    }

    pub const fn dead() -> Self {
        Self {
            alive: Bool8x8::FALSE,
        }
    }

    pub const fn alive() -> Self {
        Self {
            alive: Bool8x8::TRUE,
        }
    }

    pub const fn step(self, rule: Rule) -> Self {
        let cells = self.alive;

        let neighbors = Adder::new()
            .add(cells.up(1))
            .add(cells.down(1))
            .add(cells.left(1))
            .add(cells.right(1))
            .add(cells.up(1).left(1))
            .add(cells.up(1).right(1))
            .add(cells.down(1).left(1))
            .add(cells.left(1).right(1))
            .sum();

        let alive = cells;
        let dead = cells.not();

        let cells = Bool8x8::FALSE
            .or(dead.and(rule.birth[0]).and(neighbors[0]))
            .or(dead.and(rule.birth[1]).and(neighbors[1]))
            .or(dead.and(rule.birth[2]).and(neighbors[2]))
            .or(dead.and(rule.birth[3]).and(neighbors[3]))
            .or(dead.and(rule.birth[4]).and(neighbors[4]))
            .or(dead.and(rule.birth[5]).and(neighbors[5]))
            .or(dead.and(rule.birth[6]).and(neighbors[6]))
            .or(dead.and(rule.birth[7]).and(neighbors[7]))
            .or(dead.and(rule.birth[8]).and(neighbors[8]))
            .or(alive.and(rule.survival[0]).and(neighbors[0]))
            .or(alive.and(rule.survival[1]).and(neighbors[1]))
            .or(alive.and(rule.survival[2]).and(neighbors[2]))
            .or(alive.and(rule.survival[3]).and(neighbors[3]))
            .or(alive.and(rule.survival[4]).and(neighbors[4]))
            .or(alive.and(rule.survival[5]).and(neighbors[5]))
            .or(alive.and(rule.survival[6]).and(neighbors[6]))
            .or(alive.and(rule.survival[7]).and(neighbors[7]))
            .or(alive.and(rule.survival[8]).and(neighbors[8]));

        Self::new(cells)
    }
}

pub const fn evolve(cell: Macrocell<Bool8x8>, rule: Rule) -> Bool8x8 {
    let mask_nw = Bool8x8(0x0);
    let mask_ne = Bool8x8(0x0);
    let mask_sw = Bool8x8(0x0);
    let mask_se = Bool8x8(0x0);

    let e = cell.nw.and(mask_nw).up(2).left(6);
    let f = cell.ne.and(mask_ne).up(2).right(2);
    let g = cell.sw.and(mask_se).down(6).left(6);
    let h = cell.se.and(mask_sw).down(6).right(2);

    let efgh = Bool8x8::FALSE.or(e).or(f).or(g).or(h);

    let mask_nw = Bool8x8(0x0);
    let mask_ne = Bool8x8(0x0);
    let mask_sw = Bool8x8(0x0);
    let mask_se = Bool8x8(0x0);

    let i = cell.nw.and(mask_nw).up(6).left(2);
    let j = cell.ne.and(mask_ne).up(6).right(6);
    let k = cell.sw.and(mask_se).down(2).left(2);
    let l = cell.se.and(mask_sw).down(2).right(6);

    let ijkl = Bool8x8::FALSE.or(i).or(j).or(k).or(l);

    let mask_nw = Bool8x8(0x0);
    let mask_ne = Bool8x8(0x0);
    let mask_sw = Bool8x8(0x0);
    let mask_se = Bool8x8(0x0);

    let m = cell.nw.and(mask_nw).up(6).left(6);
    let n = cell.ne.and(mask_ne).up(6).right(2);
    let o = cell.sw.and(mask_sw).down(2).left(6);
    let p = cell.se.and(mask_se).down(2).right(2);

    let mnop = Bool8x8::FALSE.or(m).or(n).or(o).or(p);

    mnop
}

#[cfg(test)]
mod tests {}
