use std::io::prelude::*;
use std::collections::{BinaryHeap};

use modinverse::egcd;

// computes the modular additive inverse of x (mod p)
fn modular_negative(x: usize, p: usize) -> usize {
    let m = x % p;
    match m {
        0 => 0,
        _ => p - m
    }
}

fn min_by<T, J, F, U>(ts: &mut J, f: F) -> Option<T>
where J: Iterator<Item=T>, F: Fn(&T) -> U, U: Ord {
    let mut r: Option<(T, U)> = None;

    while let Some(t) = ts.next() {
        let u = f(&t);
        match r {
            None => {
                r = Some((t, u))
            },
            Some((_, ref prev_u)) if u < *prev_u => {
                r = Some((t,u))
            },
            _ => ()
        }
    }

    r.map(|p| p.0)
}

fn soonest_bus(current_time: usize, bus_ids: &str) -> Option<(usize, usize)> {
    let mut ids = bus_ids.split(',').flat_map(|id| usize::from_str_radix(id, 10));
    min_by(&mut ids, |t| modular_negative(current_time, *t)).map(|bus_id| (bus_id, bus_id - (current_time % bus_id)))
}

fn bus_constraints(bus_ids: &str) -> BinaryHeap<(usize, usize)> {
    let mut cs = BinaryHeap::new();

    for (idx, bus_id) in bus_ids.split(',').enumerate() {
        for id in usize::from_str_radix(bus_id, 10) {
            cs.push((id, modular_negative(idx, id)));
        }
    }

    cs
}

// We need to use signed int here because the egcd crate author does unsafe multiplication
fn chinese_remainder(mut constraints: BinaryHeap<(usize, usize)>) -> Option<i128> {
    let mut p = 1;
    let mut s: i128 = 0;

    while let Some((n, rem)) = constraints.pop() {
        let rem: i128 = rem as i128;
        let n: i128 = n as i128;
        if let (1, cp, cn) = egcd(p, n) {
            let next_p = p * n;
            s = (cp * rem * p + cn * s * n) % next_p;
            if s < 0 {
                s += next_p;
            }
            p = next_p;
        } else {
            eprintln!("Non-relatively-prime inputs discovered!");
            return None
        }
    }

    return Some(s)
}


fn main() {
    let stdin = std::io::stdin();
    let mut stdin_lines = stdin.lock().lines().flatten();
    let current_time = stdin_lines.next().and_then(|s| usize::from_str_radix(&s, 10).ok()).unwrap();
    let bus_ids_line = stdin_lines.next().unwrap();
    println!("Part 1:");
    let (bus_id, wait_time) = soonest_bus(current_time, &bus_ids_line).unwrap();
    println!("The first bus to leave will be #{} in {} minutes. Part 1: {}", bus_id, wait_time, bus_id * wait_time);

    println!("Part 2:");
    let constraints = bus_constraints(&bus_ids_line);
    let departure_time = chinese_remainder(constraints).unwrap();
    println!("Earliest timestamp: {}", departure_time);
}

#[cfg(test)]
mod day13_spec {
    use super::*;

    #[test]
    fn min_by_test() {
        let xs = vec!(11, 14, 15, 17);
        let m = min_by(&mut xs.iter(), |&x| x % 3).unwrap();
        assert_eq!(m, &15);
    }

    #[test]
    fn soonest_bus_test() {
        let bus_id = soonest_bus(939, "7,13,59,31,19").unwrap();
        assert_eq!(bus_id, (59, 5));

        let bus_id = soonest_bus(939, "7,13,x,x,59,x,31,19").unwrap();
        assert_eq!(bus_id, (59, 5));
    }

    #[test]
    fn bus_constraints_test() {
        let bus_id_line = "7,13,x,x,59,x,31,19";
        let mut constraints = bus_constraints(bus_id_line);
        assert_eq!(constraints.pop(), Some((59, 55)));
        assert_eq!(constraints.pop(), Some((31, 25)));
        assert_eq!(constraints.pop(), Some((19, 12)));
        assert_eq!(constraints.pop(), Some((13, 12)));
        assert_eq!(constraints.pop(), Some((7, 0)));
        assert_eq!(constraints.pop(), None);

        let bus_id_line = "5,x,x,7,x,x,3,11";
        let mut constraints = bus_constraints(bus_id_line);
        assert_eq!(constraints.pop(), Some((11, 4)));
        assert_eq!(constraints.pop(), Some((7, 4)));
        assert_eq!(constraints.pop(), Some((5, 0)));
        assert_eq!(constraints.pop(), Some((3, 0)));
    }

    #[test]
    fn chinese_remainder_test() {
        let mut constraints: BinaryHeap<(usize, usize)> = vec!(
            (7, 0),
            (13, 12),
            (19, 12),
            (31, 25),
            (59, 55)
        ).into_iter().collect();
        let cr = chinese_remainder(constraints).unwrap();
        assert_eq!(cr, 1068781);

        constraints = vec!(
            (17, 0),
            (13, 13 - 2),
            (19, 19 - 3)
        ).into_iter().collect();
        let cr = chinese_remainder(constraints).unwrap();
        assert_eq!(cr, 3417);

        constraints = vec!(
            (67, 0),
            (7, 7 - 1),
            (59, 59 - 2),
            (61, 61 - 3)
        ).into_iter().collect();
        let cr = chinese_remainder(constraints).unwrap();
        assert_eq!(cr, 754018);

        constraints = vec!(
            (67, 0),
            (7, 7 - 2),
            (59, 59 - 3),
            (61, 61 - 4)
        ).into_iter().collect();
        let cr = chinese_remainder(constraints).unwrap();
        assert_eq!(cr, 779210);
    }

}