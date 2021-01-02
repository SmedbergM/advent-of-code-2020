use std::io::prelude::*;
use std::collections::{HashSet, HashMap};
use std::hash::Hash;


#[derive(Debug, PartialEq, Eq, Hash)]
struct Point3 {
    x: isize, y: isize, z: isize
}


impl Point3 {
    fn new(x: isize, y: isize, z: isize) -> Point3 {
        Point3 { x, y, z }
    }

    fn neighbors(&self) -> impl Iterator<Item=Point3> {
        let (x, y, z) = (self.x, self.y, self.z); // So we don't have to fiddle with the lifetime of self
        (-1..=1).flat_map(move |dx|
        (-1..=1).flat_map(move |dy|
        (-1..=1).flat_map(move |dz|
            if (dx, dy, dz) == (0, 0, 0) {
                None
            } else {
                Some(Point3::new(x + dx, y + dy, z + dz))
            }
        )))
    }
}


#[derive(Debug, PartialEq, Eq, Hash)]
struct Point4 {
    w: isize, x: isize, y: isize, z: isize
}

impl Point4 {
    fn new(w: isize, x: isize, y: isize, z: isize) -> Point4 {
        Point4 { w, x, y, z }
    }

    fn neighbors(&self) -> impl Iterator<Item=Point4> {
        let (w, x, y, z) = (self.w, self.x, self.y, self.z);
        (-1..=1).flat_map(move |dx| 
        (-1..=1).flat_map(move |dy|
        (-1..=1).flat_map(move |dz|
        (-1..=1).flat_map(move |dw|
            if (dx, dy, dz, dw) == (0, 0, 0, 0) {
                None
            } else {
                Some(Point4::new(w + dw, x + dx, y + dy, z + dz))
            }
        ))))
    }
}

struct Conway<T> {
    cells: HashSet<T> // only record active cells
}

impl<T: Hash + Eq> Conway<T> {

    fn parse<L, F>(lines: L, f: F) -> Conway<T>
    where L: Iterator<Item=String>, F: Fn(isize, isize) -> T {
        let mut cells = HashSet::new();

        for (y, line) in lines.enumerate() {
            for (x, c) in line.chars().enumerate() {
                if c == '#' {
                    cells.insert(f(x as isize, y as isize));
                }
            }
        }

        Conway { cells }
    }

    fn evolve<J, F>(&self, neighbors: F) -> Conway<T>
    where J: Iterator<Item=T>, F: Fn(&T) -> J {
        let mut visited: HashMap<T, bool> = HashMap::new();

        // iterate over all neighbors of self's cells
        for cell in &self.cells {
            for candidate in neighbors(cell) {
                if !visited.contains_key(&candidate) {
                    
                    let mut active_neighbors = 0;
                    for nbr in neighbors(&candidate) {
                        active_neighbors += self.cells.contains(&nbr) as u8;
                        if active_neighbors > 3 {
                            break
                        }
                    }
                    match active_neighbors {
                        2 if self.cells.contains(&candidate) => {
                            visited.insert(candidate, true);
                        },
                        3 => {
                            visited.insert(candidate, true);
                        },
                        _ => {
                            visited.insert(candidate, false);
                        }
                    }
                }
            }
        }

        let cells = visited.into_iter().flat_map(|p| {
            match p {
                (x, true) => Some(x),
                _ => None
            }
        }).collect();
        Conway{ cells }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let conway3_0: Conway<Point3> = Conway::parse(stdin.lock().lines().flatten(), |x,y| Point3::new(x,y,0));
    let conway4_0: Conway<Point4> = {
        let cells: HashSet<Point4> = (&conway3_0.cells).iter().map(|c|
            Point4::new(0, c.x, c.y, 0)
        ).collect();
        Conway { cells }
    };

    let conway3_6 = (0..6).fold(conway3_0, |c, _| c.evolve(|p| p.neighbors()));

    println!("3D active cells after 6 generations: {}", conway3_6.cells.len());

    let conway4_6 = (0..6).fold(conway4_0, |c, _| c.evolve(|p| p.neighbors()));

    println!("4D Active cells after 6 generations: {}", conway4_6.cells.len());
}

#[cfg(test)]
mod day17_spec {
    use super::*;
    
    #[test]
    fn conway_parse_test() {
        let input = ".#.\n\
                     ..#\n\
                     ###";
        let conway: Conway<Point3> = Conway::parse(input.lines().map(|s| s.to_owned()), |x, y| Point3::new(x, y, 0));
        assert_eq!(conway.cells.len(), 5);
        assert!(conway.cells.contains(&Point3::new(0, 2, 0)))
    }

    #[test]
    fn conway_cycle_test() {
        let input = ".#.\n\
                     ..#\n\
                     ###";
        let conway: Conway<Point3> = Conway::parse(input.lines().map(|s| s.to_owned()),
            |x, y| Point3::new(x, y, 0));
        let conway1 = conway.evolve(|p| p.neighbors());

        assert_eq!(conway1.cells.len(), 11);
        assert!(conway1.cells.contains(&Point3::new(0, 1, -1)));
        assert!(conway1.cells.contains(&Point3::new(1, 3, 1)));
        assert!(conway1.cells.contains(&Point3::new(0, 1, 0)));
        assert!(conway1.cells.contains(&Point3::new(1, 2, 0)));

        let conway2 = conway1.evolve(|p| p.neighbors());
        assert_eq!(conway2.cells.len(), 21);
    }
}