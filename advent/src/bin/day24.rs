use std::io::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[macro_use]
extern crate lazy_static;
use regex::Regex;

// Consider a tiling by regular hexagons whose sides are 2 units in length, and such that the origin
// (0,0) is the center of one tile. Then each tile's center will be at (k * sqrt(3), m) where k,m are integers.
// (Not all such points are centers of a tile, of course.)

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
struct Tile {
    x: isize,
    y: isize
}

impl Tile {
    // Constructs the tile centered at (x*sqrt(3), y)
    fn new(x: isize, y: isize) -> Tile {
        Tile { x, y }
    }

    fn east(&self) -> Tile {
        Tile::new(self.x + 2, self.y)
    }

    fn west(&self) -> Tile {
        Tile::new(self.x - 2, self.y)
    }

    fn northeast(&self) -> Tile {
        Tile::new(self.x + 1, self.y + 3)
    }

    fn northwest(&self) -> Tile {
        Tile::new(self.x - 1, self.y + 3)
    }

    fn southeast(&self) -> Tile {
        Tile::new(self.x + 1, self.y - 3)
    }

    fn southwest(&self) -> Tile {
        Tile::new(self.x - 1, self.y - 3)
    }

    fn neighbors(&self) -> Vec<Tile> {
        vec!(
            self.east(), self.northeast(),
            self.northwest(), self.west(),
            self.southwest(), self.southeast()
        )
    }
}

// Start from the reference tile (0,0) and read directions
fn traverse(line: &str) -> Option<Tile> {
    lazy_static! {
        static ref EAST_PAT: Regex = Regex::new(r"^e([news]*)$").unwrap();
        static ref WEST_PAT: Regex = Regex::new(r"^w([news]*)$").unwrap();
        static ref NORTHEAST_PAT: Regex = Regex::new(r"^ne([news]*)$").unwrap();
        static ref NORTHWEST_PAT: Regex = Regex::new(r"^nw([news]*)$").unwrap();
        static ref SOUTHWEST_PAT: Regex = Regex::new(r"^sw([news]*)$").unwrap();
        static ref SOUTHEAST_PAT: Regex = Regex::new(r"^se([news]*)$").unwrap();
    }

    let mut tail: &str = line;
    let mut tile = Tile::new(0,0);

    while !tail.is_empty() {
        
        if let Some(caps) = NORTHEAST_PAT.captures(tail) {
            tile = tile.northeast();
            for m in caps.get(1) {
                tail = m.as_str();
            }
        } else if let Some(caps) = NORTHWEST_PAT.captures(tail) {
            tile = tile.northwest();
            for m in caps.get(1) {
                tail = m.as_str();
            }
        } else if let Some(caps) = SOUTHEAST_PAT.captures(tail) {
            tile = tile.southeast();
            for m in caps.get(1) {
                tail = m.as_str();
            }
        } else if let Some(caps) = SOUTHWEST_PAT.captures(tail) {
            tile = tile.southwest();
            for m in caps.get(1) {
                tail = m.as_str()
            }
        } else if let Some(caps) = EAST_PAT.captures(tail) {
            tile = tile.east();
            for m in caps.get(1) {
                tail = m.as_str();
            }
        } else if let Some(caps) = WEST_PAT.captures(tail) {
            tile = tile.west();
            for m in caps.get(1) {
                tail= m.as_str();
            }
            // TODO: Can this by DRYed out?
        } else {
            eprintln!("Could not match text {}", tail);
            return None
        }
    }

    return Some(tile);
}

fn collect_keys<K: Copy + Ord, V, F>(m: &BTreeMap<K, V>, f: F) -> BTreeSet<K>
    where F: Fn(&K, &V) -> bool {

    m.iter().flat_map(|(k,v)| {
        Some(*k).filter(|_| f(k, v))
    }).collect()
}

fn evolve(black_tiles: &BTreeSet<Tile>) -> BTreeSet<Tile> {
    let mut visited: BTreeMap<Tile, bool> = BTreeMap::new();

    for tile in black_tiles {
        for neighbor in tile.neighbors() { // decide if `neighbor` should be black or white in the next iteration
            visited.entry(neighbor).or_insert({
                let mut borders = 0;
                for n2 in neighbor.neighbors() {
                    if black_tiles.contains(&n2) {
                        borders += 1;
                    }
                }
                if black_tiles.contains(&neighbor) {
                    borders == 1 || borders == 2
                } else {
                    borders == 2
                }
            });
        }
    }

    collect_keys(&visited, |_,v| *v)
}

fn main() {
    let stdin = std::io::stdin();
    let mut tiles: BTreeMap<Tile, usize> = BTreeMap::new();

    for line in stdin.lock().lines().flatten() {
        let tile = traverse(&line).unwrap();
        *tiles.entry(tile).or_insert(0) += 1;
    }

    println!("{} distinct tiles parsed.", tiles.len());

    let black_tiles = collect_keys(&tiles, |_,v| v % 2 == 1);

    println!("{} tiles are black on day 0", black_tiles.len());

    let black_tiles_100 = (0..100).fold(black_tiles, |acc, _| {
        evolve(&acc)
    });
    println!("After 100 evolutions, {} tiles are black.", black_tiles_100.len());
}

#[cfg(test)]
mod day24_spec {
    use super::*;

    #[test]
    fn traverse_test() {
        let line = "esenee";
        let tile = traverse(line).unwrap();
        assert_eq!(tile, Tile::new(6, 0));

        let line = "nwwswee";
        let tile = traverse(line).unwrap();
        assert_eq!(tile, Tile::new(0, 0));
    }

    #[test]
    fn evolve_test() {
        let lines = vec!(
            "sesenwnenenewseeswwswswwnenewsewsw",
            "neeenesenwnwwswnenewnwwsewnenwseswesw",
            "seswneswswsenwwnwse",
            "nwnwneseeswswnenewneswwnewseswneseene",
            "swweswneswnenwsewnwneneseenw",
            "eesenwseswswnenwswnwnwsewwnwsene",
            "sewnenenenesenwsewnenwwwse",
            "wenwwweseeeweswwwnwwe",
            "wsweesenenewnwwnwsenewsenwwsesesenwne",
            "neeswseenwwswnwswswnw",
            "nenwswwsewswnenenewsenwsenwnesesenew",
            "enewnwewneswsewnwswenweswnenwsenwsw",
            "sweneswneswneneenwnewenewwneswswnese",
            "swwesenesewenwneswnwwneseswwne",
            "enesenwswwswneneswsenwnewswseenwsese",
            "wnwnesenesenenwwnenwsewesewsesesew",
            "nenewswnwewswnenesenwnesewesw",
            "eneswnwswnwsenenwnwnwwseeswneewsenese",
            "neswnwewnwnwseenwseesewsenwsweewe",
            "wseweeenwnesenwwwswnew"
        );
        let black_tiles_0: BTreeSet<Tile> = lines.iter().fold(BTreeSet::new(), |mut acc, line| {
            let tile = traverse(line).unwrap();
            if acc.contains(&tile) {
                acc.remove(&tile);
            } else {
                acc.insert(tile);
            }
            acc
        });

        assert_eq!(black_tiles_0.len(), 10);

        let black_tiles_1 = evolve(&black_tiles_0);
        assert_eq!(black_tiles_1.len(), 15);

        let black_tiles_2 = evolve(&black_tiles_1);
        assert_eq!(black_tiles_2.len(), 12);

        let black_tiles_100 = (0..100).fold(black_tiles_0, |acc, _| {
            evolve(&acc)
        });
        assert_eq!(black_tiles_100.len(), 2208);
    }
}