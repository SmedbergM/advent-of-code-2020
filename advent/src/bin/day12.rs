use std::io::prelude::*;

#[macro_use]
extern crate lazy_static;

use regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cardinal {
    North,
    South,
    East,
    West
}

impl Cardinal {
    fn left(&self, quarter_turns: u8) -> Cardinal {
        let mut r = *self;

        for _ in 0..quarter_turns {
            match r {
                Cardinal::North => r = Cardinal::West,
                Cardinal::West => r = Cardinal::South,
                Cardinal::South => r = Cardinal::East,
                Cardinal::East => r = Cardinal::North
            }
        }

        r
    }

    fn right(&self, quarter_turns: u8) -> Cardinal {
        self.left(4 - (quarter_turns % 4))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    North(usize),
    South(usize),
    East(usize),
    West(usize),
    Left(u8), // in range 0..4
    Right(u8), // in range 0..4
    Forward(usize)
}

impl Instruction {
    fn parse(line: &str) -> Option<Instruction> {
        lazy_static! {
            static ref INSTRUCTION_PAT: Regex = Regex::new(r"([NSEWLRF])(\d+)").unwrap();
        }

        INSTRUCTION_PAT.captures(line).and_then(|caps| {
            usize::from_str_radix(&caps[2], 10).ok().and_then(|x| {
                match &caps[1] {
                    "N" => Some(Instruction::North(x)),
                    "S" => Some(Instruction::South(x)),
                    "E" => Some(Instruction::East(x)),
                    "W" => Some(Instruction::West(x)),
                    "L" => Some(Instruction::Left(((x % 360 ) / 90) as u8)),
                    "R" => Some(Instruction::Right(((x % 360 ) / 90) as u8)),
                    "F" => Some(Instruction::Forward(x)),
                    _ => None
                }
            })
        }).or_else(|| {
            eprintln!("Unable to parse instruction from {}", line);
            None
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ShipsPosition {
    x: isize,
    y: isize,
    heading: Cardinal
}

impl ShipsPosition {
    fn new() -> ShipsPosition {
        ShipsPosition { x: 0, y: 0, heading: Cardinal::East }
    }

    fn apply(&mut self, instr: &Instruction) {
        match *instr {
            Instruction::North(dy) => self.y += dy as isize,
            Instruction::South(dy) => self.y -= dy as isize,
            Instruction::East(dx) => self.x += dx as isize,
            Instruction::West(dx) => self.x -= dx as isize,
            Instruction::Left(qt) => self.heading = self.heading.left(qt),
            Instruction::Right(qt) => self.heading = self.heading.right(qt),
            Instruction::Forward(s) => match self.heading {
                Cardinal::North => self.y += s as isize,
                Cardinal::South => self.y -= s as isize,
                Cardinal::East => self.x += s as isize,
                Cardinal::West => self.x -= s as isize
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct WaypointPosition {
    waypoint_dx: isize,
    waypoint_dy: isize,
    ship_x: isize,
    ship_y: isize
}

impl WaypointPosition {
    fn new() -> WaypointPosition {
        WaypointPosition {
            ship_x: 0, ship_y: 0,
            waypoint_dx: 10, waypoint_dy: 1
        }
    }

    fn apply(&mut self, instr: &Instruction) {
        match *instr {
            Instruction::North(dy) => self.waypoint_dy += dy as isize,
            Instruction::South(dy) => self.waypoint_dy -= dy as isize,
            Instruction::East(dx) => self.waypoint_dx += dx as isize,
            Instruction::West(dx) => self.waypoint_dx -= dx as isize,
            Instruction::Left(qt) => {
                let prev = (self.waypoint_dx, self.waypoint_dy);
                match qt % 4 {
                    1 => {
                        self.waypoint_dx = -prev.1;
                        self.waypoint_dy = prev.0;
                    },
                    2 => {
                        self.waypoint_dx = -prev.0;
                        self.waypoint_dy = -prev.1;
                    },
                    3 => {
                        self.waypoint_dx = prev.1;
                        self.waypoint_dy = -prev.0;
                    },
                    _ => () // no rotation
                }
            },
            Instruction::Right(qt) => {
                self.apply(&Instruction::Left(4 - (qt % 4)));
            },
            Instruction::Forward(s) => {
                self.ship_x += self.waypoint_dx * s as isize;
                self.ship_y += self.waypoint_dy * s as isize;
            }
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut ships_position = ShipsPosition::new();
    let mut waypoint_position = WaypointPosition::new();
    for instruction in stdin.lock().lines().flatten().flat_map(|line| Instruction::parse(&line)) {
        ships_position.apply(&instruction);
        waypoint_position.apply(&instruction);
    }
    println!("Ship's position: x={}, y={}. Manhattan displacement: {}",
        ships_position.x, ships_position.y, ships_position.x.abs() + ships_position.y.abs()
    );
    println!("Waypoint method: x={}, y={}. Manhattan displacement: {}",
        waypoint_position.ship_x, waypoint_position.ship_y, waypoint_position.ship_x.abs() + waypoint_position.ship_y.abs()
    );
}

#[cfg(test)]
mod day12_spec {
    use super::*;

    mod cardinal {
        use super::*;

        #[test]
        fn left_turn_test() {
            assert_eq!(Cardinal::East.left(0), Cardinal::East);
            assert_eq!(Cardinal::East.left(1), Cardinal::North);
            assert_eq!(Cardinal::North.left(2), Cardinal::South);
            assert_eq!(Cardinal::South.left(3), Cardinal::West);
        }

        #[test]
        fn right_turn_test() {
            assert_eq!(Cardinal::North.right(0), Cardinal::North);
            assert_eq!(Cardinal::North.right(1), Cardinal::East);
            assert_eq!(Cardinal::East.right(2), Cardinal::West);
            assert_eq!(Cardinal::West.right(3), Cardinal::South);
        }
    }

    mod instruction {
        use super::*;

        #[test]
        fn parse_test() {
            assert_eq!(Instruction::parse("F10"), Some(Instruction::Forward(10)));
            assert_eq!(Instruction::parse("N3"), Some(Instruction::North(3)));
            assert_eq!(Instruction::parse("R90"), Some(Instruction::Right(1)));
            assert_eq!(Instruction::parse("L270"), Some(Instruction::Left(3)));
            assert_eq!(Instruction::parse("S11"), Some(Instruction::South(11)));
            assert_eq!(Instruction::parse("E13"), Some(Instruction::East(13)));
            assert_eq!(Instruction::parse("W87"), Some(Instruction::West(87)));
            
            // even absurd instructions can be parsed
            assert_eq!(Instruction::parse("L181"), Some(Instruction::Left(2)));
    
            // but negatives cannot
            assert_eq!(Instruction::parse("N-3"), None);
        }
    }

    mod ships_position {
        use super::*;

        #[test]
        fn apply_test() {
            let mut sp = ShipsPosition::new();
            sp.apply(&Instruction::Forward(10));
            assert_eq!(sp, ShipsPosition { heading: Cardinal::East, x: 10, y: 0 });

            sp.apply(&Instruction::North(3));
            assert_eq!(sp, ShipsPosition { heading: Cardinal::East, x: 10, y: 3 });

            sp.apply(&Instruction::Forward(7));
            assert_eq!(sp, ShipsPosition { heading: Cardinal::East, x: 17, y: 3 });

            sp.apply(&Instruction::Right(1));
            assert_eq!(sp, ShipsPosition { heading: Cardinal::South, x: 17, y: 3 });

            sp.apply(&Instruction::Forward(11));
            assert_eq!(sp, ShipsPosition { heading: Cardinal::South, x: 17, y: -8 });
        }
    }

    mod waypoint_position {
        use super::*;

        #[test]
        fn apply_test() {
            let mut wp = WaypointPosition::new();

            wp.apply(&Instruction::Forward(10));
            assert_eq!(wp, WaypointPosition{
                ship_x: 100, ship_y: 10,
                waypoint_dx: 10, waypoint_dy: 1
            });

            wp.apply(&Instruction::North(3));
            assert_eq!(wp, WaypointPosition{
                ship_x: 100, ship_y: 10,
                waypoint_dx: 10, waypoint_dy: 4
            });

            wp.apply(&Instruction::Forward(7));
            assert_eq!(wp, WaypointPosition {
                ship_x: 170, ship_y: 38,
                waypoint_dx: 10, waypoint_dy: 4
            });

            wp.apply(&Instruction::Right(1));
            assert_eq!(wp, WaypointPosition {
                ship_x: 170, ship_y: 38,
                waypoint_dx: 4, waypoint_dy: -10
            });

            wp.apply(&Instruction::Forward(11));
            assert_eq!(wp, WaypointPosition {
                ship_x: 214, ship_y: -72,
                waypoint_dx: 4, waypoint_dy: -10
            });
        }
    }
}