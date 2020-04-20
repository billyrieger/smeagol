pub struct Counter {
    bits: [u64; 4],
}

impl Counter {
    /// Returns an empty counter initialized to 0.
    pub const fn new() -> Self {
        Self { bits: [0; 4] }
    }

    pub const fn add(self, i: u64) -> Self {
        let [a, b, c, d] = self.bits;

        let b0 = a ^ i; // sum
        let q = a & i; // carry

        let b1 = b ^ q; // sum
        let q = b & q; // carry

        let b2 = c ^ q; // sum
        let q = c & q; // carry

        let b3 = d | q; // saturating sum

        Self {
            bits: [b0, b1, b2, b3],
        }
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
    fn histogram() {
        let buckets = Counter::new()
            //       |1234567876543210|
            .add(0x___0000000F00000000)
            .add(0x___000000FFF0000000)
            .add(0x___00000FFFFF000000)
            .add(0x___0000FFFFFFF00000)
            .add(0x___000FFFFFFFFF0000)
            .add(0x___00FFFFFFFFFFF000)
            .add(0x___0FFFFFFFFFFFFF00)
            .add(0x___FFFFFFFFFFFFFFF0)
            .finish();
        //           |1234567876543210|
        assert_eq!(0x_0000000F00000000, buckets[8]);
        assert_eq!(0x_000000F0F0000000, buckets[7]);
        assert_eq!(0x_00000F000F000000, buckets[6]);
        assert_eq!(0x_0000F00000F00000, buckets[5]);
        assert_eq!(0x_000F0000000F0000, buckets[4]);
        assert_eq!(0x_00F000000000F000, buckets[3]);
        assert_eq!(0x_0F00000000000F00, buckets[2]);
        assert_eq!(0x_F0000000000000F0, buckets[1]);
        assert_eq!(0x_000000000000000F, buckets[0]);
    }
}
