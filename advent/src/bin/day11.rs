use std::io::prelude::*;

use itertools::Itertools;

#[macro_use]
extern crate lazy_static;


#[derive(Debug, PartialEq, Eq, Clone)]
enum SeatState {
    Floor,
    Empty,
    Occupied
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct SeatingDiagram {
    width: usize,
    seats: Vec<Vec<SeatState>>
}

impl SeatingDiagram {
    fn build<J>(j: &mut J) -> Option<SeatingDiagram>
    where J: Iterator<Item=String> {

        fn parse_line(line: &str) -> Vec<SeatState> {
            line.chars().flat_map(|c| {
                match c {
                    '.' => Some(SeatState::Floor),
                    'L' => Some(SeatState::Empty),
                    '#' => Some(SeatState::Occupied),
                    _ => None
                }
            }).collect()
        }

        let mut diagram: SeatingDiagram;
        if let Some(line) = j.next() {
            diagram = SeatingDiagram { width: line.len(), seats: vec!(parse_line(&line))};
        } else {
            return None
        }

        while let Some(line) = j.next() {
            diagram.seats.push(parse_line(&line));
        }

        Some(diagram)
    }

    fn count_occupied_seats(&self) -> usize {
        let mut r = 0;

        for row in &self.seats {
            for seat in row {
                r += (*seat == SeatState::Occupied) as usize;
            }
        }

        r
    }

    fn count_adjacent_occupied_seats(&self, row: usize, column: usize) -> u8 {
        let mut ret = 0;

        fn checked_add(x: usize, dx: i8) -> Option<usize> {
            if dx < 0 {
                x.checked_sub(-dx as usize)
            } else {
                x.checked_add(dx as usize)
            }
        }

        for dr in -1..=1 {
        for dc in -1..=1 {
            if (dr, dc) != (0, 0) {
                let opt_r1 = checked_add(row, dr).filter(|&r1| r1 < self.seats.len());
                let opt_c1 = checked_add(column, dc).filter(|&c1| c1 < self.width);
                for r1 in opt_r1 {
                for c1 in opt_c1 {
                    if let SeatState::Occupied = self.seats[r1][c1] {
                        ret += 1;
                    }
                }}
            }
        }}

        ret
    }

    fn step(&mut self) -> usize {
        let mut newly_occupied: Vec<(usize, usize)> = vec!();
        let mut newly_empty: Vec<(usize, usize)> = vec!();

        for row in 0..self.seats.len() {
        for col in 0..self.width {
            match self.seats[row][col] {
                SeatState::Empty => {
                    if self.count_adjacent_occupied_seats(row, col) == 0 {
                        newly_occupied.push((row, col));
                    }
                },
                SeatState::Occupied => {
                    if self.count_adjacent_occupied_seats(row, col) >= 4 {
                        newly_empty.push((row, col))
                    }
                },
                _ => ()
            }
        }}

        for (row, col) in &newly_occupied {
            self.seats[*row][*col] = SeatState::Occupied;
        }

        for (row, col) in &newly_empty {
            self.seats[*row][*col] = SeatState::Empty;
        }

        newly_occupied.len() + newly_empty.len()
    }

    fn count_visible_occupied_seats(&self, row: usize, column: usize) -> u8 {
        lazy_static! {
            static ref DIRECTIONS: Vec<(i8, i8)> = (-1..=1).cartesian_product(-1..=1).filter(|p| *p != (0, 0)).collect();
        }

        fn step_direction(this: &SeatingDiagram, rc: (usize, usize), v: &(i8, i8)) -> Option<(usize, usize)> {
            let (row, col) = rc;
            let (dr, dc) = *v;
            let opt_r1 = if dr < 0 {
                row.checked_sub(-dr as usize)
            } else {
                row.checked_add(dr as usize)
            };
            let opt_c1 = if dc < 0 {
                col.checked_sub(-dc as usize)
            } else {
                col.checked_add(dc as usize)
            };
            opt_r1.and_then(|r1| opt_c1.map(|c1| (r1, c1)))
                .filter(|(r1, c1)| r1 < &this.seats.len() && c1 < &this.width)
        }

        let mut r = 0;

        'vector: for v in DIRECTIONS.iter() {
            let mut xy = step_direction(self, (row, column), v);
            while let Some((x,y)) = xy {
                match self.seats[x][y] {
                    SeatState::Occupied => {
                        r += 1;
                        continue 'vector
                    },
                    SeatState::Empty => {
                        continue 'vector
                    },
                    _ => ()
                };
                xy = step_direction(self, (x,y), v);
            }
        }

        r
    }

    fn step_visible(&mut self) -> usize {
        let mut newly_occupied: Vec<(usize, usize)> = vec!();
        let mut newly_empty: Vec<(usize, usize)> = vec!();

        for row in 0..self.seats.len() {
        for col in 0..self.width {
            match self.seats[row][col] {
                SeatState::Empty => {
                    if self.count_visible_occupied_seats(row, col) == 0 {
                        newly_occupied.push((row, col));
                    }
                },
                SeatState::Occupied => {
                    if self.count_visible_occupied_seats(row, col) >= 5 {
                        newly_empty.push((row, col))
                    }
                },
                _ => ()
            }
        }}

        for (row, col) in &newly_occupied {
            self.seats[*row][*col] = SeatState::Occupied;
        }

        for (row, col) in &newly_empty {
            self.seats[*row][*col] = SeatState::Empty;
        }

        newly_occupied.len() + newly_empty.len()
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut seating_diagram = SeatingDiagram::build(&mut stdin.lock().lines().flatten()).unwrap();

    println!("Parsed seating diagram of width {} and {} rows.", seating_diagram.width, seating_diagram.seats.len());

    let mut seating_diagram_part1 = seating_diagram.clone();
    while seating_diagram_part1.step() != 0 {};

    let occupied_count = seating_diagram_part1.count_occupied_seats();
    println!("Part 1: {} seats are occupied", occupied_count);

    while seating_diagram.step_visible() != 0 {};
    let occupied_count = seating_diagram.count_occupied_seats();
    println!("Part 2: {} seats are occupied", occupied_count);
}

#[cfg(test)]
mod day11_spec {
    use super::*;

    fn build_from_str(input: &str) -> Option<SeatingDiagram> {
        SeatingDiagram::build(&mut input.lines().map(|s| s.to_owned()))
    }

    #[test]
    fn build_test() {
        let input = "L.LL.LL.LL\n\
                     LLLLLLL.LL\n\
                     L.L.L..L..\n\
                     LLLL.LL.LL\n\
                     L.LL.LL.LL\n\
                     L.LLLLL.LL\n\
                     ..L.L.....\n\
                     LLLLLLLLLL\n\
                     L.LLLLLL.L\n\
                     L.LLLLL.LL";
        let seating_diagram = build_from_str(input).unwrap();
        assert_eq!(seating_diagram.width, 10);
        assert_eq!(seating_diagram.seats.len(), 10);
        assert_eq!(seating_diagram.seats[0][1], SeatState::Floor);
        assert_eq!(seating_diagram.seats[1][0], SeatState::Empty);
        assert_eq!(seating_diagram.seats.iter().fold(0, |acc, v| acc + v.len()), 100);

        let input = "#.##.##.##\n\
                     #######.##\n\
                     #.#.#..#..\n\
                     ####.##.##\n\
                     #.##.##.##\n\
                     #.#####.##\n\
                     ..#.#.....\n\
                     ##########\n\
                     #.######.#\n\
                     #.#####.##";
        let seating_diagram = build_from_str(input).unwrap();
        assert_eq!(seating_diagram.seats[0][1], SeatState::Floor);
        assert_eq!(seating_diagram.seats[1][0], SeatState::Occupied);
        assert_eq!(seating_diagram.seats[9][9], SeatState::Occupied);
    }

    #[test]
    fn step_test() {
        let pre = "L.LL.LL.LL\n\
                   LLLLLLL.LL\n\
                   L.L.L..L..\n\
                   LLLL.LL.LL\n\
                   L.LL.LL.LL\n\
                   L.LLLLL.LL\n\
                   ..L.L.....\n\
                   LLLLLLLLLL\n\
                   L.LLLLLL.L\n\
                   L.LLLLL.LL";
        let mut seating_diagram = build_from_str(pre).unwrap();

        let post1 = "#.##.##.##\n\
                     #######.##\n\
                     #.#.#..#..\n\
                     ####.##.##\n\
                     #.##.##.##\n\
                     #.#####.##\n\
                     ..#.#.....\n\
                     ##########\n\
                     #.######.#\n\
                     #.#####.##";
        let seating_diagram_post1 = build_from_str(post1).unwrap();
        assert_eq!(seating_diagram.step(), 71);
        assert_eq!(seating_diagram, seating_diagram_post1);

        let post2 = "#.LL.L#.##\n\
                     #LLLLLL.L#\n\
                     L.L.L..L..\n\
                     #LLL.LL.L#\n\
                     #.LL.LL.LL\n\
                     #.LLLL#.##\n\
                     ..L.L.....\n\
                     #LLLLLLLL#\n\
                     #.LLLLLL.L\n\
                     #.#LLLL.##";
        let seating_diagram_post2 = build_from_str(post2).unwrap();
        assert_eq!(seating_diagram.step(), 51);
        assert_eq!(seating_diagram, seating_diagram_post2);

        let post3 = "#.##.L#.##\n\
                     #L###LL.L#\n\
                     L.#.#..#..\n\
                     #L##.##.L#\n\
                     #.##.LL.LL\n\
                     #.###L#.##\n\
                     ..#.#.....\n\
                     #L######L#\n\
                     #.LL###L.L\n\
                     #.#L###.##";
        let seating_diagram_post3 = build_from_str(post3).unwrap();
        assert_eq!(seating_diagram.step(), 31);
        assert_eq!(seating_diagram, seating_diagram_post3);

        let post4 = "#.#L.L#.##\n\
                     #LLL#LL.L#\n\
                     L.L.L..#..\n\
                     #LLL.##.L#\n\
                     #.LL.LL.LL\n\
                     #.LL#L#.##\n\
                     ..L.L.....\n\
                     #L#LLLL#L#\n\
                     #.LLLLLL.L\n\
                     #.#L#L#.##";
        let seating_diagram_post4 = build_from_str(post4).unwrap();
        assert_eq!(seating_diagram.step(), 21);
        assert_eq!(seating_diagram, seating_diagram_post4);

        let post5 = "#.#L.L#.##\n\
                     #LLL#LL.L#\n\
                     L.#.L..#..\n\
                     #L##.##.L#\n\
                     #.#L.LL.LL\n\
                     #.#L#L#.##\n\
                     ..L.L.....\n\
                     #L#L##L#L#\n\
                     #.LLLLLL.L\n\
                     #.#L#L#.##";
        let seating_diagram_post5 = build_from_str(post5).unwrap();
        assert_eq!(seating_diagram.step(), 7);
        assert_eq!(seating_diagram, seating_diagram_post5);

        assert_eq!(seating_diagram.step(), 0);
    }

    #[test]
    fn step_visible_test() {
        let pre = "L.LL.LL.LL\n\
                   LLLLLLL.LL\n\
                   L.L.L..L..\n\
                   LLLL.LL.LL\n\
                   L.LL.LL.LL\n\
                   L.LLLLL.LL\n\
                   ..L.L.....\n\
                   LLLLLLLLLL\n\
                   L.LLLLLL.L\n\
                   L.LLLLL.LL";
        let mut seating_diagram = build_from_str(pre).unwrap();

        let post1 = "#.##.##.##\n\
                     #######.##\n\
                     #.#.#..#..\n\
                     ####.##.##\n\
                     #.##.##.##\n\
                     #.#####.##\n\
                     ..#.#.....\n\
                     ##########\n\
                     #.######.#\n\
                     #.#####.##";
        let seating_diagram_post1 = build_from_str(post1).unwrap();
        assert_eq!(seating_diagram.step_visible(), 71);
        assert_eq!(seating_diagram, seating_diagram_post1);

        let post2 = "#.LL.LL.L#\n\
                     #LLLLLL.LL\n\
                     L.L.L..L..\n\
                     LLLL.LL.LL\n\
                     L.LL.LL.LL\n\
                     L.LLLLL.LL\n\
                     ..L.L.....\n\
                     LLLLLLLLL#\n\
                     #.LLLLLL.L\n\
                     #.LLLLL.L#";
        let seating_diagram_post2 = build_from_str(post2).unwrap();
        assert_eq!(seating_diagram.step_visible(), 64);
        assert_eq!(seating_diagram, seating_diagram_post2);

        let post3 = "#.L#.##.L#\n\
                     #L#####.LL\n\
                     L.#.#..#..\n\
                     ##L#.##.##\n\
                     #.##.#L.##\n\
                     #.#####.#L\n\
                     ..#.#.....\n\
                     LLL####LL#\n\
                     #.L#####.L\n\
                     #.L####.L#";
        let seating_diagram_post3 = build_from_str(post3).unwrap();
        assert_eq!(seating_diagram.step_visible(), 46);
        assert_eq!(seating_diagram, seating_diagram_post3);

        let post4 = "#.L#.L#.L#\n\
                     #LLLLLL.LL\n\
                     L.L.L..#..\n\
                     ##LL.LL.L#\n\
                     L.LL.LL.L#\n\
                     #.LLLLL.LL\n\
                     ..L.L.....\n\
                     LLLLLLLLL#\n\
                     #.LLLLL#.L\n\
                     #.L#LL#.L#";
        let seating_diagram_post4 = build_from_str(post4).unwrap();
        assert_eq!(seating_diagram.step_visible(), 35);
        assert_eq!(seating_diagram, seating_diagram_post4);

        let post5 = "#.L#.L#.L#\n\
                     #LLLLLL.LL\n\
                     L.L.L..#..\n\
                     ##L#.#L.L#\n\
                     L.L#.#L.L#\n\
                     #.L####.LL\n\
                     ..#.#.....\n\
                     LLL###LLL#\n\
                     #.LLLLL#.L\n\
                     #.L#LL#.L#";
        let seating_diagram_post5 = build_from_str(post5).unwrap();
        assert_eq!(seating_diagram.step_visible(), 13);
        assert_eq!(seating_diagram, seating_diagram_post5);

        let post6 = "#.L#.L#.L#\n\
                     #LLLLLL.LL\n\
                     L.L.L..#..\n\
                     ##L#.#L.L#\n\
                     L.L#.LL.L#\n\
                     #.LLLL#.LL\n\
                     ..#.L.....\n\
                     LLL###LLL#\n\
                     #.LLLLL#.L\n\
                     #.L#LL#.L#";
        let seating_diagram_post6 = build_from_str(post6).unwrap();
        assert_eq!(seating_diagram.step_visible(), 5);
        assert_eq!(seating_diagram, seating_diagram_post6);

        assert_eq!(seating_diagram.step_visible(), 0);
    }
}
