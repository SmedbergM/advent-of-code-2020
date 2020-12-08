
pub struct BitSet {
    n: usize,
    bytes: Vec<u8>
}

fn bit_at(byte: u8, idx: usize) -> bool {
    (byte << idx).leading_ones() > 0
}

impl BitSet {
    pub fn new(n: usize) -> BitSet {
        let bytes = match (n / 8, n % 8) {
            (byte_len, 0) => vec![0; byte_len],
            (byte_len, _) => vec![0; byte_len + 1]
        };

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
}

#[cfg(test)]
mod bitset_spec {
    use super::*;

    #[test]
    fn get_test() {
        let bitset = BitSet {
            n: 9,
            bytes: vec!(0x82, 0x80)
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
            bytes: vec!(0x82, 0xef)
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
            bytes: vec!(0x82, 0xab)
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
}