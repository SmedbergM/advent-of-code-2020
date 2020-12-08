use std::io;
use std::io::prelude::*;

#[macro_use]
extern crate lazy_static;
use regex::Regex;

use advent::bitset::BitSet;

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    Nop,
    Acc(i32),
    Jmp(i32)
}

impl Instruction {
    fn parse(line: &str) -> Option<Instruction> {
        lazy_static! {
            static ref INSTRUCTION_PAT: Regex = Regex::new(r"(\w{3}) ([+-]\d+)").unwrap();
        }

        INSTRUCTION_PAT.captures(line).and_then(|cap| {
            i32::from_str_radix(&cap[2], 10).ok().and_then(|n| {
                match &cap[1] {
                    "nop" => Some(Instruction::Nop),
                    "acc" => Some(Instruction::Acc(n)),
                    "jmp" => Some(Instruction::Jmp(n)),
                    _ => None
                }
            })
        })
    }
}

struct HandheldGameConsole {
    instructions: Vec<Instruction>,
    instruction_ptr: usize,
    accumulator: i32,
}

impl HandheldGameConsole {
    fn parse<J>(j: &mut J) -> HandheldGameConsole
    where J: Iterator<Item=String> {
        let instructions = j.flat_map(|line| Instruction::parse(&line)).collect();
        HandheldGameConsole {
            instructions, instruction_ptr: 0, accumulator: 0
        }
    }

    fn step(&mut self) {
        if let Some(instruction) = self.instructions.get(self.instruction_ptr) {
            match instruction {
                Instruction::Nop => self.instruction_ptr += 1,
                Instruction::Acc(x) => {
                    self.accumulator += x;
                    self.instruction_ptr += 1;
                },
                Instruction::Jmp(x) => {
                    self.instruction_ptr = wrapping_add(self.instruction_ptr, *x);
                }
            }
        } else {
            eprintln!("Attempt to access instruction out of bounds at {}!", self.instruction_ptr);
        }
    }
}

fn find_repeated_instruction(console: &mut HandheldGameConsole) {
    let mut executed_instructions = BitSet::new(console.instructions.len());
    while let Some(false) = executed_instructions.get(console.instruction_ptr) {
        executed_instructions.set(console.instruction_ptr);
        console.step();
    }
}

fn wrapping_add(lhs: usize, rhs: i32) -> usize {
    if rhs >= 0 {
        lhs + (rhs as usize)
    } else {
        lhs.wrapping_sub(-rhs as usize)
    }
}

fn main() {
    let stdin = io::stdin();
    let mut console = HandheldGameConsole::parse(&mut stdin.lock().lines().flatten());
    find_repeated_instruction(&mut console);
    println!("Entering infinite loop: accumulator = {}", console.accumulator);
}

#[cfg(test)]
mod day08_spec {
    use super::*;

    mod instruction {
        use super::*;

        #[test]
        fn parse_test() {
            assert_eq!(Instruction::parse("nop +0"), Some(Instruction::Nop));
            assert_eq!(Instruction::parse("acc +1"), Some(Instruction::Acc(1)));
            assert_eq!(Instruction::parse("jmp +4"), Some(Instruction::Jmp(4)));
            assert_eq!(Instruction::parse("nop -1"), Some(Instruction::Nop));
            assert_eq!(Instruction::parse("acc -11"), Some(Instruction::Acc(-11)));
            assert_eq!(Instruction::parse("jmp -4"), Some(Instruction::Jmp(-4)));
        }
    }
}