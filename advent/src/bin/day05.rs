use std::io;
use std::io::prelude::*;

use std::collections::{BinaryHeap, BTreeSet};

fn seat_id(k: &str) -> Option<usize> {
    let b: String = k.chars().flat_map(|c| match c {
        'F' | 'L' => Some('0'),
        'B' | 'R' => Some('1'),
        _ => None
    }).collect();
    usize::from_str_radix(b.as_ref(), 2).ok()
}

fn open_seat(ids: &BTreeSet<usize>)-> Option<usize> {
    for &x in ids {
        let candidate = x + 1;
        if ids.contains(&(candidate + 1)) && !ids.contains(&candidate) {
            return Some(candidate)
        }
    };
    None
}

fn main() {
    let stdin = io::stdin();
    let seat_ids: BinaryHeap<usize> = stdin.lock().lines()
    .flatten()
    .flat_map(|line| seat_id(&line)).collect();
    let max_seat_id = seat_ids.peek().unwrap();

    println!("Max seat id: {}", max_seat_id);

    let seat_ids: BTreeSet<usize> = {
        seat_ids.iter().map(|j| *j).collect()
    };

    let my_seat = open_seat(&seat_ids).unwrap();
    println!("Open seat found at: {}", my_seat);
}

#[cfg(test)]
mod day05_spec {
    use super::*;

    #[test]
    fn seat_id_test() {
        assert_eq!(seat_id("FBFBBFFRLR"), Some(357));
        assert_eq!(seat_id("BFFFBBFRRR"), Some(567));
        assert_eq!(seat_id("FFFBBBFRRR"), Some(119));
        assert_eq!(seat_id("BBFFBBFRLL"), Some(820));
    }
}