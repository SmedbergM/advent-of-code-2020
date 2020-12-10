use std::io;
use std::io::prelude::*;

use std::collections::BTreeMap;

// for both count_differences and count_paths, xs must be pre-sorted
fn count_differences(xs: &Vec<u16>) -> BTreeMap<u16, u64> {
    let mut r = BTreeMap::new();

    for idx in 1..xs.len() {
        let d = xs[idx] - xs[idx - 1];
        *r.entry(d).or_insert(0) += 1;
    }
    r
}

fn count_paths(xs: &Vec<u16>) -> u64 {
    // ps[x] represents the number of paths from x to the sink
    let mut ps: BTreeMap<u16, u64> = BTreeMap::new();
    ps.insert(xs[xs.len() - 1], 1);

    fn get_or_zero(ps: &BTreeMap<u16, u64>, k: u16) -> u64 {
        *ps.get(&k).unwrap_or(&0)
    }

    for idx in (0..xs.len()).rev() {
        let x = xs[idx];
        if let None = ps.get(&x) {
            let p1 = get_or_zero(&ps, x + 1);
            let p2 = get_or_zero(&ps, x + 2);
            let p3 = get_or_zero(&ps, x + 3);
            ps.insert(x, p1 + p2 + p3);
        }
    }

    get_or_zero(&ps, 0)
}

fn main() {
    let stdin = io::stdin();
    let jolts = {
        let mut m = 0;
        let mut jolts: Vec<u16> = stdin.lock().lines().flatten()
            .flat_map(|line| u16::from_str_radix(&line, 10))
            .map(|x| {
                if x > m {
                    m = x
                };
                x
            })
            .collect();
        jolts.push(0);
        jolts.push(m + 3);
        jolts.sort();
        jolts
    };

    let diffs = count_differences(&jolts);
    println!("Challenge 1: {} * {} = {}", diffs[&1], diffs[&3], diffs[&1] * (diffs[&3]));

    let path_count = count_paths(&jolts);
    println!("There are {} paths.", path_count);
}

#[cfg(test)]
mod day10_spec {
    use super::*;

    #[test]
    fn count_differences_test() {
        let mut jolts = vec!(0, 1, 4, 5, 6, 7, 10, 11, 12, 15, 16, 19, 22);
        let mut expected: BTreeMap<u16, u64> = vec!((1,7), (3,5)).iter().map(|p| *p).collect();
        assert_eq!(count_differences(&jolts), expected);

        jolts = vec!();
        jolts.extend(0..5);
        jolts.extend(7..12);
        jolts.push(14);
        jolts.extend(17..21);
        jolts.extend(23..26);
        jolts.push(28);
        jolts.extend(31..36);
        jolts.extend(38..40);
        jolts.push(42);
        jolts.extend(45..50);
        jolts.push(52);
        expected = vec!((1, 22), (3, 10)).iter().map(|p| *p).collect();
        assert_eq!(count_differences(&jolts), expected);
    }

    #[test]
    fn count_paths_test() {
        let mut jolts = vec!(0, 1, 4, 5, 6, 7, 10, 11, 12, 15, 16, 19, 22);
        assert_eq!(count_paths(&jolts), 8);

        jolts = vec!();
        jolts.extend(0..5);
        jolts.extend(7..12);
        jolts.push(14);
        jolts.extend(17..21);
        jolts.extend(23..26);
        jolts.push(28);
        jolts.extend(31..36);
        jolts.extend(38..40);
        jolts.push(42);
        jolts.extend(45..50);
        jolts.push(52);
        assert_eq!(count_paths(&jolts), 19208);
    }
}
