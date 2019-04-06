use std::ops::Range;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

#[derive(Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct BitSet256 {
    bits: [u64; 4],
}

impl BitSet256 {
    pub fn set_all(&mut self) {
        self.bits[0] = u64::max_value();
        self.bits[1] = u64::max_value();
        self.bits[2] = u64::max_value();
        self.bits[3] = u64::max_value();
    }

    pub fn unset_all(&mut self) {
        self.bits[0] = 0;
        self.bits[1] = 0;
        self.bits[2] = 0;
        self.bits[3] = 0;
    }

    pub fn set(&mut self, n: usize) {
        debug_assert!(n < 256);
        self.bits[Self::getword(n)] |= Self::maskbit(n);
    }

    pub fn unset(&mut self, n: usize) {
        debug_assert!(n < 256);
        self.bits[Self::getword(n)] &= Self::unmaskbit(n);
    }

    pub fn set_range(&mut self, range: Range<usize>) {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end < 256);

        if range.start / 64 == range.end / 64 {
            let mut block = u64::max_value() << (range.start % 64);
            if range.end % 64 != 63 {
                block &= Self::maskbit(range.end + 1) - 1;
            }
            self.bits[range.start / 64] |= block;
            return;
        }

        let mut i = range.start;
        if i % 64 != 0 {
            let block = u64::max_value() << (i % 64);
            self.bits[i / 64] |= block;
            i = roundup_64(i);
        }
        while i + 63 <= range.end {
            self.bits[i / 64] = u64::max_value();
            i += 64;
        }
        if i <= range.end {
            debug_assert!(range.end - i < 63);
            self.bits[i / 64] |= Self::maskbit(range.end + 1) - 1;
        }
    }

    pub fn flip(&mut self, n: usize) {
        debug_assert!(n < 256);
        self.bits[Self::getword(n)] ^= Self::maskbit(n);
    }

    pub fn flip_all(&mut self) {
        self.bits[0] ^= u64::max_value();
        self.bits[1] ^= u64::max_value();
        self.bits[2] ^= u64::max_value();
        self.bits[3] ^= u64::max_value();
    }

    pub fn test(&self, n: usize) -> bool {
        debug_assert!(n < 256);
        self.bits[Self::getword(n)] & Self::maskbit(n) != 0
    }

    pub fn count(&self) -> usize {
        (self.bits[0].count_ones()
            + self.bits[1].count_ones()
            + self.bits[2].count_ones()
            + self.bits[3].count_ones()) as usize
    }

    pub fn none(&self) -> bool {
        self.bits[0] == 0 && self.bits[1] == 0 && self.bits[2] == 0 && self.bits[3] == 0
    }

    pub fn any(&self) -> bool {
        !self.none()
    }

    pub fn all(&self) -> bool {
        self.bits[0] == u64::max_value()
            && self.bits[1] == u64::max_value()
            && self.bits[2] == u64::max_value()
            && self.bits[3] == u64::max_value()
    }

    pub fn find_first(&self) -> usize {
        let mut pos = 0;
        for block in &self.bits {
            if *block != 0 {
                return pos + block.trailing_zeros() as usize;
            }
            pos += 64;
        }
        256
    }

    pub fn find_last(&self) -> usize {
        let mut pos = 256;
        for block in self.bits.iter().rev() {
            if *block != 0 {
                return pos - 1 - block.leading_zeros() as usize;
            }
            pos -= 64;
        }
        256
    }

    pub fn find_next(&self, last: usize) -> usize {
        if last >= 256 {
            return 256;
        }

        let i = Self::getword(last);
        let mut last_block = self.bits[i];
        if (last % 64) != 63 {
            last_block &= u64::max_value() << ((last % 64) + 1);
            if last_block > 0 {
                return i * 64 + last_block.trailing_zeros() as usize;
            }
        }

        let mut i = i + 1;
        while i < 4 {
            if self.bits[i] > 0 {
                return i * 64 + self.bits[i].trailing_zeros() as usize;
            }
            i += 1;
        }
        256
    }

    pub fn select(&self, n: usize) -> usize {
        debug_assert!(n < 256);

        let mut sum = 0;
        let mut pos = 0;
        for block in &self.bits {
            let next_sum = block.count_ones() as usize;
            if next_sum > n {
                let mut block = *block;
                while sum < n {
                    block &= block - 1;
                    sum += 1;
                }
                return pos + block.trailing_zeros() as usize;
            }
            pos += 64;
        }
        256
    }

    const fn getword(n: usize) -> usize {
        n / 64
    }

    const fn maskbit(n: usize) -> u64 {
        1 << (n % 64)
    }

    const fn unmaskbit(n: usize) -> u64 {
        0xfffffffffffffffe << (n % 64)
    }
}

impl BitAnd for BitSet256 {
    type Output = Self;

    fn bitand(mut self, rhs: Self) -> Self {
        self &= rhs;
        self
    }
}

impl BitAndAssign for BitSet256 {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits[0] &= rhs.bits[0];
        self.bits[1] &= rhs.bits[1];
        self.bits[2] &= rhs.bits[2];
        self.bits[3] &= rhs.bits[3];
    }
}

impl BitOr for BitSet256 {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self {
        self |= rhs;
        self
    }
}

impl BitOrAssign for BitSet256 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits[0] |= rhs.bits[0];
        self.bits[1] |= rhs.bits[1];
        self.bits[2] |= rhs.bits[2];
        self.bits[3] |= rhs.bits[3];
    }
}

impl BitXor for BitSet256 {
    type Output = Self;

    fn bitxor(mut self, rhs: Self) -> Self {
        self ^= rhs;
        self
    }
}

impl BitXorAssign for BitSet256 {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits[0] ^= rhs.bits[0];
        self.bits[1] ^= rhs.bits[1];
        self.bits[2] ^= rhs.bits[2];
        self.bits[3] ^= rhs.bits[3];
    }
}

#[inline]
const fn roundup_64(n: usize) -> usize {
    (n + 0x3f) & 0x3f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitset256_empty() {
        let b = BitSet256::default();
        assert!(b.none());
        assert_eq!(b.count(), 0);
    }

    #[test]
    fn bitset256_set_1() {
        let mut bs = BitSet256::default();
        for i in 0..256 {
            bs.unset_all();
            assert!(bs.none());

            bs.set(i);
            assert!(!bs.none());
            assert!(bs.test(i));
            dbg!(i);
            assert_eq!(bs.find_first(), i);
            assert_eq!(bs.find_next(i), 256);
            assert_eq!(bs.count(), 1);

            bs.unset(i);
            assert!(bs.none());
            assert!(!bs.test(i));
            assert_eq!(bs.find_first(), 256);
            assert_eq!(bs.count(), 0);
        }
    }

    #[test]
    fn bitset256_set_n() {
        let strides = [80, 17, 7, 3];
        let mut b = BitSet256::default();
        for &s in &strides {
            b.set(s);
        }
        assert_eq!(b.count(), strides.len());
    }
}
