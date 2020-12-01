use std::io;
use std::io::prelude::*;

use std::collections::BTreeSet;

fn decompose_2(summands: &BTreeSet<usize>, target: usize) -> Option<(usize, usize)> {
    for &s in summands {
        if s <= target {
            let diff = target - s;
            if s != diff && summands.contains(&diff) {
                return Some((s, diff))
            }
        }
    };
    return None
}

fn decompose_3(summands: &BTreeSet<usize>, target: usize) -> Option<(usize, usize, usize)> {
    let mut summands_copy: BTreeSet<usize> = summands.clone();
    for &s in summands {
        if s <= target {
            summands_copy.remove(&s);
            if let Some((s0, s1)) = decompose_2(&summands_copy, target - s) {
                return Some((s, s0, s1))
            }
            summands_copy.insert(s);
        }
    };
    return None
}

fn main() {
    let stdin = io::stdin();
    let expenses: BTreeSet<usize> = stdin.lock().lines().flatten()
        .flat_map(|s| usize::from_str_radix(&s, 10)).collect();
    println!("Part 1:");
    let (e0, e1) = decompose_2(&expenses, 2020).unwrap();
    println!("Found expenses {}, {}. Product: {}", e0, e1, e0*e1);

    println!("Part 2:");
    let (e0, e1, e2) = decompose_3(&expenses, 2020).unwrap();
    println!("Found expenses {}, {}, {}. Product: {}", e0, e1, e2, e0*e1*e2);
}

#[cfg(test)]
mod decompose {
    use super::*;

    #[test]
    fn decompose_2_should_decompose_a_target() {
        let summands: BTreeSet<usize> = vec!(1,2,3).iter().map(|x| *x).collect();
        let (x0, x1) = decompose_2(&summands, 4).unwrap();
        if x0 < x1 {
            assert_eq!(x0, 1);
            assert_eq!(x1, 3)
        } else {
            assert_eq!(x0, 3);
            assert_eq!(x1, 1);
        }
    }

    #[test]
    fn decompose_2_should_not_reuse() {
        let summands: BTreeSet<usize> = vec!(1,2,3).iter().map(|x| *x).collect();
        assert_eq!(decompose_2(&summands, 6), None);
    }

    #[test]
    fn decompose_3_should_decompose_a_target() {
        let summands: BTreeSet<usize> = vec!(1,2,3,4).iter().map(|x| *x).collect();
        let (x0, x1, x2) = decompose_3(&summands, 8).unwrap();
        let mut xs: [usize;3] = [x0, x1, x2];
        xs.sort();
        assert_eq!(xs, [1, 3, 4]);
    }

    #[test]
    fn decompose_3_should_not_reuse() {
        let summands: BTreeSet<usize> = vec!(1,2,3,4).iter().map(|x| *x).collect();
        assert_eq!(decompose_3(&summands, 3), None);
        assert_eq!(decompose_3(&summands, 10), None);
    }
}