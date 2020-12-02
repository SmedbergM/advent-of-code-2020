#[macro_use]
extern crate lazy_static;

use std::io;
use std::io::prelude::*;

use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
struct Password {
    c: char,
    idx0: usize,
    idx1: usize,
    word: String
}

impl Password {
    fn from(line: &str) -> Option<Password> {
        lazy_static! {
            static ref PAT: Regex = Regex::new(r"(\d+)-(\d+) (\w): (\w+)").unwrap();
        }
        PAT.captures(line).and_then(|caps| {
            usize::from_str_radix(&caps[1], 10).ok().and_then(|idx0| {
            usize::from_str_radix(&caps[2], 10).ok().and_then(|idx1| {
            caps[3].chars().next().map(|c| {
                let word = caps[4].to_owned();
                Password { idx0, idx1, c, word }
            }) }) })
        })
    }

    fn is_valid_1(&self) -> bool {
        let c_count: usize = self.word.chars().fold(0, |acc,c| acc + ((self.c == c) as usize));
        self.idx0 <= c_count && c_count <= self.idx1
    }

    fn is_valid_2(&self) -> bool {
        let bytes = self.word.as_bytes();
        let c = self.c as u8;
        let idx0 = self.idx0 - 1;
        let idx1 = self.idx1 - 1;
        (bytes[idx0] == c) ^ (bytes[idx1] == c)
    }
}

fn main() {
    let stdin = io::stdin();
    let corrupted_passwords: Vec<Password> = {
        stdin.lock().lines().flatten()
            .flat_map(|line| Password::from(&line)).collect()
    };

    println!("Parsed {} corrupted passwords", corrupted_passwords.len());

    let valid: usize = corrupted_passwords.iter().fold(0, |acc, cp| acc + (cp.is_valid_1() as usize));
    println!("{} passwords are valid.", valid);

    let valid_2: usize = corrupted_passwords.iter().fold(0, |acc, cp| acc + (cp.is_valid_2() as usize));
    println!("{} passwords are valid in the second sense.", valid_2);
}

#[cfg(test)]
mod day02_spec {
    mod from {
        use super::super::*;

        #[test]
        fn should_parse_a_line() {
            assert_eq!(Password::from("1-3 a: abcde"), Some(Password{
                idx0: 1, idx1: 3, c: 'a', word: "abcde".to_owned()
            }));
            
            assert_eq!(Password::from("1-3 b: cdefg"), Some(Password{
                idx0: 1, idx1: 3, c: 'b', word: "cdefg".to_owned()
            }));

            assert_eq!(Password::from("2-9 c: ccccccccc"), Some(Password{
                idx0: 2, idx1: 9, c: 'c', word: "ccccccccc".to_owned()
            }))
        }
    }

    mod is_valid_1 {
        use super::super::*;

        #[test]
        fn should_validate() {
            let pw = Password{
                idx0: 1, idx1: 3, c: 'a', word: "abcde".to_owned()
            };
            assert!(pw.is_valid_1());

            let pw = Password{
                idx0: 1, idx1: 3, c: 'b', word: "cdefg".to_owned()
            };
            assert!(!pw.is_valid_1());

            let pw = Password{
                idx0: 2, idx1: 9, c: 'c', word: "ccccccccc".to_owned()
            };
            assert!(pw.is_valid_1());
        }
    }

    mod is_valid_2 {
        use super::super::*;

        #[test]
        fn should_validate() {
            let pw = Password{
                idx0: 1, idx1: 3, c: 'a', word: "abcde".to_owned()
            };
            assert!(pw.is_valid_2());

            let pw = Password{
                idx0: 1, idx1: 3, c: 'b', word: "cdefg".to_owned()
            };
            assert!(!pw.is_valid_2());

            let pw = Password{
                idx0: 2, idx1: 9, c: 'c', word: "ccccccccc".to_owned()
            };
            assert!(!pw.is_valid_2());
        }
    }
}
