// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub struct Counter {
    bits: [u64; 4],
}

impl Counter {
    pub const fn new() -> Self {
        Self { bits: [0; 4] }
    }

    pub const fn add(self, i: u64) -> Self {
        let [a, b, c, d] = self.bits;

        let w = a ^ i; // sum
        let q = a & i; // carry

        let x = b ^ q; // sum
        let q = b & q; // carry

        let y = c ^ q; // sum
        let q = c & q; // carry

        let z = d | q; // saturing sum

        Self { bits: [w, x, y, z] }
    }

    #[rustfmt::skip]
    pub const fn finish(self) -> [u64; 9] {
        let [q0, q1, q2, q3] = self.bits;
        [
            !q3 & !q2 & !q1 & !q0, // 0000 = 0
            !q3 & !q2 & !q1 &  q0, // 0001 = 1
            !q3 & !q2 &  q1 & !q0, // 0010 = 2
            !q3 & !q2 &  q1 &  q0, // 0011 = 3
            !q3 &  q2 & !q1 & !q0, // 0100 = 4
            !q3 &  q2 & !q1 &  q0, // 0101 = 5
            !q3 &  q2 &  q1 & !q0, // 0110 = 6
            !q3 &  q2 &  q1 &  q0, // 0111 = 7
             q3 & !q2 & !q1 & !q0, // 1000 = 8
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let sums = Counter::new()
            .add(0x_FFFF_FFFF_FFFF_FFF0)
            .add(0x_0FFF_FFFF_FFFF_FF00)
            .add(0x_00FF_FFFF_FFFF_F000)
            .add(0x_000F_FFFF_FFFF_0000)
            .add(0x_0000_FFFF_FFF0_0000)
            .add(0x_0000_0FFF_FF00_0000)
            .add(0x_0000_00FF_F000_0000)
            .add(0x_0000_000F_0000_0000)
            .finish();

        assert_eq!(sums[0], 0x_0000_0000_0000_000F);
        assert_eq!(sums[1], 0x_F000_0000_0000_00F0);
        assert_eq!(sums[2], 0x_0F00_0000_0000_0F00);
        assert_eq!(sums[3], 0x_00F0_0000_0000_F000);
        assert_eq!(sums[4], 0x_000F_0000_000F_0000);
        assert_eq!(sums[5], 0x_0000_F000_00F0_0000);
        assert_eq!(sums[6], 0x_0000_0F00_0F00_0000);
        assert_eq!(sums[7], 0x_0000_00F0_F000_0000);
        assert_eq!(sums[8], 0x_0000_000F_0000_0000);
    }
}
