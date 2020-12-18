use std::io::prelude::*;

#[macro_use]
extern crate lazy_static;
use regex::Regex;

fn left_to_right(line: &str) -> Option<u64> {
    #[derive(Clone, Copy, Debug)]
    enum Acc {
        Empty,
        Infix(u64),
        Add(u64),
        Mul(u64)
    }

    lazy_static! {
        static ref OPEN_PAREN_PAT: Regex = Regex::new(r"^\((.*)").unwrap();
        static ref NUMBER_PAT: Regex = Regex::new(r"^(\d+)\s*(.*)").unwrap();
        static ref OP_PAT: Regex = Regex::new(r"^([+*])\s*(.*)").unwrap();
        static ref CLOSE_PAREN_PAT: Regex = Regex::new(r"^\)\s*(.*)").unwrap();
    }

    let mut stack: Vec<Acc> = vec!();
    let mut current: Acc = Acc::Empty;
    let mut rest: String = line.to_owned();

    loop {
        if let Some(caps) = OPEN_PAREN_PAT.captures(&rest) {
            stack.push(current);
            current = Acc::Empty;
            rest = caps[1].to_owned();
        } else if let Some(caps) = NUMBER_PAT.captures(&rest) {
            let x = u64::from_str_radix(&caps[1], 10).unwrap();
            rest = caps[2].to_owned();
            match current {
                Acc::Empty => current = Acc::Infix(x),
                Acc::Infix(_) => {
                    eprintln!("Grammar error: stack depth {}, current token {:?}, rest {}",
                        stack.len(), current, rest);
                    return None
                },
                Acc::Add(x0) => current = Acc::Infix(x + x0),
                Acc::Mul(x0) => current = Acc::Infix(x * x0)
            }
        } else if let Some(caps) = OP_PAT.captures(&rest) {
            match (current, &caps[1]) {
                (Acc::Infix(x0), "+") => current = Acc::Add(x0),
                (Acc::Infix(x0), "*") => current = Acc::Mul(x0),
                _ => {
                    eprintln!("Unexpected result. Stack depth {}, current token {:?}, rest {}",
                        stack.len(), current, rest    
                    );
                    return None
                }
            }
            rest = caps[2].to_owned();
        } else if let Some(caps) = CLOSE_PAREN_PAT.captures(&rest) {
            match (stack.pop(), current) {
                (None, _) => {
                    eprintln!("Unmatched closing paren! Current token {:?}, rest {}", current, rest);
                    return None
                },
                (Some(Acc::Empty), Acc::Infix(c)) => current = Acc::Infix(c),
                (Some(Acc::Add(x0)), Acc::Infix(c)) => current = Acc::Infix(x0 + c),
                (Some(Acc::Mul(x0)), Acc::Infix(c)) => current = Acc::Infix(x0 * c),
                _ => {
                    eprintln!("Missing operand! Stack depth {}, current acc {:?}, rest {}",
                        stack.len() + 1, current, rest);
                    return None
                },
            }
            rest = caps[1].to_owned();
        } else if rest.is_empty() {
            if stack.is_empty() {
                match current {
                    Acc::Infix(x) => return Some(x),
                    _ => {
                        eprintln!("Unterminated expression! Current token {:?}", current);
                        return None
                    }
                }
            } else {
                eprintln!("Unmatched opening parenthesis! Stack depth {}, current token {:?}",
                stack.len(), current);
                return None
            }
        } else {
            eprintln!("Unmatched text {}. (Stack depth {}, current token {:?}", rest, stack.len(), current);
            return None
        }
    }
}

fn add_before_mult(line: &str) -> Option<u64> {
    #[derive(Debug, Clone, Copy)]
    enum Current {
        Empty,
        Value(u64)
    }

    enum StackFrame {
        Mult(u64),
        OpenP,
        PAdd(u64), // PAdd(x) means that x + ( is the beginning of the current sub-expression
        PMult(u64)
    }

    lazy_static! {
        static ref OPEN_PAREN_PAT: Regex = Regex::new(r"^\(\s*(.*)").unwrap();
        static ref NUMBER_PAT: Regex = Regex::new(r"^(\d+)\s*(.*)").unwrap();
        static ref TIMES_NUMBER_PAT: Regex = Regex::new(r"^\s*\*\s*(\d+)\s*(.*)").unwrap();
        static ref TIMES_PAREN_PAT: Regex = Regex::new(r"^\s*\*\s*\((.*)").unwrap();
        static ref PLUS_NUMBER_PAT: Regex = Regex::new(r"^\s*\+\s*(\d+)\s*(.*)").unwrap();
        static ref PLUS_PAREN_PAT: Regex = Regex::new(r"^\s*\+\s*\((.*)").unwrap();
        static ref CLOSE_PAREN_PAT: Regex = Regex::new(r"^\)\s*(.*)").unwrap();
    }

    let mut stack: Vec<StackFrame> = vec!();
    let mut current = Current::Empty;
    let mut rest: String = line.to_owned();

    loop {
        if let Some(caps) = OPEN_PAREN_PAT.captures(&rest) {
            match current {
                Current::Empty => {
                    stack.push(StackFrame::OpenP);
                },
                _ => {
                    eprintln!("Cannot start sub-expression here. Stack depth {}\ncurrent: {:?}\nrest: {}",
                        stack.len(), current, rest);
                    return None
                }
            }
            rest = caps[1].to_owned();
        } else if let Some(caps) = NUMBER_PAT.captures(&rest) {
            let x = u64::from_str_radix(&caps[1], 10).unwrap();
            match current {
                Current::Empty => current = Current::Value(x),
                _ => {
                    eprintln!("Token {} not expected here.\nStack depth {}\ncurrent: {:?}\nrest: {}",
                        x, stack.len(), current, rest);
                    return None
                }
            }
            rest = caps[2].to_owned();
        } else if let Some(caps) = PLUS_NUMBER_PAT.captures(&rest) {
            let x = u64::from_str_radix(&caps[1], 10).unwrap();
            match current {
                Current::Value(x0) => current = Current::Value(x0 + x),
                _ => {
                    eprintln!("Unexpected token '+'.\nStack depth {}\ncurrent: {:?}\nrest: {}",
                        stack.len(), current, rest);
                    return None
                }
            }
            rest = caps[2].to_owned();
        } else if let Some(caps) = PLUS_PAREN_PAT.captures(&rest) {
            match current {
                Current::Value(x0) => {
                    stack.push(StackFrame::PAdd(x0));
                    current = Current::Empty;
                },
                _ => {
                    eprintln!("Unexpected token '+'.\nStack depth {}\ncurrent: {:?}\nrest: {}",
                        stack.len(), current, rest);
                    return None
                }
            }
            rest = caps[1].to_owned();
        } else if let Some(caps) = TIMES_NUMBER_PAT.captures(&rest) {
            let x = u64::from_str_radix(&caps[1], 10).unwrap();
            match current {
                Current::Value(x0) => {
                    stack.push(StackFrame::Mult(x0));
                    current = Current::Value(x);
                },
                _ => {
                    eprintln!("Unexpected token '*'.\nStack depth {}\ncurrent: {:?}\nrest: {}",
                        stack.len(), current, rest);
                    return None
                }
            }

            rest = caps[2].to_owned();
        } else if let Some(caps) = TIMES_PAREN_PAT.captures(&rest) {
            match current {
                Current::Value(x0) => {
                    stack.push(StackFrame::PMult(x0));
                    current = Current::Empty;
                },
                _ => {
                    eprintln!("Unexpected token '*'.\nStack depth {}\ncurrent: {:?}\nrest: {}",
                        stack.len(), current, rest);
                    return None
                }
            }

            rest = caps[1].to_owned();
        } else if let Some(caps) = CLOSE_PAREN_PAT.captures(&rest) {
            match current {
                Current::Empty => {
                    eprintln!("Empty subexpression encountered!\nStack depth {}\nrest: {}", stack.len(), rest);
                    return None
                },
                Current::Value(x) => {
                    let mut s = x;
                    let mut open_paren_found = false;
                    while let Some(frame) = stack.pop() {
                        match frame {
                            StackFrame::Mult(x0) => s *= x0,
                            StackFrame::OpenP => {
                                open_paren_found = true; break
                            },
                            StackFrame::PAdd(x0) => {
                                open_paren_found = true;
                                s += x0; break
                            },
                            StackFrame::PMult(x0) => {
                                // demote PMult to Mult but don't multiply yet
                                open_paren_found = true;
                                stack.push(StackFrame::Mult(x0)); break
                            }
                        }
                    }
                    current = Current::Value(s);
                    if !open_paren_found {
                        eprintln!("Unmatched closing parenthesis found!\nrest: {}", rest);
                        return None
                    }
                }
            }

            rest = caps[1].to_owned();
        } else if rest.is_empty() {
            match current {
                Current::Empty => {
                    eprintln!("Empty expression or sub-expression cannot be evaluated");
                    return None
                },
                Current::Value(x) => {
                    let mut s = x;
                    while let Some(acc) = stack.pop() {
                        match acc {
                            StackFrame::Mult(x0) => s *= x0,
                            _ => {
                                eprintln!("Unmatched opening parenthesis encountered!");
                                return None
                            }
                        }
                    }
                    return Some(s)
                },
            }
        } else {
            eprintln!("Unmatched text {}.\nStack depth: {}\ncurrent: {:?}", rest, stack.len(), current);
            return None
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut ltr = 0;
    let mut abm = 0;
    for line in stdin.lock().lines().flatten() {
        ltr += left_to_right(&line).unwrap();
        abm += add_before_mult(&line).unwrap();
    }

    println!("Left-to-right sum of provided expressions: {}", ltr);
    println!("Add-before-multiply sum of expressions: {}", abm);
}

#[cfg(test)]
mod day18_spec {
    use super::*;

    #[test]
    fn left_to_right_test() {
        let expr = "1 + 3";
        assert_eq!(left_to_right(expr), Some(4));

        let expr = "3 * 5";
        assert_eq!(left_to_right(expr), Some(15));

        let expr = "3 * 2 + 4";
        assert_eq!(left_to_right(expr), Some(10));

        let expr = "3 + 2 * 4";
        assert_eq!(left_to_right(expr), Some(20));

        let expr = "3 * (2 + 4)";
        assert_eq!(left_to_right(expr), Some(18));

        let expr = "(3 * 2) + 4";
        assert_eq!(left_to_right(expr), Some(10));

        let expr = "1 + 2 * 3 + 4 * 5 + 6";
        assert_eq!(left_to_right(expr), Some(71));

        let expr = "1 + (2 * 3) + (4 * (5 + 6))";
        assert_eq!(left_to_right(expr), Some(51));

        let expr = "2 * 3 + (4 * 5)";
        assert_eq!(left_to_right(expr), Some(26));

        let expr = "5 + (8 * 3 + 9 + 3 * 4 * 3)";
        assert_eq!(left_to_right(expr), Some(437));

        let expr = "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))";
        assert_eq!(left_to_right(expr), Some(12240));

        let expr = "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2";
        assert_eq!(left_to_right(expr), Some(13632));
    }

    #[test]
    fn add_before_mult_test() {
        let expr = "1 + 3";
        assert_eq!(add_before_mult(expr), Some(4));

        let expr = "3 * 5";
        assert_eq!(add_before_mult(expr), Some(15));

        let expr = "3 * 2 + 4";
        assert_eq!(add_before_mult(expr), Some(18));

        let expr = "3 + 2 * 4";
        assert_eq!(add_before_mult(expr), Some(20));

        let expr = "3 * (2 + 4)";
        assert_eq!(add_before_mult(expr), Some(18));

        let expr = "(3 * 2) + 4";
        assert_eq!(add_before_mult(expr), Some(10));

        let expr = "1 + 2 * 3 + 4 * 5 + 6";
        assert_eq!(add_before_mult(expr), Some(231));

        let expr = "1 + (2 * 3) + (4 * (5 + 6))";
        assert_eq!(add_before_mult(expr), Some(51));

        let expr = "2 * 3 + (4 * 5)";
        assert_eq!(add_before_mult(expr), Some(46));

        let expr = "5 + (8 * 3 + 9 + 3 * 4 * 3)";
        assert_eq!(add_before_mult(expr), Some(1445));

        let expr = "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))";
        assert_eq!(add_before_mult(expr), Some(669060));

        let expr = "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2";
        assert_eq!(add_before_mult(expr), Some(23340));
    }

    #[test]
    fn test_51() {
        let expr = "1 + (2 * 3) + (4 * (5 + 6))";
        assert_eq!(add_before_mult(expr), Some(51));
    }
}
