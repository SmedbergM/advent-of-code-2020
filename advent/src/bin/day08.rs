use std::io;
use std::io::prelude::*;

#[macro_use]
extern crate lazy_static;
use regex::Regex;

use advent::bitset::BitSet;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Instruction {
    Nop(i32),
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
                    "nop" => Some(Instruction::Nop(n)),
                    "acc" => Some(Instruction::Acc(n)),
                    "jmp" => Some(Instruction::Jmp(n)),
                    _ => None
                }
            })
        })
    }
}

#[derive(Clone)]
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
                Instruction::Nop(_) => self.instruction_ptr += 1,
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

    fn reset(&mut self) {
        self.accumulator = 0;
        self.instruction_ptr = 0;
    }
}

fn find_infinite_loop(console: &mut HandheldGameConsole) {
    let mut executed_instructions = BitSet::new(console.instructions.len());
    while let Some(false) = executed_instructions.get(console.instruction_ptr) {
        executed_instructions.set(console.instruction_ptr);
        console.step();
    }
}

// Returns the index of an instruction which must be changed from NOP to JMP or vice versa
fn fix_infinite_loop(console: &mut HandheldGameConsole) -> Option<(usize, i32)> {
    let mut executed_instructions = BitSet::new(console.instructions.len());

    enum ExitStatus {
        InfiniteLoop,
        Zero(i32),
        Nonzero(usize)
    }

    fn attempt_fix(mut console: HandheldGameConsole, mut executed_instructions: BitSet) -> ExitStatus {
        // first, switch the current instruction
        match console.instructions.get_mut(console.instruction_ptr).map(|j| {
            match j {
                Instruction::Nop(x) => *j = Instruction::Jmp(*x),
                Instruction::Jmp(x) => *j = Instruction::Nop(*x),
                _ => ()
            }
        }) {
            None => return ExitStatus::Nonzero(console.instruction_ptr),
            _ => ()
        };

        // then run the modified instruction set
        loop {
            if console.instruction_ptr == console.instructions.len() {
                return ExitStatus::Zero(console.accumulator)
            } else if console.instruction_ptr > console.instructions.len() {
                return ExitStatus::Nonzero(console.instruction_ptr)
            } else if let Some(true) = executed_instructions.get(console.instruction_ptr) {
                return ExitStatus::InfiniteLoop
            } else {
                executed_instructions.set(console.instruction_ptr);
                console.step();
            }
        }
    }

    loop {
        match console.instructions.get(console.instruction_ptr) {
            None => return None,
            Some(Instruction::Acc(_)) => {
                if let Some(true) = executed_instructions.set(console.instruction_ptr) {
                    return None
                }
                console.step()
            },
            _ => {
                match attempt_fix(console.clone(), executed_instructions.clone()) {
                    ExitStatus::InfiniteLoop => {
                        if let Some(true) = executed_instructions.set(console.instruction_ptr) {
                            return None
                        }
                        console.step()
                    },
                    ExitStatus::Nonzero(_) => {
                        if let Some(true) = executed_instructions.set(console.instruction_ptr) {
                            return None
                        }
                        console.step()
                    },
                    ExitStatus::Zero(acc) => {
                        return Some((console.instruction_ptr, acc))
                    }
                }
            }
        }
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
    find_infinite_loop(&mut console);
    println!("Entering infinite loop: accumulator = {}", console.accumulator);

    console.reset();
    match fix_infinite_loop(&mut console) {
        None => println!("No fix found."),
        Some((ptr, acc)) => println!(
            "Fix found: corrupted instruction at {}.\n\
             Output of fixed program: {}", ptr, acc
        )
    }
}

#[cfg(test)]
mod day08_spec {
    use super::*;

    mod instruction {
        use super::*;

        #[test]
        fn parse_test() {
            assert_eq!(Instruction::parse("nop +0"), Some(Instruction::Nop(0)));
            assert_eq!(Instruction::parse("acc +1"), Some(Instruction::Acc(1)));
            assert_eq!(Instruction::parse("jmp +4"), Some(Instruction::Jmp(4)));
            assert_eq!(Instruction::parse("nop -1"), Some(Instruction::Nop(-1)));
            assert_eq!(Instruction::parse("acc -11"), Some(Instruction::Acc(-11)));
            assert_eq!(Instruction::parse("jmp -4"), Some(Instruction::Jmp(-4)));
        }
    }

    mod handheld_game_console {
        use super::*;

        #[test]
        fn parse_test() {
            let input = "nop +0\n\
            acc +1\n\
            jmp +4\n\
            acc +3\n\
            jmp -3\n\
            acc -99\n\
            acc +1\n\
            jmp -4\n\
            acc +6\n";
            
            let console = HandheldGameConsole::parse(&mut input.lines().map(|s| s.to_owned()));
            assert_eq!(console.accumulator, 0);
            assert_eq!(console.instruction_ptr, 0);
            assert_eq!(console.instructions, vec!(
                Instruction::Nop(0),
                Instruction::Acc(1),
                Instruction::Jmp(4),
                Instruction::Acc(3),
                Instruction::Jmp(-3),
                Instruction::Acc(-99),
                Instruction::Acc(1),
                Instruction::Jmp(-4),
                Instruction::Acc(6)
            ));
        }
    }

    #[test]
    fn find_infinite_loop_test() {
        let mut console = HandheldGameConsole {
            accumulator: 0, instruction_ptr: 0,
            instructions: vec!(
                Instruction::Nop(0),
                Instruction::Acc(1),
                Instruction::Jmp(4),
                Instruction::Acc(3),
                Instruction::Jmp(-3),
                Instruction::Acc(-99),
                Instruction::Acc(1),
                Instruction::Jmp(-4),
                Instruction::Acc(6)
            )
        };

        find_infinite_loop(&mut console);
        assert_eq!(console.accumulator, 5)
    }

    #[test]
    fn fix_infinite_loop_test() {
        let mut console = HandheldGameConsole {
            accumulator: 0, instruction_ptr: 0,
            instructions: vec!(
                Instruction::Nop(0),
                Instruction::Acc(1),
                Instruction::Jmp(4),
                Instruction::Acc(3),
                Instruction::Jmp(-3),
                Instruction::Acc(-99),
                Instruction::Acc(1),
                Instruction::Jmp(-4),
                Instruction::Acc(6)
            )
        };

        assert_eq!(fix_infinite_loop(&mut console), Some((7,8)));
    }
}
