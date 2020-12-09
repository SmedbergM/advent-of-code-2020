use std::io;
use std::io::prelude::*;


// Finds the index of the first element of xs which cannot be decomposed as the sum of two different elements in the
// previous `lookback`
fn indecomposeable(xs: &Vec<u32>, lookback: usize) -> Option<usize> {
    fn can_decompose(summands: &[u32], target: &u32) -> bool {
        for idx0 in 0..summands.len() {
            let s0 = &summands[idx0];
            for s1 in &summands[idx0 + 1..] {
                if (*s0 + *s1) == *target {
                    return true
                }
            }
        }
        return false
    }

    for idx in lookback..xs.len() {
        if !can_decompose(&xs[idx - lookback..idx], &xs[idx]) {
            return Some(idx)
        }
    }
    return None
}


// decomposes `target` into a sum of consecutive elements of `summands` if possible
fn decompose<'a>(summands: &'a [u32], target: u32) -> Option<&'a [u32]> {
    'outer: for idx0 in 0..summands.len() {
        for idx1 in idx0..summands.len() {
            let consecutive_sum: u32 = summands[idx0..idx1].iter().sum();
            if consecutive_sum == target {
                return Some(&summands[idx0..idx1])
            } else if consecutive_sum > target {
                continue 'outer
            }
        }
    }
    return None
}

fn min_max<'a>(slice: &'a [u32]) -> Option<(u32, u32)> {
    slice.iter().fold(None, |acc, x| {
        match acc {
            None => Some((*x, *x)),
            Some((min, max)) if *x < min => Some((*x, max)),
            Some((min, max)) if max < *x => Some((min, *x)),
            acc => acc
        }
    })
}

fn main() {
    let stdin = io::stdin();
    let input: Vec<u32> = stdin.lock().lines()
        .flatten()
        .flat_map(|line| u32::from_str_radix(&line, 10))
        .collect();

    let idx0 = indecomposeable(&input, 25).unwrap();
    let offending_value = input[idx0];
    println!("Indecomposable XMAS value: {} at index {}.", offending_value, idx0);

    let xs = decompose(&input[..idx0], offending_value)
        .or_else(|| decompose(&input[idx0+1..], offending_value))
        .unwrap();
    println!("Sum slice: {:?}", xs);
    let (x0, x1) = min_max(xs).unwrap();
    println!("Bounds of sum slice: {}, {}. Min/Max Sum: {}", x0, x1, x0 + x1);
}

#[cfg(test)]
mod day09_spec {
    use super::*;

    #[test]
    fn indecompose_test() {
        let input = vec!(1,2,3,6);
        assert_eq!(indecomposeable(&input, 3), Some(3));

        let input = vec!(1,2,3,5);
        assert_eq!(indecomposeable(&input, 3), None);

        let input = vec!(
            35, 20, 15, 25, 47,
            40, 62, 55, 65, 95,
            102, 117, 150, 182, 127,
            219, 299, 277, 309, 576);
        assert_eq!(indecomposeable(&input, 5), Some(14));
    }

    #[test]
    fn decompose_test() {
        let input = vec!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
        let xs = decompose(&input[..8], 9).unwrap();
        assert_eq!(xs, [2,3,4]);

        let input = vec!(1, 3, 5, 7, 9);
        assert_eq!(decompose(&input[..], 2), None);

        let input = vec!(
            35, 20, 15, 25, 47,
            40, 62, 55, 65, 95,
            102, 117, 150, 182, 127,
            219, 299, 277, 309, 576
        );
        let xs = decompose(&input[..14], 127).unwrap();
        assert_eq!(xs, [15, 25, 47, 40]);
    }
}
