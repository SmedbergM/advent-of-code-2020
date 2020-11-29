#[macro_use]
extern crate lazy_static;

use std::io;
use std::io::prelude::*;
use std::collections::{BTreeSet, BTreeMap, VecDeque};

use regex::Regex;

use coordinate::XY;

// For this toy day, a puzzle is a rectangular character array such that
// * the perimeter is marked by | (north-south wall), - (east-west wall) and + (corner);
// * the interior consists of one D (door), one o (your position), and zero or more X (wall).
// The challenge is to find the shortest path from o to D through open squares, expressed
// as a string in the alphabet {N,S,E,W}

#[derive(Debug, PartialEq, Eq)]
struct Puzzle {
    width: usize,
    height: usize,
    door: XY,
    player: XY,
    walls: BTreeSet<XY>
}

impl Puzzle {
    fn escape(&self) -> Result<String, IllPosedPuzzle> {
        // Using BFS, find a shortest path from the player to the door, if one exists.
        // If this were a real puzzle, it might be worth it to use a proper linked list
        // with structural sharing, but String is fine for prototyping.
        let mut visited: BTreeMap<XY, String> = BTreeMap::new();
        let mut to_visit: VecDeque<(XY, String)> = VecDeque::new();

        to_visit.push_back((self.player.clone(), "".to_owned()));

        while let Some((xy, path)) = to_visit.pop_front() {
            if xy == self.door {
                return Ok(path)
            } else if !visited.contains_key(&xy) {
                let south = xy.south();
                if !visited.contains_key(&south) && !self.walls.contains(&south) && south.y < self.height {
                    to_visit.push_back((south, path.clone() + "S"));
                }

                let east = xy.east();
                if !visited.contains_key(&east) && !self.walls.contains(&east) && east.x < self.width {
                    to_visit.push_back((east, path.clone() + "E"));
                }

                for north in xy.north() {
                    if !visited.contains_key(&north) && !self.walls.contains(&north) {
                        to_visit.push_back((north, path.clone() + "N"));
                    }
                }

                for west in xy.west() {
                    if !visited.contains_key(&west) && !self.walls.contains(&west) {
                        to_visit.push_back((west, path.clone() + "W"));
                    }
                }

                visited.insert(xy, path);
            }
        }

        // if the queue is exhausted but we haven't found a path to the door:
        Err(IllPosedPuzzle{})
    }
}

#[derive(Debug)]
struct IllPosedPuzzle {}

#[derive(Debug, PartialEq, Eq)]
enum PuzzleBuilder {
    Empty,
    Open{ width: usize, height: usize, door: Option<XY>, player: Option<XY>, walls: BTreeSet<XY> },
    Closed { width: usize, height: usize, door: XY, player: XY, walls: BTreeSet<XY>},
    Error(String)
}

impl PuzzleBuilder {
    fn open(width: usize) -> PuzzleBuilder {
        PuzzleBuilder::Open{ width, height: 0, door: None, player: None, walls: BTreeSet::new() }
    }


    fn err(msg: &str) -> PuzzleBuilder {
        PuzzleBuilder::Error(msg.to_owned())
    }

    fn add(self, line: &str) -> PuzzleBuilder {
        lazy_static! {
            static ref PAT_OUTER: Regex = Regex::new(r"\+(-*)\+").unwrap();
            static ref PAT_INNER: Regex = Regex::new(r"\|([ DoX]*)\|").unwrap();
        }
        match self {
            PuzzleBuilder::Error(msg) => PuzzleBuilder::Error(msg),
            PuzzleBuilder::Empty => {
                PAT_OUTER.captures(line).and_then(|cs| {
                    cs.get(1).map(|m| {
                        let width = m.as_str().len();
                        PuzzleBuilder::open(width)
                    })
                }).unwrap_or({
                    let msg = format!("Incorrect boundary string `{}`", line);
                    PuzzleBuilder::err(&msg)
                })
            },
            PuzzleBuilder::Closed { .. } => PuzzleBuilder::err("Cannot add line to closed puzzle."),
            PuzzleBuilder::Open { width, height, door: Some(door), player: Some(player), walls } if PAT_OUTER.is_match(line) => {
                match PAT_OUTER.captures(line).and_then(|c|{ c.get(1) }) {
                    None => {
                        eprintln!("Pattern reported matched and unmatched on `{}`. This should never happen.", line);
                        PuzzleBuilder::err("Internal server error")
                    },
                    Some(m) if m.as_str().len() != width => {
                        let error_message = format!("Improper line length {} != {}", m.as_str().len(), width);
                        PuzzleBuilder::err(&error_message)
                    },
                    Some(_) => PuzzleBuilder::Closed { width, height, door, player, walls }
                }
            },
            PuzzleBuilder::Open { door: None, .. } if PAT_OUTER.is_match(line) => PuzzleBuilder::err("No door in puzzle."),
            PuzzleBuilder::Open { player: None, .. } if PAT_OUTER.is_match(line) => PuzzleBuilder::err("No player in puzzle."),
            PuzzleBuilder::Open { width, height, door, player, mut walls } => {
                match PAT_INNER.captures(line).and_then(|c|{ c.get(1) }) {
                    None => {
                        let error_message = format!("Improper line `{}`", line);
                        PuzzleBuilder::err(error_message.as_str())
                    },
                    Some(m) => {
                        let row = m.as_str();
                        if row.len() != width {
                            let error_message = format!("Improper line length {} != {}", row.len(), width);
                            PuzzleBuilder::err(error_message.as_str())
                        } else {
                            enum B {
                                Error(String),
                                Open { door: Option<XY>, player: Option<XY> }
                            }
                            let b: B = row.chars().enumerate().fold(
                                B::Open { door, player },
                                |builder, (idx, c)| {
                                    match (builder, c) {
                                        (B::Open { door: Some(_), ..}, 'D') => {
                                            let error_message = format!("Duplicate door detected in row {}.", height);
                                            B::Error(error_message)
                                        },
                                        (B::Open { player, ..}, 'D') => B::Open{ player, door: Some(XY{ x: idx, y: height })},
                                        (B::Open { player: Some(_), ..}, 'o') => {
                                            let error_message = format!("Duplicate player detected in row {}.", height);
                                            B::Error(error_message)
                                        },
                                        (B::Open { door, ..}, 'o') => B::Open {door, player: Some(XY{ x: idx, y: height })},
                                        (b@B::Open { .. }, 'X') => {
                                            walls.insert(XY { x: idx, y: height });
                                            b
                                        }
                                        (other, _) => other
                                    }
                                }
                            );
                            match b {
                                B::Error(msg) => PuzzleBuilder::Error(msg),
                                B::Open { door, player } => PuzzleBuilder::Open { width, height: height + 1, door, player, walls }
                            }
                        }
                    }
                }
            }
        }
    }

    fn build(self) -> Result<Puzzle, PuzzleParseError> {
        match self {
            PuzzleBuilder::Empty => Err(PuzzleParseError::err("Empty builder")),
            PuzzleBuilder::Error(msg) => Err(PuzzleParseError{ msg }),
            PuzzleBuilder::Closed { door, walls, .. } if walls.contains(&door) => {
                Err(PuzzleParseError::err("Door and wall at same location."))
            },
            PuzzleBuilder::Closed { player, walls, .. } if walls.contains(&player) => {
                Err(PuzzleParseError::err("Player and wall at same location."))
            },
            PuzzleBuilder::Closed { width, height, door, player, walls } => {
                Ok(Puzzle { width, height, door, player, walls })
            },
            PuzzleBuilder::Open { .. } => Err(PuzzleParseError::err("Incomplete builder")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PuzzleParseError {
    msg: String
}

impl PuzzleParseError {
    fn err(msg: &str) -> PuzzleParseError {
        PuzzleParseError { msg: msg.to_owned() }
    }
}

fn main() {
    let stdin = io::stdin();
    let puzzle: Puzzle = stdin.lock().lines().flatten().fold(PuzzleBuilder::Empty, |builder, line| {
        builder.add(line.as_str())
    }).build().unwrap();

    let escape_path = puzzle.escape().unwrap();
    println!("Escape path of length {} found: \n{}", escape_path.len(), escape_path);
}

#[cfg(test)]
mod puzzle_builder_spec {

    mod add {
        use super::super::*;
        
        #[test]
        fn empty_should_read_boundary_line() {
            let empty = PuzzleBuilder::Empty;
            let line = "++";
            match empty.add(line) {
                PuzzleBuilder::Open { width: 0, height: 0, door: None, player: None, walls } => {
                    assert!(walls.is_empty())
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let empty = PuzzleBuilder::Empty;
            let line = "+---+";
            match empty.add(line) {
                PuzzleBuilder::Open { width: 3, height: 0, door: None, player: None, walls } => {
                    assert!(walls.is_empty())
                },
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn empty_should_fail_on_interior_line() {
            match PuzzleBuilder::Empty.add("||") {
                PuzzleBuilder::Error(msg) => {
                    assert_eq!(msg, "Incorrect boundary string `||`")
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            match PuzzleBuilder::Empty.add("| X o X D|") {
                PuzzleBuilder::Error(msg) => {
                    assert_eq!(msg, "Incorrect boundary string `| X o X D|`")
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }

        #[test]
        fn empty_should_fail_on_bad_line() {
            match PuzzleBuilder::Empty.add("+-") {
                PuzzleBuilder::Error(msg) => {
                    assert_eq!(msg, "Incorrect boundary string `+-`")
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }

        #[test]
        fn error_should_always_fail() {
            let builder = PuzzleBuilder::err("I'm in an error state");
            match builder.add("") {
                PuzzleBuilder::Error(msg) => {
                    assert_eq!(msg, "I'm in an error state")
                },
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn open_should_add_empty_row() {
            let builder = PuzzleBuilder::Open { width: 10, height: 0, door: None, player: None, walls: BTreeSet::new() };
            match builder.add("|          |") {
                PuzzleBuilder::Open { width: 10, height: 1, door: None, player: None, walls } => {
                    assert!(walls.is_empty());
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }

        #[test]
        fn open_should_add_walls() {
            let builder = PuzzleBuilder::Open { width: 7, height: 0, door: None, player: None, walls: BTreeSet::new() };
            match builder.add("|  X X  |") {
                PuzzleBuilder::Open { width: 7, height: 1, door: None, player: None, walls } => {
                    assert!(walls.contains(&XY::new(2, 0)));
                    assert!(walls.contains(&XY::new(4,0)));
                    assert_eq!(walls.len(), 2);
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::Open { width: 7, height: 2, door: None, player: None, walls: BTreeSet::new() };
            match builder.add("|  X X  |") {
                PuzzleBuilder::Open { width: 7, height: 3, door: None, player: None, walls } => {
                    assert!(walls.contains(&XY::new(2, 2)));
                    assert!(walls.contains(&XY::new(4,2)));
                    assert_eq!(walls.len(), 2);
                },
                other => assert!(false, "Unexpected result {:?}", other)
            }

            let walls0: BTreeSet<XY> = {
                let mut w = BTreeSet::new();
                w.insert(XY::new(4,0));
                w.insert(XY::new(3,1));
                w.insert(XY::new(6,1));
                w
            };
            let builder = PuzzleBuilder::Open { width: 7, height: 2, door: None, player: None, walls: walls0.clone() };
            match builder.add("|  X X  |") {
                PuzzleBuilder::Open { width: 7, height: 3, door: None, player: None, walls } => {
                    assert!(walls.contains(&XY::new(2, 2)));
                    assert!(walls.contains(&XY::new(4,2)));
                    assert_eq!(walls.len(), 5);
                },
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn open_should_add_door() {
            let builder = PuzzleBuilder::open(6);
            match builder.add("|    D |") {
                PuzzleBuilder::Open { width: 6, height: 1, door: Some(XY { x: 4, y: 0 }), player: None, walls} => {
                    assert!(walls.is_empty());
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: None, player: None, walls: BTreeSet::new() };
            match builder.add("|    D |") {
                PuzzleBuilder::Open { width: 6, height: 3, door: Some(XY { x: 4, y: 2 }), player: None, walls} => {
                    assert!(walls.is_empty());
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let walls0: BTreeSet<XY> = {
                let mut w = BTreeSet::new();
                w.insert(XY::new(4,0));
                w.insert(XY::new(3,1));
                w.insert(XY::new(6,1));
                w
            };
            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: None, player: None, walls: walls0.clone() };
            match builder.add("|    D |") {
                PuzzleBuilder::Open { width: 6, height: 3, door: Some(XY { x: 4, y: 2 }), player: None, walls} => {
                    assert_eq!(walls, walls0);
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }

        #[test]
        fn open_should_reject_duplicate_door() {
            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: Some(XY::new(4,1)), player: None, walls: BTreeSet::new() };
            match builder.add("| D    |") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Duplicate door detected in row 2."),
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn open_should_add_player() {
            let builder = PuzzleBuilder::open(6);
            match builder.add("|    o |") {
                PuzzleBuilder::Open { width: 6, height: 1, door: None, player: Some(XY { x: 4, y: 0 }), walls} => {
                    assert!(walls.is_empty());
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: None, player: None, walls: BTreeSet::new() };
            match builder.add("|    o |") {
                PuzzleBuilder::Open { width: 6, height: 3, door: None, player: Some(XY { x: 4, y: 2 }), walls} => {
                    assert!(walls.is_empty());
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let walls0: BTreeSet<XY> = {
                let mut w = BTreeSet::new();
                w.insert(XY::new(4,0));
                w.insert(XY::new(3,1));
                w.insert(XY::new(6,1));
                w
            };
            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: None, player: None, walls: walls0.clone() };
            match builder.add("|    o |") {
                PuzzleBuilder::Open { width: 6, height: 3, door: None, player: Some(XY { x: 4, y: 2 }), walls} => {
                    assert_eq!(walls, walls0);
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }

        #[test]
        fn open_should_reject_duplicate_player() {
            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: None, player: Some(XY::new(4,1)), walls: BTreeSet::new() };
            match builder.add("| o    |") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Duplicate player detected in row 2."),
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn open_should_add_complex_line() {
            let walls0: BTreeSet<XY> = {
                let mut w = BTreeSet::new();
                w.insert(XY::new(4,0));
                w.insert(XY::new(3,1));
                w.insert(XY::new(6,1));
                w
            };
            let builder = PuzzleBuilder::Open { width: 8, height: 2, door: None, player: None, walls: walls0.clone() };
            match builder.add("|oX XD XX|") {
                PuzzleBuilder::Open { width: 8, height: 3, door: Some(XY { x: 4, y: 2}), player: Some(XY { x: 0, y: 2}), walls } => {
                    assert_eq!(walls.len(), 7);
                    assert!(walls.contains(&XY::new(1,2)));
                    assert!(walls.contains(&XY::new(3,2)));
                    assert!(walls.contains(&XY::new(6,2)));
                    assert!(walls.contains(&XY::new(7,2)));
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::Open { width: 8, height: 2, door: Some(XY::new(5,1)), player: None, walls: walls0.clone() };
            match builder.add("|oX X  XX|") {
                PuzzleBuilder::Open { width: 8, height: 3, door: Some(XY { x: 5, y: 1}), player: Some(XY { x: 0, y: 2}), walls } => {
                    assert_eq!(walls.len(), 7);
                    assert!(walls.contains(&XY::new(1,2)));
                    assert!(walls.contains(&XY::new(3,2)));
                    assert!(walls.contains(&XY::new(6,2)));
                    assert!(walls.contains(&XY::new(7,2)));
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::Open { width: 8, height: 2, door: None, player: Some(XY::new(5,1)), walls: walls0.clone() };
            match builder.add("| X XD XX|") {
                PuzzleBuilder::Open { width: 8, height: 3, door: Some(XY { x: 4, y: 2}), player: Some(XY { x: 5, y: 1}), walls } => {
                    assert_eq!(walls.len(), 7);
                    assert!(walls.contains(&XY::new(1,2)));
                    assert!(walls.contains(&XY::new(3,2)));
                    assert!(walls.contains(&XY::new(6,2)));
                    assert!(walls.contains(&XY::new(7,2)));
                },
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }

        #[test]
        fn open_should_fail_on_bad_line() {
            let builder = PuzzleBuilder::open(6);
            match builder.add("|XXX XX") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Improper line `|XXX XX`"),
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::open(6);
            match builder.add("|X X|") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Improper line length 3 != 6"),
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn open_should_close_on_boundary_line() {
            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: Some(XY::new(4,1)), player: Some(XY::new(3,0)), walls: BTreeSet::new() };
            match builder.add("+------+") {
                PuzzleBuilder::Closed { width: 6, height: 2, door: XY { x: 4, y: 1}, player: XY{ x: 3, y: 0}, walls } => {
                    assert!(walls.is_empty())
                },
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn open_should_error_on_improper_line() {
            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: Some(XY::new(4,1)), player: Some(XY::new(3,0)), walls: BTreeSet::new() };
            match builder.add("+---+") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Improper line length 3 != 6"),
                other => assert!(false, "Unexpected result {:?}", other)
            }

            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: Some(XY::new(4,1)), player: Some(XY::new(3,0)), walls: BTreeSet::new() };
            match builder.add("+--X--X+") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Improper line `+--X--X+`"),
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn open_should_error_on_close_if_door_or_player_missing() {
            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: None, player: Some(XY::new(3,0)), walls: BTreeSet::new() };
            match builder.add("+------+") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "No door in puzzle."),
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::Open { width: 6, height: 2, door: Some(XY::new(3,0)), player: None, walls: BTreeSet::new() };
            match builder.add("+------+") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "No player in puzzle."),
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }

        #[test]
        fn closed_should_error_on_any_line() {
            let builder = PuzzleBuilder::Closed { width: 6, height: 2, door: XY::new(3,1), player: XY::new(5,0), walls: BTreeSet::new() };
            match builder.add("+------+") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Cannot add line to closed puzzle."),
                other => assert!(false, "Unexpected result {:?}", other)
            };

            let builder = PuzzleBuilder::Closed { width: 6, height: 2, door: XY::new(3,1), player: XY::new(5,0), walls: BTreeSet::new() };
            match builder.add("| X X  |") {
                PuzzleBuilder::Error(msg) => assert_eq!(msg, "Cannot add line to closed puzzle."),
                other => assert!(false, "Unexpected result {:?}", other)
            };
        }
    }

    mod build {
        use super::super::*;
        #[test]
        fn empty_should_error() {
            let empty = PuzzleBuilder::Empty;
            let expected_error_message: String = "Empty builder".to_owned();
            assert_eq!(empty.build(), Err(PuzzleParseError{ msg: expected_error_message }));
        }

        #[test]
        fn err_should_error() {
            let err = PuzzleBuilder::err("Failure");
            assert_eq!(err.build(), Err(PuzzleParseError{ msg: "Failure".to_owned()}));
        }

        #[test]
        fn open_should_error() {
            let builder = PuzzleBuilder::Open {
                width: 10,
                height: 11,
                door: Some(XY::new(3,5)),
                player: Some(XY::new(2,7)),
                walls: BTreeSet::new()
            };
            match builder.build() {
                Err(PuzzleParseError{msg}) => {
                    assert_eq!(msg, "Incomplete builder")
                },
                other => assert!(false, "Unexpected result {:?}", other)
            }
        }

        #[test]
        fn closed_with_door_and_player_should_succeed() {
            let walls: BTreeSet<XY> = {
                let mut ws: BTreeSet<XY> = BTreeSet::new();
                ws.insert(XY::new(3,4));
                ws.insert(XY::new(4,6));
                ws
            };
            let builder = PuzzleBuilder::Closed {
                width: 10,
                height: 11,
                door: XY::new(3,5),
                player: XY::new(2,7),
                walls: walls.clone()
            };
            let puzzle = builder.build().unwrap();
            assert_eq!(puzzle.width, 10);
            assert_eq!(puzzle.height, 11);
            assert_eq!(puzzle.walls, walls);
        }

        #[test]
        fn overlapping_door_and_wall_should_error() {
            // Ordinarily, if a builder is used only via public methods, this is impossible.
            let walls0: BTreeSet<XY> = {
                let mut ws = BTreeSet::new();
                ws.insert(XY::new(1,3));
                ws
            };
            let builder = PuzzleBuilder::Closed {
                width: 6,
                height: 5,
                door: XY::new(1,3),
                player: XY::new(2,4),
                walls: walls0.clone()
            };
            assert_eq!(builder.build(), Err(PuzzleParseError::err("Door and wall at same location.")));
        }

        #[test]
        fn overlapping_player_and_wall_should_error() {
            let walls0: BTreeSet<XY> = {
                let mut ws = BTreeSet::new();
                ws.insert(XY::new(1,3));
                ws
            };
            let builder = PuzzleBuilder::Closed {
                width: 6,
                height: 5,
                door: XY::new(2,4),
                player: XY::new(1,3),
                walls: walls0.clone()
            };
            assert_eq!(builder.build(), Err(PuzzleParseError::err("Player and wall at same location.")))
        }
    }
}
