use std::io::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

#[macro_use]
extern crate lazy_static;
use regex::Regex;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum D4 { // the symmetry group of the square: FIRST, flip the square across the vertical axis if true, THEN rotate CCW
    R0(bool),
    R1(bool),
    R2(bool),
    R3(bool)
}

impl D4 {
    fn items() -> Vec<D4> {
        vec!(D4::R0(false), D4::R1(false), D4::R2(false), D4::R3(false), D4::R0(true), D4::R1(true), D4::R2(true), D4::R3(true))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Edge {
    Top,
    Bottom,
    Right,
    Left
}

impl Edge {
    fn items() -> Vec<Edge> {
        vec!(Edge::Top, Edge::Bottom, Edge::Left, Edge::Right)
    }
}

const BOTTOM_EDGE_MASK: u128 = 0x3ff;
const TOP_EDGE_MASK: u128 = reverse_100(BOTTOM_EDGE_MASK);
const RIGHT_EDGE_MASK: u128 = 0x040100401004010040100401; // bits: 90, 80, ..., 10, 0
const LEFT_EDGE_MASK: u128 = reverse_100(RIGHT_EDGE_MASK); // bits: 99, 89, ..., 19, 9

const BOTTOM_PIXEL_ROW_MASK: u128 = 0xff << 11; // bits: 18, 17, ..., 11
const TOP_PIXEL_ROW_MASK: u128 = BOTTOM_PIXEL_ROW_MASK << 70; // bits: 88, 87, ..., 81
const RIGHT_PIXEL_COLUMN_MASK: u128 = 0x200802008020080200800; // bits: 81, 71, ..., 11
const LEFT_PIXEL_COLUMN_MASK: u128 = RIGHT_PIXEL_COLUMN_MASK << 7; // bits: 88, 78, ..., 18

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Tile(u128, u16);

impl Tile {
    fn new(pixels: &str, id: u16) -> Option<Tile> {
        let mut p = 0;
        for pixel in pixels.chars() {
            match pixel {
                '.' => p <<= 1,
                '#' => {
                    p <<= 1;
                    p += 1;
                },
                _ => return None
            }
        }

        Some(Tile(p, id))
    }

    fn id(&self) -> u16 {
        self.1
    }

    fn read_edge(&self, d4: D4, edge: Edge) -> u16 {
        match (d4, edge) {
            (D4::R0(false), Edge::Bottom) | (D4::R3(false), Edge::Left) | (D4::R2(true), Edge::Top) | (D4::R1(true), Edge::Right) => {
                extract_mask(self.0, BOTTOM_EDGE_MASK) as u16
            },
            (D4::R0(true), Edge::Bottom) | (D4::R3(true), Edge::Left) | (D4::R2(false), Edge::Top) | (D4::R1(false), Edge::Right) => {
                reverse_10(extract_mask(self.0, BOTTOM_EDGE_MASK) as u16)
            },
            (D4::R0(false), Edge::Top) | (D4::R3(false), Edge::Right) | (D4::R2(true), Edge::Bottom) | (D4::R1(true), Edge::Left) => {
                extract_mask(self.0, TOP_EDGE_MASK) as u16
            },
            (D4::R0(true), Edge::Top) | (D4::R3(true), Edge::Right) | (D4::R2(false), Edge::Bottom) | (D4::R1(false), Edge::Left) => {
                reverse_10(extract_mask(self.0, TOP_EDGE_MASK) as u16)
            },
            (D4::R0(false), Edge::Left) | (D4::R1(false), Edge::Bottom) | (D4::R0(true), Edge::Right) | (D4::R1(true), Edge::Top) => {
                extract_mask(self.0, LEFT_EDGE_MASK) as u16
            },
            (D4::R2(false), Edge::Right) | (D4::R2(true), Edge::Left) | (D4::R3(false), Edge::Top) | (D4::R3(true), Edge::Bottom) => {
                reverse_10(extract_mask(self.0, LEFT_EDGE_MASK) as u16)
            },
            (D4::R0(false), Edge::Right) | (D4::R0(true), Edge::Left) | (D4::R1(false), Edge::Top) | (D4::R1(true), Edge::Bottom) => {
                extract_mask(self.0, RIGHT_EDGE_MASK) as u16
            },
            (D4::R2(false), Edge::Left) | (D4::R2(true), Edge::Right) | (D4::R3(false), Edge::Bottom) | (D4::R3(true), Edge::Top) => {
                reverse_10(extract_mask(self.0, RIGHT_EDGE_MASK) as u16)
            }
        }
    }

    // Always returns a vector of length 8; the bytes of the interior of this tile in the requested orientation
    // Each byte is a row (read left to right)
    fn read_pixels(&self, d4: D4) -> Vec<u8> {
        match d4 {
            D4::R0(false) => {
                (0..8).map(|shift| {
                    extract_mask(self.0, TOP_PIXEL_ROW_MASK >> (10 * shift)) as u8
                }).collect()
            },
            D4::R0(true) => {
                (0..8).map(|shift| {
                    (extract_mask(self.0, TOP_PIXEL_ROW_MASK >> (10 * shift)) as u8).reverse_bits()
                }).collect()
            }
            D4::R2(true) => {
                (0..8).map(|shift| {
                    extract_mask(self.0, BOTTOM_PIXEL_ROW_MASK << (10 * shift)) as u8
                }).collect()
            },
            D4::R2(false) => {
                (0..8).map(|shift| {
                    (extract_mask(self.0, BOTTOM_PIXEL_ROW_MASK << (10 * shift)) as u8).reverse_bits()
                }).collect()
            },
            D4::R1(false) => {
                (0..8).map(|shift| {
                    extract_mask(self.0, RIGHT_PIXEL_COLUMN_MASK << shift) as u8
                }).collect()
            },
            D4::R3(true) => {
                (0..8).map(|shift| {
                    (extract_mask(self.0, RIGHT_PIXEL_COLUMN_MASK << shift) as u8).reverse_bits()
                }).collect()
            },
            D4::R1(true) => {
                (0..8).map(|shift| {
                    extract_mask(self.0, LEFT_PIXEL_COLUMN_MASK >> shift) as u8
                }).collect()
            },
            D4::R3(false) => {
                (0..8).map(|shift| {
                    (extract_mask(self.0, LEFT_PIXEL_COLUMN_MASK >> shift) as u8).reverse_bits()
                }).collect()
            }
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let top_bit: u128 = 1 << 127;
        let mut u: u128 = self.0 << 28;
        let mut r = String::new();

        for i in 0..100 {
            if i > 0 && (i % 10 == 0) {
                r.push('\n');
            }
            if (u & top_bit) > 0 {
                r.push('#');
            } else {
                r.push('.');
            }
            u <<= 1;
        }

        write!(f, "{}", r)
    }
}

const fn extract_mask(x: u128, mask: u128) -> u128 {
    let mut mask = mask;
    let mut r = 0;

    while mask.count_ones() > 0 {
        let mask_bit_idx = 127 - mask.leading_zeros();
        let mask_bit = 1 << mask_bit_idx;
        let x_bit = x & mask_bit;
        r |= x_bit >> (mask_bit_idx + 1 - mask.count_ones());
        mask &= !mask_bit;
    }

    r
}

// reverse the last 100 bits of x
const fn reverse_100(x: u128) -> u128 {
    x.reverse_bits() >> 28
}

const fn reverse_10(x: u16) -> u16 {
    x.reverse_bits() >> 6
}

fn group_by_edge(tiles: &BTreeSet<Tile>) -> BTreeMap<u16, BTreeSet<(&Tile, D4, Edge)>> {
    let mut r = BTreeMap::new();

    for tile in tiles {
        for d4 in D4::items() {
            for edge in Edge::items() {
                let e = tile.read_edge(d4, edge);
                r.entry(e).or_insert(BTreeSet::new()).insert((tile, d4, edge));
            }
        }
    }
    
    r
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pixel {
    On, Off
}

impl Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let c = match self {
            Pixel::Off => '.',
            Pixel::On => '#'
        };
        write!(f, "{}", c)
    }
}

struct Image {
    rows: Vec<Vec<Pixel>>
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.rows.iter().fold(Ok(()), |r, row| {
            r.and_then(|_| {
                let mut line = String::new();
                for pixel in row {
                    line.push_str(&pixel.to_string());
                }
                write!(f, "{}\n", line)
            })
        })
    }
}

impl Image {
    fn new(tiles: &Vec<Vec<(&Tile, D4)>>) -> Image {
        let mut rows = vec!();

        for tile_row in tiles {
            let mut current_pixel_rows: Vec<Vec<Pixel>> = vec!(vec!(); 8);
            for (tile, d4) in tile_row {
                for (idx, byte) in tile.read_pixels(*d4).iter().enumerate() {
                    let mut mask = 0x80;
                    while mask > 0 {
                        if byte & mask > 0 {
                            current_pixel_rows[idx].push(Pixel::On);
                        } else {
                            current_pixel_rows[idx].push(Pixel::Off);
                        }
                        mask >>= 1;
                    }
                    
                }
            }

            rows.append(&mut current_pixel_rows);
        }

        Image { rows }
    }

    // is the pixel at the specified coordinates on or off?
    fn is_on(&self, x: usize, y: usize) -> bool {
        self.rows.get(y).and_then(|row| row.get(x)).map_or(false, |pixel| *pixel == Pixel::On)
    }

    fn rotate(&self) -> Image {
        let mut rows: Vec<Vec<Pixel>> = vec!(vec!(); self.rows.len());

        for row in &self.rows {
            for (y, pixel) in row.iter().rev().enumerate() {
                rows[y].push(*pixel);
            }
        }

        Image { rows }
    }

    fn flip(&self) -> Image {
        // flips across the 1st-quadrant diagonal because that's easier
        let mut flipped_rows: Vec<Vec<Pixel>> = vec!(vec!(Pixel::Off; self.rows.len()); self.rows.len());

        for (y, row) in self.rows.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                flipped_rows[x][y] = *pixel;
            }
        }


        Image { rows: flipped_rows }
    }

    // returns a dict of sea monsters, keyed by their tail point
    fn sea_monsters(&self) -> BTreeMap<(usize, usize), BTreeSet<(usize, usize)>> {
        let mut r = BTreeMap::new();

        for y in 1..(self.rows.len() - 1) {
            // A sea monster is a subset of a 3x20 window of pixels
            'x: for x in 0..(self.rows.len() - 20) {
                let monster: BTreeSet<(usize, usize)> = vec!(
                    (x    ,  y),
                    (x + 1,  y + 1),
                    (x + 4,  y + 1),
                    (x + 5,  y),
                    (x + 6,  y),
                    (x + 7,  y + 1),
                    (x + 10, y + 1),
                    (x + 11, y),
                    (x + 12, y),
                    (x + 13, y + 1),
                    (x + 16, y + 1),
                    (x + 17, y),
                    (x + 18, y),
                    (x + 18, y - 1),
                    (x + 19, y)
                ).into_iter().collect();
                for (mx, my) in &monster {
                    if !self.is_on(*mx, *my) {
                        continue 'x
                    }
                }
                r.insert((x,y), monster);
            }
        }

        r
    }
}

fn assemble_greedy(tiles: &BTreeMap<u16, BTreeSet<(&Tile, D4, Edge)>>) -> Result<Image, String> {
    // We must have a perfect square of tiles
    let mut available_tiles: BTreeSet<&Tile> = tiles.values().flat_map(|ps| ps.iter().map(|p| p.0)).collect();
    let side_length: usize = (available_tiles.len() as f32).sqrt() as usize;
    if side_length * side_length != available_tiles.len() {
        let msg = format!("Tile-set has {} entries, which is not a perfect square.", available_tiles.len());
        return Err(msg)
    }

    let mut tile_matrix: Vec<Vec<(&Tile, D4)>> = {        
        // seed with upper-left tile
        let upper_left_tile: (&Tile, D4) = {
            let mut lefts: BTreeSet<(&Tile, D4)> = BTreeSet::new();
            let mut uppers: BTreeSet<(&Tile, D4)> = BTreeSet::new();
    
            let mut ult: Result<(&Tile, D4), String> = Err("No corner tile found".to_owned());
    
            'a: for (_, ts) in tiles {
                let tile_ids: BTreeSet<u16> = ts.iter().map(|p| p.0.id()).collect();
                if tile_ids.len() == 1 {
                    for (tile, d4, edge) in ts {
                        match edge {
                            Edge::Left if uppers.contains(&(*tile, *d4)) => {
                                ult = Ok((*tile, *d4)); break 'a
                            },
                            Edge::Left => {
                                lefts.insert((*tile, *d4));
                            },
                            Edge::Top if lefts.contains(&(*tile, *d4)) => {
                                ult = Ok((*tile, *d4)); break 'a
                            },
                            Edge::Top => {
                                uppers.insert((*tile, *d4));
                            },
                            _ => ()
                        }
                    }
                }
            }
    
            match ult { // Oh for a for..else construct...
                Err(msg) => return Err(msg),
                Ok(t) => t
            }
        };

        available_tiles.remove(upper_left_tile.0);
        vec!(vec!(upper_left_tile))
    };

    loop {
        if tile_matrix.len() == side_length {
            if let Some(last_row) = tile_matrix.last() {
                if last_row.len() == side_length {
                    break
                }
            }
        }
        match tile_matrix.last_mut() {
            None => return Err("Unreachable error; tile_matrix is always non-empty".to_owned()),
            Some(last_row) => {
                while last_row.len() < side_length {
                    match last_row.last() {
                        None => return Err("Unreachable error; last_row is always non-empty".to_owned()),
                        Some((tile, d4)) => {
                            let right_border = tile.read_edge(*d4, Edge::Right);
                            let opt_next_tile = tiles.get(&right_border).and_then(|candidates| {
                                let mut j = candidates.iter().flat_map(|p| {
                                    if let (tile, d4, Edge::Left) = p {
                                        Some((*tile, *d4)).filter(|q| available_tiles.contains(q.0))
                                    } else {
                                        None
                                    }
                                });
                                j.next()
                            });
                            match opt_next_tile {
                                Some((tile, d4)) => {
                                    last_row.push((tile, d4));
                                    available_tiles.remove(&tile);
                                },
                                None => return Err("No suitable continuation tile found.".to_owned())
                            }
                        }
                    }
                };
                let (upper_tile, upper_d4) = last_row[0];
                let lower_border = upper_tile.read_edge(upper_d4, Edge::Bottom);
                let opt_next_tile = tiles.get(&lower_border).and_then(|candidates| {
                    let mut j = candidates.iter().flat_map(|p| {
                        if let (tile, d4, Edge::Top) = p {
                            Some((*tile, *d4)).filter(|q| available_tiles.contains(q.0))
                        } else {
                            None
                        }
                    });
                    j.next()
                });
                match opt_next_tile {
                    Some(t) => {
                        tile_matrix.push(vec!(t));
                        available_tiles.remove(t.0);
                    },
                    None if tile_matrix.len() == side_length => break,
                    None => return Err("No suitable tile available to start next row!".to_owned())
                };
            }
        }
    }

    let image = Image::new(&tile_matrix);

    Ok(image)
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdin_lines = stdin.lock().lines().flatten();

    lazy_static! {
        static ref TILE_HEADER: Regex = Regex::new(r"Tile (\d+):").unwrap();
    }

    let tiles: BTreeSet<Tile> = {
        let mut tiles = BTreeSet::new();

        enum LoopState {
            Begin,
            // We flatten the pixels into a single line
            Partial(u16, String)
        }

        let mut loop_state = LoopState::Begin;

        while let Some(line) = stdin_lines.next() {
            if let Some(caps) = TILE_HEADER.captures(&line) {
                let tile_id = u16::from_str_radix(&caps[1], 10).unwrap();
                loop_state = LoopState::Partial(tile_id, String::new());
            } else if line.is_empty() {
                if let LoopState::Partial(tile_id, pixels) = &loop_state {
                    if let Some(tile) = Tile::new(pixels, *tile_id) {
                        tiles.insert(tile);
                    } else {
                        eprintln!("Could not parse tile from pixels {}", pixels);
                    }
                } else {
                    eprintln!("Wrong state!")
                }
            } else if let LoopState::Partial(_, pixels) = &mut loop_state {
                pixels.push_str(&line);
            }
        }

        tiles
    };

    println!("Parsed {} tiles", tiles.len());

    let tiles_by_edge: BTreeMap<u16, BTreeSet<(&Tile, D4, Edge)>> = group_by_edge(&tiles);
    let mut border_tiles: BTreeMap<u16, u8> = BTreeMap::new();
    for (_e, ts) in &tiles_by_edge {
        let tile_ids: BTreeSet<u16> = ts.iter().map(|p| p.0.id()).collect();
        if tile_ids.len() == 1 {
            for tile_id in tile_ids {
                *border_tiles.entry(tile_id).or_insert(0) += 1;
            }
        }
    }
    let mut c: u128 = 1;
    for (tile_id, count) in border_tiles {
        if count > 2 {
            c *= tile_id as u128;
        }
    }
    println!("Product of corner tile ids: {}", c);

    let mut image = assemble_greedy(&tiles_by_edge).unwrap();

    println!("{}", image);

    for i in 0..8 {
        let sea_monsters = image.sea_monsters();

        if sea_monsters.len() > 0 {
            println!("{} sea monsters found!", sea_monsters.len());
            let sea_monster_coordinates: BTreeSet<(usize, usize)> = sea_monsters.values().fold(BTreeSet::new(), |mut acc, m| {
                for (x,y) in m {
                    acc.insert((*x, *y));
                }                
                acc
            });

            let mut t = 0;

            for (y, row) in image.rows.iter().enumerate() {
                for (x, pixel) in row.iter().enumerate() {
                    if let Pixel::On = pixel {
                        if !sea_monster_coordinates.contains(&(x,y)) {
                            t += 1;
                        }
                    }
                }
            }

            println!("The image contains {} sea monster pixels and {} rough-water pixels.", sea_monster_coordinates.len(), t);
        }
        if i == 3 {
            image = image.flip();
        } else {
            image = image.rotate();
        }
    }

}

#[cfg(test)]
mod day20_spec {
    use super::*;

    #[test]
    fn extract_mask_test() {
        let x = 0b10101;
        let mask = 0b01110;
        assert_eq!(extract_mask(x, mask), 2);

        let x = 0x10;
        assert_eq!(extract_mask(x, 0x20), 0);
        assert_eq!(extract_mask(x, 0x10), 1);
        assert_eq!(extract_mask(x, 0x18), 2);
        assert_eq!(extract_mask(x, 0x14), 2);
        assert_eq!(extract_mask(x, 0x1c), 4);
        assert_eq!(extract_mask(x, 0x12), 2);
        assert_eq!(extract_mask(x, 0x1e), 8);
        assert_eq!(extract_mask(x, 0x11), 2);
        assert_eq!(extract_mask(x, 0x1f), 16);
    }

    #[test]
    fn reverse_100_test() {
        assert_eq!(reverse_100(1), 1 << 99);
        assert_eq!(reverse_100(2), 1 << 98);
        assert_eq!(reverse_100(0b1101), 0b1011 << 96);
    }

    #[test]
    fn read_edge_test() {
        let pixels = "..##.#..#.\
                      ##..#.....\
                      #...##..#.\
                      ####.#...#\
                      ##.##.###.\
                      ##...#.###\
                      .#.#.#..##\
                      ..#....#..\
                      ###...#.#.\
                      ..###..###";
        let tile2311 = Tile::new(pixels, 2311).unwrap();
        // identity transformation
        let d4 = D4::R0(false);
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b0011010010);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b0111110010);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b0011100111);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b0001011001);
        
        // reflect across vertical axis
        let d4 = D4::R0(true);
        // .#..#.##..
        // .....#..##
        // .#..##...#
        // #...#.####
        // .###.##.##
        // ###.#...##
        // ##..#.#.#.
        // ..#....#..
        // .#.#...###
        // ###..###..
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b0100101100);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b0001011001);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b1110011100);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b0111110010);

        // 1/4 turn
        let d4 = D4::R1(false);
        // ...#.##..#
        // #.#.###.##
        // ....##.#.#
        // ....#...#.
        // #.##.##...
        // .##.#....#
        // #..##.#..#
        // #..#...###
        // .#.####.#.
        // .#####..#.
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b0001011001);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b0111110010);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b0100101100);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b1110011100);

        let d4 = D4::R1(true);
        // .#####..#.
        // .#.####.#.
        // #..#...###
        // #..##.#..#
        // .##.#....#
        // #.##.##...
        // ....#...#.
        // ....##.#.#
        // #.#.###.##
        // ...#.##..#
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b0111110010);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b0001011001);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b0011010010);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b0011100111);

        let d4 = D4::R2(false);
        // ###..###..
        // .#.#...###
        // ..#....#..
        // ##..#.#.#.
        // ###.#...##
        // .###.##.##
        // #...#.####
        // .#..##...#
        // .....#..##
        // .#..#.##..
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b1110011100);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b0100101100);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b1001101000);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b0100111110);

        let d4 = D4::R2(true);
        // ..###..###
        // ###...#.#.
        // ..#....#..
        // .#.#.#..##
        // ##...#.###
        // ##.##.###.
        // ####.#...#
        // #...##..#.
        // ##..#.....
        // ..##.#..#.        
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b0011100111);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b0011010010);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b0100111110);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b1001101000);

        let d4 = D4::R3(false);
        // .#..#####.
        // .#.####.#.
        // ###...#..#
        // #..#.##..#
        // #....#.##.
        // ...##.##.#
        // .#...#....
        // #.#.##....
        // ##.###.#.#
        // #..##.#...
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b0100111110);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b1001101000);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b0011100111);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b0011010010);

        let d4 = D4::R3(true);
        // #..##.#...
        // ##.###.#.#
        // #.#.##....
        // .#...#....
        // ...##.##.#
        // #....#.##.
        // #..#.##..#
        // ###...#..#
        // .#.####.#.
        // .#..#####.
        assert_eq!(tile2311.read_edge(d4, Edge::Top),    0b1001101000);
        assert_eq!(tile2311.read_edge(d4, Edge::Bottom), 0b0100111110);
        assert_eq!(tile2311.read_edge(d4, Edge::Left),   0b1110011100);
        assert_eq!(tile2311.read_edge(d4, Edge::Right),  0b0100101100);
    }

    #[test]
    fn read_pixels_test() {
        let pixels = "..##.#..#.\
                      ##..#.....\
                      #...##..#.\
                      ####.#...#\
                      ##.##.###.\
                      ##...#.###\
                      .#.#.#..##\
                      ..#....#..\
                      ###...#.#.\
                      ..###..###";
        let tile2311 = Tile::new(pixels, 2311).unwrap();
        assert_eq!(tile2311.read_pixels(D4::R0(false)), vec!(
            0x90, 0x19, 0xe8, 0xb7, 0x8b, 0xa9, 0x42, 0xc5
        ));

        assert_eq!(tile2311.read_pixels(D4::R2(true)), vec!(
            0xc5, 0x42, 0xa9, 0x8b, 0xb7, 0xe8, 0x19, 0x90
        ));

        assert_eq!(tile2311.read_pixels(D4::R1(false)), vec!(
            0x5d, 0x1a, 0x11, 0x6c, 0xd0, 0x34, 0x23, 0xbd
        ));

        assert_eq!(tile2311.read_pixels(D4::R1(true)), vec!(
            0xbd, 0x23, 0x34, 0xd0, 0x6c, 0x11, 0x1a, 0x5d
        ));

        assert_eq!(tile2311.read_pixels(D4::R2(false)), vec!(
            0xa3, 0x42, 0x95, 0xd1, 0xed, 0x17, 0x98, 0x09
        ));    

        assert_eq!(tile2311.read_pixels(D4::R0(true)), vec!(
            0x09, 0x98, 0x17, 0xed, 0xd1, 0x95, 0x42, 0xa3
        ));    

        assert_eq!(tile2311.read_pixels(D4::R3(false)), vec!(
            0xbd, 0xc4, 0x2c, 0x0b, 0x36, 0x88, 0x58, 0xba 
        ));        

        assert_eq!(tile2311.read_pixels(D4::R3(true)), vec!(
            0xba, 0x58, 0x88, 0x36, 0x0b, 0x2c, 0xc4, 0xbd
        ));
    }

    #[test]
    fn sea_monsters_test() {
        let pixels = "..##.#..#.\
                      ##..#.....\
                      #...##..#.\
                      ####.#...#\
                      ##.##.###.\
                      ##...#.###\
                      .#.#.#..##\
                      ..#....#..\
                      ###...#.#.\
                      ..###..###";
        let tile2311 = Tile::new(pixels, 2311).unwrap();

        let pixels = "#.##...##.\
                      #.####...#\
                      .....#..##\
                      #...######\
                      .##.#....#\
                      .###.#####\
                      ###.##.##.\
                      .###....#.\
                      ..#.#..#.#\
                      #...##.#..";
        let tile1951 = Tile::new(pixels, 1951).unwrap();

        let pixels = "####...##.\
                      #..##.#..#\
                      ##.#..#.#.\
                      .###.####.\
                      ..###.####\
                      .##....##.\
                      .#...####.\
                      #.##.####.\
                      ####..#...\
                      .....##...";
        let tile1171 = Tile::new(pixels, 1171).unwrap();

        let pixels = "###.##.#..\
                      .#..#.##..\
                      .#.##.#..#\
                      #.#.#.##.#\
                      ....#...##\
                      ...##..##.\
                      ...#.#####\
                      .#.####.#.\
                      ..#..###.#\
                      ..##.#..#.";
        let tile1427 = Tile::new(pixels, 1427).unwrap();

        let pixels = "##.#.#....\
                      ..##...#..\
                      .##..##...\
                      ..#...#...\
                      #####...#.\
                      #..#.#.#.#\
                      ...#.#.#..\
                      ##.#...##.\
                      ..##.##.##\
                      ###.##.#..";
        let tile1489 = Tile::new(pixels, 1489).unwrap();

        let pixels = "#....####.\
                      #..#.##...\
                      #.##..#...\
                      ######.#.#\
                      .#...#.#.#\
                      .#########\
                      .###.#..#.\
                      ########.#\
                      ##...##.#.\
                      ..###.#.#.";
        let tile2473 = Tile::new(pixels, 2473).unwrap();

        let pixels = "..#.#....#\
                      #...###...\
                      #.#.###...\
                      ##.##..#..\
                      .#####..##\
                      .#..####.#\
                      #..#.#..#.\
                      ..####.###\
                      ..#.#.###.\
                      ...#.#.#.#";
        let tile2971 = Tile::new(pixels, 2971).unwrap();

        let pixels = "...#.#.#.#\
                      ####.#....\
                      ..#.#.....\
                      ....#..#.#\
                      .##..##.#.\
                      .#.####...\
                      ####.#.#..\
                      ##.####...\
                      ##..#.##..\
                      #.##...##.";
        let tile2729 = Tile::new(pixels, 2729).unwrap();

        let pixels = "#.#.#####.\
                      .#..######\
                      ..#.......\
                      ######....\
                      ####.#..#.\
                      .#...#.##.\
                      #.#####.##\
                      ..#.###...\
                      ..#.......\
                      ..#.###...";
        let tile3079 = Tile::new(pixels, 3079).unwrap();

        let orientations: Vec<Vec<(&Tile, D4)>> = vec!(
            vec!((&tile1951, D4::R2(true)), (&tile2311, D4::R2(true)), (&tile3079, D4::R0(false))),
            vec!((&tile2729, D4::R2(true)), (&tile1427, D4::R2(true)), (&tile2473, D4::R3(true))),
            vec!((&tile2971, D4::R2(true)), (&tile1489, D4::R2(true)), (&tile1171, D4::R0(true)))
        );
        let image = Image::new(&orientations);

        assert_eq!(image.sea_monsters().len(), 0);

        let orientations2: Vec<Vec<(&Tile, D4)>> = vec!(
            vec!((&tile1951, D4::R3(false)), (&tile2729, D4::R3(false)), (&tile2971, D4::R3(false))),
            vec!((&tile2311, D4::R3(false)), (&tile1427, D4::R3(false)), (&tile1489, D4::R3(false))),
            vec!((&tile3079, D4::R1(true)),  (&tile2473, D4::R2(false)), (&tile1171, D4::R1(false)))
        );
        let image2 = Image::new(&orientations2);

        assert_eq!(image2.sea_monsters().len(), 2);

        let image3 = image.flip();

        assert_eq!(image3.sea_monsters().len(), 2);

        let image4 = image.rotate();

        assert_eq!(image4.sea_monsters().len(), 0);
    }
}