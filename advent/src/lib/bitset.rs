use std::boxed::Box;

#[derive(Clone, Debug)]
pub struct BitSet {
    n: usize,
    bytes: Box<[u8]>
}

impl std::fmt::Display for BitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for byte in self.bytes.iter() {
            s.push_str(format!("{:08b}", byte).as_ref());
        }

        write!(f, "[n = {}] {}", self.n, &s[0..self.n])
    }
}

fn bit_at(byte: u8, idx: usize) -> bool {
    (byte << idx).leading_ones() > 0
}

impl BitSet {
    pub fn new(n: usize) -> BitSet {
        let bytes = match (n / 8, n % 8) {
            (byte_len, 0) => vec![0; byte_len],
            (byte_len, _) => vec![0; byte_len + 1]
        }.into_boxed_slice();

        BitSet { n, bytes }
    }


    pub fn get(&self, bit: usize) -> Option<bool> {
        if bit < self.n {
            let (byte_idx, bit_idx) = (bit / 8, bit % 8);
            self.bytes.get(byte_idx).map(|prev_byte| {
                bit_at(*prev_byte, bit_idx)
            })
        } else {
            None
        }
    }

    /// If `bit` is in range, return the previous value.
    pub fn set(&mut self, bit: usize) -> Option<bool> {
        if bit < self.n {
            let (byte_idx, bit_idx) = (bit / 8, bit % 8);
            self.bytes.get_mut(byte_idx).map(|prev_byte| {
                let prev_bit = bit_at(*prev_byte, bit_idx);
                *prev_byte |= 0x80 >> bit_idx;
                prev_bit
            })
        } else {
            None
        }
    }

    pub fn unset(&mut self, bit: usize) -> Option<bool> {
        if bit < self.n {
            let (byte_idx, bit_idx) = (bit / 8, bit % 8);
            self.bytes.get_mut(byte_idx).map(|prev_byte| {
                let prev_bit = bit_at(*prev_byte, bit_idx);
                *prev_byte &= 0xff - (0x80 >> bit_idx);
                prev_bit
            })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.iter().fold(0, |acc, b| acc + b.count_ones() as usize)
    }

    /// Returns the minimum set index in this bitset
    pub fn min(&self) -> Option<usize> {
        for (byte_idx, &byte) in self.bytes.iter().enumerate() {
            if byte > 0 {
                return Some(8*byte_idx + byte.leading_zeros() as usize)
            }
        }
        return None
    }
}

#[cfg(test)]
mod bitset_spec {
    use super::*;

    #[test]
    fn new_test() {
        let bitset = BitSet::new(8);
        assert_eq!(bitset.bytes.as_ref(), &[0]);

        let bitset = BitSet::new(9);
        assert_eq!(bitset.bytes.as_ref(), &[0, 0]);

        let bitset = BitSet::new(24);
        assert_eq!(bitset.bytes.as_ref(), &[0, 0, 0]);

        let bitset = BitSet::new(25);
        assert_eq!(bitset.bytes.as_ref(), &[0, 0, 0, 0]);
    }

    #[test]
    fn get_test() {
        let bitset = BitSet {
            n: 9,
            bytes: vec!(0x82, 0x80).into_boxed_slice()
        };
        assert_eq!(bitset.get(0), Some(true));
        assert_eq!(bitset.get(1), Some(false));
        assert_eq!(bitset.get(6), Some(true));
        assert_eq!(bitset.get(7), Some(false));
        assert_eq!(bitset.get(8), Some(true));
        assert_eq!(bitset.get(9), None);
        assert_eq!(bitset.get(17), None);
    }

    #[test]
    fn set_test() {
        let mut bitset = BitSet {
            n: 12,
            bytes: vec!(0x82, 0xef).into_boxed_slice()
        };
        assert_eq!(bitset.set(0), Some(true));
        assert_eq!(bitset.bytes[0], 0x82);
        assert_eq!(bitset.set(3), Some(false));
        assert_eq!(bitset.bytes[0], 0x92);

        assert_eq!(bitset.set(4), Some(false));
        assert_eq!(bitset.bytes[0], 0x9a);

        assert_eq!(bitset.set(10), Some(true));
        assert_eq!(bitset.bytes[1], 0xef);
        
        assert_eq!(bitset.set(11), Some(false));
        assert_eq!(bitset.bytes[1], 0xff);

        assert_eq!(bitset.set(12), None);
        assert_eq!(bitset.set(17), None);
    }

    #[test]
    fn unset_test() {
        let mut bitset = BitSet {
            n: 16,
            bytes: vec!(0x82, 0xab).into_boxed_slice()
        };

        assert_eq!(bitset.unset(0), Some(true));
        assert_eq!(bitset.bytes[0], 0x02);
        
        assert_eq!(bitset.unset(1), Some(false));
        assert_eq!(bitset.bytes[0], 0x02);

        assert_eq!(bitset.unset(6), Some(true));
        assert_eq!(bitset.bytes[0], 0);
        
        assert_eq!(bitset.unset(10), Some(true));
        assert_eq!(bitset.bytes[1], 0x8b);

        assert_eq!(bitset.unset(17), None);
    }

    #[test]
    fn len_test() {
        let mut bitset = BitSet::new(24);
        assert_eq!(bitset.len(), 0);

        bitset.set(1);
        bitset.set(3);
        bitset.set(7);

        assert_eq!(bitset.len(), 3);

        bitset.set(22);
        assert_eq!(bitset.len(), 4);

        bitset.unset(1);
        bitset.unset(3);
        bitset.unset(7);
        assert_eq!(bitset.len(), 1);
    }

    #[test]
    fn min_test() {
        let mut bitset = BitSet::new(15);
        assert_eq!(bitset.min(), None);

        bitset.set(0);
        assert_eq!(bitset.min(), Some(0));

        bitset.set(1);
        assert_eq!(bitset.min(), Some(0));

        bitset.unset(0);
        assert_eq!(bitset.min(), Some(1));

        bitset.set(13);
        assert_eq!(bitset.min(), Some(1));

        bitset.unset(1);
        assert_eq!(bitset.min(), Some(13));
    }
}