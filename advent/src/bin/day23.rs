use std::io::prelude::*;
use std::collections::{VecDeque, BTreeSet};

#[derive(Debug)]
struct RingBuffer {
    // We use a pair of queues to implement a ring of entries,
    left: VecDeque<u32>,
    right: VecDeque<u32>,
    m: u32 // the greatest value in the ring, inclusive
}

impl RingBuffer {
    fn parse(line: &str, m: u32) -> Option<RingBuffer> {
        let mut items: BTreeSet<u32> = BTreeSet::new();
        let mut right: VecDeque<u32> = VecDeque::new();

        for c in line.chars() {
            match c.to_digit(10) {
                Some(x) if x > m => return None,
                Some(x) if !items.contains(&x) => {
                    items.insert(x);
                    right.push_back(x);
                },
                _ => return None
            }
        }

        let max_provided: u32 = match items.iter().max() {
            None => return None,
            Some(&max) if max as usize != right.len() => return None,
            Some(&max) => max
        };

        let mut left = VecDeque::new();
        match right.pop_front() {
            None => return None,
            Some(h) => left.push_back(h)
        };

        for x in (max_provided + 1)..=m {
            right.push_back(x);
        }

        Some(RingBuffer{ left, right, m })
    }

    // extracts the n items in the ring immediately following 1
    fn key(&self, n: usize) -> Vec<String> {
        let mut r = Vec::new();
        use std::collections::vec_deque::Iter;

        enum Buffer<'a> {
            Left(Iter<'a, u32>, bool),
            Right(Iter<'a, u32>, bool)
        }

        let mut buffer = Buffer::Left(self.left.iter(), false);

        while r.len() < n {
            match buffer {
                Buffer::Left(ref mut j, true) => {
                    match j.next() {
                        Some(digit) => r.push(digit.to_string()),
                        None => buffer = Buffer::Right(self.right.iter(), true)
                    }
                },
                Buffer::Right(ref mut j, true) => {
                    match j.next() {
                        Some(digit) => r.push(digit.to_string()),
                        None => buffer = Buffer::Left(self.left.iter(), true)
                    }
                },
                Buffer::Left(ref mut j, false) => {
                    match j.next() {
                        Some(1) => buffer = Buffer::Left(j.clone(), true),
                        Some(_) => (),
                        None => buffer = Buffer::Right(self.right.iter(), false)
                    }
                },
                Buffer::Right(ref mut j, false) => {
                    match j.next() {
                        Some(1) => buffer = Buffer::Right(j.clone(), true),
                        Some(_) => (),
                        None => buffer = Buffer::Left(self.left.iter(), false)
                    }
                }
            }
        }

        r
    }

    fn step(&mut self) -> Result<(), String> {
        // play one move of the crab's game
        let current: u32 = match self.left.back().or_else(|| self.right.front()) {
            None => {
                return Err("Unable to get current item".to_owned())
            },
            Some(c) => *c
        };
        let mut pickup: Vec<u32> = vec!();
        if self.left.len() + self.right.len() < 3 {
            let msg = format!("Cannot perform step with {} < 3 items.", self.left.len() + self.right.len());
            return Err(msg)
        }
        while pickup.len() < 3 {
            if let Some(x) = self.right.pop_front() {
                pickup.push(x);
            } else {
                std::mem::swap(&mut self.left, &mut self.right);
            }
        }

        let dest = {
            let mut dest = current - 1;
            loop {
                if dest == 0 {
                    dest = self.m;
                } else if pickup.contains(&dest) {
                    dest -= 1;
                } else {
                    break
                }
            }
            dest
        };
        let dest_queue: &mut VecDeque<u32> = if self.left.contains(&dest) {
            &mut self.left
        } else {
            &mut self.right
        };
        let dest_idx: usize = {
            let opt_idx: Option<usize> = dest_queue.iter().enumerate().fold(None, |acc, (idx, x)| {
                if x == &dest {
                    Some(idx)
                } else { acc }
            });
            match opt_idx {
                None => {
                    let msg = format!("Destination {} not found", dest);
                    return Err(msg)
                },
                Some(idx) => idx
            }
        };

        while let Some(x) = pickup.pop() {
            dest_queue.insert(dest_idx + 1, x);
        }

        if let Some(next_current) = self.right.pop_front() {
            self.left.push_back(next_current);
        } else if let Some(next_current) = self.left.pop_front() {
            std::mem::swap(&mut self.left, &mut self.right);
            self.left.push_back(next_current);
        } else {
            let msg = format!("Empty!");
            return Err(msg)
        }

        Ok(())
    }
}

fn main() {
    let stdin = std::io::stdin();
    let line = stdin.lock().lines().flatten().next().unwrap();
    let mut ring = RingBuffer::parse(&line, 9).unwrap();
    println!("Parsed ring: {:?}", ring);

    for _ in 0..100 {
        ring.step().unwrap();
    }

    println!("Cup labels after 100 moves: {}", ring.key(9).join(""));

    let mut ring1m = RingBuffer::parse(&line, 1_000_000).unwrap();
    for idx in 0..10_000_000 {
        if (idx % 0x400) == 0 {
            println!("Step {}", idx);
        }
        ring1m.step().unwrap();
    }
    
    println!("Big cup labels after 10M moves: {:?}", ring1m.key(2));
    
}

#[cfg(test)]
mod day23_spec {
    use super::*;

    fn collect_left(ring: &RingBuffer) -> Vec<u32> {
        ring.left.iter().map(|p| *p).collect()
    }

    fn collect_right(ring: &RingBuffer) -> Vec<u32> {
        ring.right.iter().map(|p| *p).collect()
    }

    #[test]
    fn parse_test() {
        let line = "389125467";
        let ring = RingBuffer::parse(line, 9).unwrap();
        assert_eq!(ring.m, 9);

        assert_eq!(collect_left(&ring), vec!(3));
        assert_eq!(collect_right(&ring), vec!(8, 9, 1, 2, 5, 4, 6, 7));
    }

    #[test]
    fn step_test() {
        let line = "389125467";
        let mut ring = RingBuffer::parse(line, 9).unwrap();

        ring.step().unwrap();
        assert_eq!(collect_left(&ring), vec!(3,2));
        assert_eq!(collect_right(&ring), vec!(8, 9, 1, 5, 4, 6, 7));

        ring.step().unwrap();
        assert_eq!(collect_left(&ring), vec!(3, 2, 5));
        assert_eq!(collect_right(&ring), vec!(4, 6, 7, 8, 9, 1));

        ring.step().unwrap();
        assert_eq!(collect_left(&ring), vec!(3, 4, 6, 7, 2, 5, 8));
        assert_eq!(collect_right(&ring), vec!(9, 1));

        ring.step().unwrap();
        assert_eq!(collect_left(&ring), vec!(4));
        assert_eq!(collect_right(&ring), vec!(6, 7, 9, 1, 3, 2, 5, 8));
    }

    #[test]
    fn key_test() {
        let line = "389125467";
        let mut ring = RingBuffer::parse(line, 9).unwrap();

        assert_eq!(ring.key(8).join(""), "25467389");

        for _ in 0..10 {
            ring.step().unwrap();
        }

        assert_eq!(ring.key(8).join(""), "92658374");

        for _ in 10..100 {
            ring.step().unwrap()
        }

        assert_eq!(ring.key(8).join(""), "67384529");
    }

    #[test]
    fn ring_20_test() {
        let line = "389125467";
        let mut ring = RingBuffer::parse(line, 20).unwrap();

        assert_eq!(collect_left(&ring), vec!(3));
        assert_eq!(collect_right(&ring), vec!(8, 9, 1, 2, 5, 4, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20));

        ring.step().unwrap();
    }
}