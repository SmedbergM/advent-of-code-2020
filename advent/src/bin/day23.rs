use std::io::prelude::*;
use std::collections::BTreeMap;

use advent::make_string::MakeString;

#[derive(Debug, PartialEq, Eq)]
struct RingNode {
    prev: u32,
    next: u32
}


/*  A nonempty circular arrangement of items, in this case u32. It would be relatively simple to make this generic over any Copy + Ord type,
    but that's not worth the trouble here.

    `point` is guaranteed to be a key in the map; similarly, it is an invariant of the map that at the end of any method body, hopping `next` pointers
    and hopping `prev` pointers will traverse the entire keyset in the same cycle (in reverse order).
*/
struct Ring {
    nodes: BTreeMap<u32, RingNode>,
    point: u32
}

impl Ring {

    fn new<J>(mut j: J) -> Result<Ring, String> where J: Iterator<Item=u32> {
        let mut nodes = BTreeMap::new();
        let first_node: u32;
        match j.next() {
            None => return Err("Ring must be non-empty".to_owned()),
            Some(c) => {
                first_node = c;
                nodes.insert(c, RingNode { prev: c, next: c });
            }
        }
        let mut last_node: u32 = first_node;
        while let Some(c) = j.next() {
            if nodes.contains_key(&c) {
                let msg = format!("Duplicate entry {} in interator", c);
                return Err(msg)
            }
            for node in nodes.get_mut(&last_node) {
                node.next = c;
            }
            for node in nodes.get_mut(&first_node) {
                node.prev = c;
            }
            nodes.insert(c, RingNode { prev: last_node, next: first_node });
            last_node = c;
        }

        Ok(Ring { nodes, point: first_node })
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn max(&self) -> &u32 {
        self.nodes.keys().rev().nth(0).unwrap()
    }

    fn advance_clockwise(&mut self) {
        for node in self.nodes.get(&self.point) {
            self.point = node.next;
        }
    }

    fn insert_after(&mut self, existing_node: u32, new_node: u32) -> Result<(), String> {
        match self.nodes.get(&existing_node).map(|node| node.next) {
            None => {
                let msg = format!("Node {} not found in ring", existing_node);
                Err(msg)
            },
            Some(c) => {
                for node in self.nodes.get_mut(&existing_node) {
                    node.next = new_node;
                }
                for node in self.nodes.get_mut(&c) {
                    node.prev = new_node;
                }
                self.nodes.insert(new_node, RingNode { prev: existing_node, next: c});
                Ok(())
            }
        }
    }

    // Removes and returns, in order, the `n` entries clockwise from (but not including) `self.point`. If the starting
    // size of the ring is less than or equal to `n`, no modification is performed and an Err is returned.
    fn remove_after_point(&mut self, n: usize) -> Result<Vec<u32>, String> {
        if self.len() > n {
            let mut r: Vec<u32> = vec!();

            while let Some(RingNode { next, .. }) = self.nodes.get(&self.point) {
                if r.len() == n {
                    break
                }
                let next_node: u32 = *next;
                r.push(next_node);
                match self.nodes.remove(&next_node) {
                    Some(RingNode { next: next2, .. }) => {
                        for node in self.nodes.get_mut(&self.point) {
                            node.next = next2;
                        }
                        for node in self.nodes.get_mut(&next2) {
                            node.prev = self.point;
                        }
                    },
                    None => {
                        let msg = format!("No adjacency information for node {}! This should never happen.", next_node);
                        return Err(msg)
                    }
                }
            }

            Ok(r)
        } else {
            let msg = format!("Ring is too small for requested operation; {} removals requested, but only {} elements are available.", n, self.len());
            Err(msg)
        }
    }

    fn iter(&self) -> RingIterator {
        RingIterator { ring: self, start: self.point, last_yielded: None}
    }

    fn iter_from(&self, start: u32) -> RingIterator {
        RingIterator { ring: self, start, last_yielded: None }
    }
}


struct RingIterator<'a> {
    ring: &'a Ring,
    start: u32,
    last_yielded: Option<&'a u32>
}

impl<'a> Iterator for RingIterator<'a> {
    type Item = &'a u32;

    fn next(&mut self) -> Option<&'a u32> {
        match self.last_yielded {
            None => {
                let y = self.ring.nodes.get(&self.start).map(|node| &node.next);
                self.last_yielded = y;
                y
            },
            Some(y_prev) if *y_prev == self.start => None,
            Some(y_prev) => {
                let y = self.ring.nodes.get(y_prev).map(|node| &node.next);
                self.last_yielded = y;
                y
            }
        }
    }

}

// One step of the crab game
fn crab_step(ring: &mut Ring) -> Result<(), String> {
    let mut removed = ring.remove_after_point(3)?;

    let destination: u32 = {
        let mut d = ring.point;

        loop {
            match d {
                0 => d = *ring.max(),
                _ => d -= 1
            };
            if ring.nodes.contains_key(&d) {
                break
            }
        }

        d
    };

    while let Some(c) = removed.pop() {
        ring.insert_after(destination, c)?;
    }

    Ok(ring.advance_clockwise())
}

fn main() {
    let stdin = std::io::stdin();
    let line = stdin.lock().lines().flatten().next().unwrap();
    let mut ring = Ring::new(line.chars().flat_map(|c| c.to_digit(10))).unwrap();

    let label: String = ring.iter().take(8).mk_string("");
    println!("Initial ring label: {}", label);
    let label: String = ring.iter_from(1).take(8).mk_string("");
    println!("Initial ring label, starting from 1: {}", label);
    
    for _ in 0..100 {
        crab_step(&mut ring).unwrap();
    }

    let label: String = ring.iter_from(1).take(8).mk_string("");
    println!("Ring label after 100 steps: {}", label);

    let mut ring1m = {
        let nodes = Iterator::chain(
            line.chars().flat_map(|c| c.to_digit(10)),
            (*ring.max() + 1)..=1_000_000
        );
        Ring::new(nodes).unwrap()
    };

    for _i in 0..10_000_000 {
        crab_step(&mut ring1m).unwrap();
    }

    let labels: Vec<u64> = ring1m.iter_from(1).take(2).map(|x| *x as u64).collect();
    println!("After 10M steps, {:?} follows 1", labels);
    let p: u64 = 1u64 * labels[0] * labels[1];
    println!("Product of labels: {}", p);
}

#[cfg(test)]
mod day23_spec {
    use super::*;

    #[test]
    fn crab_step_test() {
        let mut ring = {
            let nodes = vec!(3, 8, 9, 1, 2, 5, 4, 6, 7);
            Ring::new(nodes.into_iter()).unwrap()
        };
        crab_step(&mut ring).unwrap();

        assert_eq!(ring.point, 2);
        assert_eq!(ring.nodes.get(&2), Some(&RingNode{ prev: 3, next: 8 }));
        assert_eq!(ring.iter().mk_string(""), "891546732");

        crab_step(&mut ring).unwrap();
        assert_eq!(ring.point, 5);
        assert_eq!(ring.nodes.get(&5), Some(&RingNode{ prev: 2, next: 4 }));
        assert_eq!(ring.iter().mk_string(""), "467891325");

        crab_step(&mut ring).unwrap();
        assert_eq!(ring.point, 8);
        assert_eq!(ring.iter().mk_string(""), "913467258");

        crab_step(&mut ring).unwrap();
        assert_eq!(ring.point, 4);
        assert_eq!(ring.iter().mk_string(""), "679132584");
    }


    mod ring {
        use super::*;

        #[test]
        fn new_test() {
            let items = vec!(1,3,4,5);
            let ring = Ring::new(items.into_iter()).unwrap();
            assert_eq!(ring.nodes.get(&1), Some(&RingNode{ prev: 5, next: 3 }));
            assert_eq!(ring.nodes.get(&3), Some(&RingNode{ prev: 1, next: 4 }));
            assert_eq!(ring.nodes.get(&4), Some(&RingNode{ prev: 3, next: 5 }));
            assert_eq!(ring.nodes.get(&5), Some(&RingNode{ prev: 4, next: 1 }));

            assert_eq!(ring.len(), 4);
            assert_eq!(ring.max(), &5);

            let items = vec!(3,4,3,5);
            assert!(Ring::new(items.into_iter()).is_err());
        }

        #[test]
        fn insert_after_test() {
            let items = vec!(2);
            let mut ring = Ring::new(items.into_iter()).unwrap();

            assert_eq!(ring.nodes.get(&2), Some(&RingNode{ prev: 2, next: 2}));

            ring.insert_after(2, 4).unwrap();

            assert_eq!(ring.len(), 2);
            assert_eq!(ring.nodes.get(&2), Some(&RingNode{ prev: 4, next: 4}));
            assert_eq!(ring.nodes.get(&4), Some(&RingNode{ prev: 2, next: 2}));

            // NB: an invalid insert does not put the ring into an inconsistent state.
            assert!(ring.insert_after(1, 3).is_err());

            ring.insert_after(2, 3).unwrap();

            assert_eq!(ring.len(), 3);
            assert_eq!(ring.nodes.get(&2), Some(&RingNode{ prev: 4, next: 3}));
            assert_eq!(ring.nodes.get(&4), Some(&RingNode{ prev: 3, next: 2}));
            assert_eq!(ring.nodes.get(&3), Some(&RingNode{ prev: 2, next: 4}));
        }

        #[test]
        fn remove_after_test() {
            let mut ring = Ring::new((0..10).into_iter().rev()).unwrap();
            ring.point = 5;
            let removed = ring.remove_after_point(3).unwrap();
            assert_eq!(removed, vec!(4, 3, 2));
            assert_eq!(ring.point, 5);
            assert_eq!(ring.len(), 7);
            assert_eq!(ring.max(), &9);

            ring.point = 1;
            let removed = ring.remove_after_point(4).unwrap();
            assert_eq!(removed, vec!(0, 9, 8, 7));
            assert_eq!(ring.point, 1);
            assert_eq!(ring.len(), 3);
            assert_eq!(ring.max(), &6);
            assert_eq!(ring.nodes.get(&1), Some(&RingNode { prev: 5, next: 6 }));
            assert_eq!(ring.nodes.get(&6), Some(&RingNode { prev: 1, next: 5 }));
            assert_eq!(ring.nodes.get(&5), Some(&RingNode { prev: 6, next: 1 }));

            match ring.remove_after_point(4) {
                Ok(_) => panic!(),
                Err(msg) => assert_eq!(msg, "Ring is too small for requested operation; 4 removals requested, but only 3 elements are available.")
            };

            match ring.remove_after_point(3) {
                Ok(_) => panic!(),
                Err(msg) => assert_eq!(msg, "Ring is too small for requested operation; 3 removals requested, but only 3 elements are available.")
            };

            let removed = ring.remove_after_point(2).unwrap();
            assert_eq!(removed, vec!(6, 5));
        }
    }
}