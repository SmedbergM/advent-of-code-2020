use std::io::prelude::*;

use std::collections::BTreeMap;

#[macro_use]
extern crate lazy_static;

use regex::Regex;


#[derive(Debug, PartialEq, Eq)]
struct Mask {
    zeros: u64, // has a 1 bit in each position where the mask forces a 0
    ones: u64 // has a 1 bit in each position where the mask forces a 1
}

impl Mask {
    // parameter `m` is just the masking string, e.g. "XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X"
    fn parse(m: &str) -> Mask {
        let mut zeros = 0;
        let mut ones = 0;

        for (idx, c) in m.chars().rev().enumerate() {
            match c {
                '0' => zeros |= 1 << idx,
                '1' => ones  |= 1 << idx,
                _ => () // do nothing
            }
        }

        Mask { zeros, ones }
    }

    const fn floating_bits(&self) -> u64 {
        !(self.zeros | self.ones) & ((1 << 36) - 1)
    }
}

fn set_mem(memory: &mut BTreeMap<u64, u64>, mask: &Mask, address: u64, value: u64) {
    let masked_value = (value | mask.ones) & !mask.zeros;
    memory.insert(address, masked_value);
}

struct Floater<'a> {
    base: u64,
    mask: &'a Mask,
    pos: u64 // the next item to be yielded, expressed as a  uint in the range 0..2^(number of floating bits)
}

impl<'a> Floater<'a> {
    fn new(base: u64, mask: &'a Mask) -> Floater<'a> {
        Floater { base, mask, pos: 0 }
    }

    fn explode(&self) -> u64 {
        let mut r = self.base & !self.mask.floating_bits() | self.mask.ones;
        let mut f = self.mask.floating_bits();
        let mut p = self.pos;

        while f > 0 && p > 0 {
            let f_bit_idx = 63 - f.leading_zeros();
            let f_bit: u64 = 1 << f_bit_idx;
            let p_bit_idx = f.count_ones() - 1;
            let p_bit: u64 = 1 << p_bit_idx;

            r |= (p & p_bit) << (f_bit_idx - p_bit_idx);
            f &= !f_bit;
            p &= !p_bit;
        }

        r
    }
}

impl Iterator for Floater<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let floating_bits = self.mask.floating_bits();
        if self.pos < (1 << floating_bits.count_ones()) {
            let item = self.explode();
            self.pos += 1;
            Some(item)
        } else {
            None
        }
    }
}

fn set_mem_2(memory: &mut BTreeMap<u64, u64>, mask: &Mask, address: u64, value: u64) {
    let floater = Floater::new(address, &mask);
    for a in floater {
        memory.insert(a, value);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    SetMask(Mask),
    SetMem { address: u64, value: u64 }
}

impl Instruction {
    fn parse(line: &str) -> Option<Instruction> {
        lazy_static! {
            static ref SET_MASK_PAT: Regex = Regex::new(r"mask = ([X01]{36})").unwrap();
            static ref SET_MEM_PAT: Regex = Regex::new(r"mem\[(\d+)\] = (\d+)").unwrap();
        }

        SET_MASK_PAT.captures(line).map(|caps| {
            let mask = Mask::parse(&caps[1]);
            Instruction::SetMask(mask)
        }).or_else(|| SET_MEM_PAT.captures(line).and_then(|caps| {
            u64::from_str_radix(&caps[1], 10).ok().and_then(|address| {
            u64::from_str_radix(&caps[2], 10).ok().map(|value| {
                Instruction::SetMem { address, value }
            }) })
        }))
    }
}

fn main() {
    let mut mask = Mask { zeros: 0, ones: 0 };
    let mut memory = BTreeMap::new();
    let mut memory_2 = BTreeMap::new();
    let stdin = std::io::stdin();
    for instruction in stdin.lock().lines().flatten().flat_map(|line| Instruction::parse(&line)) {
        match instruction {
            Instruction::SetMask(next_mask) => mask = next_mask,
            Instruction::SetMem { address, value } => {
                set_mem(&mut memory, &mask, address, value);
                set_mem_2(&mut memory_2, &mask, address, value);
            }
        }
    }
    println!("Part 1: Memory: {} addresses are set.", memory.len());
    println!("Part 2: Memory: {} addresses are set.", memory_2.len());
    let memory_sum = memory.values().fold(0, |acc, v| acc + v);
    let memory_sum_2 = memory_2.values().fold(0, |acc, v| acc + v);
    println!("Part 1: Sum of set values = {}", memory_sum);
    println!("Part 2: Sum of set values = {}", memory_sum_2);
}

#[cfg(test)]
mod day14_spec {
    use super::*;

    #[test]
    fn mask_parse_test() {
        let mask = Mask::parse("XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X");
        assert_eq!(mask, Mask { zeros: 2, ones: 64 });
    }

    #[test]
    fn set_mem_test() {
        let mut memory = BTreeMap::new();
        let mask = Mask { zeros: 2, ones: 64 };
        set_mem(&mut memory, &mask, 8, 11);
        assert_eq!(memory[&8], 73);

        set_mem(&mut memory, &mask, 7, 101);
        assert_eq!(memory[&7], 101);

        set_mem(&mut memory, &mask, 8, 0);
        assert_eq!(memory[&8], 64);
    }

    #[test]
    fn instruction_parse_test() {
        let mut expected_instruction = Instruction::SetMask(Mask { zeros: 2, ones: 64 });
        assert_eq!(Instruction::parse("mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X"), Some(expected_instruction));

        expected_instruction = Instruction::SetMem { address: 8, value: 11 };
        assert_eq!(Instruction::parse("mem[8] = 11"), Some(expected_instruction));

        expected_instruction = Instruction::SetMem { address: 7, value: 101 };
        assert_eq!(Instruction::parse("mem[7] = 101"), Some(expected_instruction));
    }

    #[test]
    fn floater_test() {
        let mask = Mask::parse("000000000000000000000000000000X1001X");
        assert_eq!(mask.floating_bits(), 33);
        let mut floater = Floater::new(42, &mask);
        assert_eq!(floater.next(), Some(26));
        assert_eq!(floater.next(), Some(27));
        assert_eq!(floater.next(), Some(58));
        assert_eq!(floater.next(), Some(59));
        assert_eq!(floater.next(), None);

        let mask = Mask::parse("00000000000000000000000000000000X0XX");
        let addresses: Vec<u64> = Floater::new(26, &mask).collect();
        assert_eq!(addresses, vec!(16, 17, 18, 19, 24, 25, 26, 27));
    }
}