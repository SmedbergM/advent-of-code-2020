use std::io;
use std::io::prelude::*;
use std::collections::BTreeSet;

// We represent a puzzle input as a width > 0, a height >=0, and a set of "trees"
// represented as (x,y) pairs, where 0 <= x < width and 0 <= y < height.
// Note that conceptually the pairs (locations of trees) repeat periodically to the right:
// if (x,y) is a tree, then (x + width, y) is a tree as well.
#[derive(Debug, PartialEq, Eq)]
struct Puzzle {
    width: usize,
    height: usize,
    trees: BTreeSet<(usize, usize)>
}

impl Puzzle {
    fn add_line(&mut self, line: &str) {
        let y = self.height;
        self.height += 1;
        for (x, c) in line.chars().enumerate() {
            if c == '#' {
                self.trees.insert((x,y));
            }
        }
    }

    fn build<J>(lines: &mut J) -> Option<Puzzle>
    where J: Iterator<Item=String> {
        lines.next().map(|first_line| {
            let mut puzzle = Puzzle { width: first_line.len(), height: 0, trees: BTreeSet::new() };
            puzzle.add_line(&first_line);
            while let Some(line) = lines.next() {
                puzzle.add_line(&line);
            }
            puzzle
        })
    }

    // Count the trees you hit starting at (0,0) and moving on the specified slope.
    fn traverse(&self, dx: usize, dy: usize) -> usize {
        let mut x = 0;
        let mut tree_count = 0;
        for y in (0..self.height).step_by(dy) {
            // invariant: x < self.width
            tree_count += (self.trees.contains(&(x,y))) as usize;

            x = (x + dx) % self.width;
        }
        tree_count
    }
}

fn main() {
    let stdin = io::stdin();
    let puzzle = Puzzle::build(&mut stdin.lock().lines().flatten()).unwrap();
    
    println!("Puzzle parsed with width {}, height {}, tree count {}",
        puzzle.width, puzzle.height, puzzle.trees.len());

    let (dx, dy) = (1,1);
    let tree_count_1_1 = puzzle.traverse(dx, dy);
    println!("With dy/dx = {}/{}, I hit {} trees.", dy, dx, tree_count_1_1);
    
    let (dx, dy) = (3,1);
    let tree_count_3_1 = puzzle.traverse(dx, dy);
    println!("With dy/dx = {}/{}, I hit {} trees.", dy, dx, tree_count_3_1);

    let (dx, dy) = (5,1);
    let tree_count_5_1 = puzzle.traverse(dx, dy);
    println!("With dy/dx = {}/{}, I hit {} trees.", dy, dx, tree_count_5_1);

    let (dx, dy) = (7,1);
    let tree_count_7_1 = puzzle.traverse(dx, dy);
    println!("With dy/dx = {}/{}, I hit {} trees.", dy, dx, tree_count_7_1);

    let (dx, dy) = (1,2);
    let tree_count_1_2 = puzzle.traverse(dx, dy);
    println!("With dy/dx = {}/{}, I hit {} trees.", dy, dx, tree_count_1_2);

    println!("Product: {}", tree_count_1_1 * tree_count_3_1 * tree_count_5_1 * tree_count_7_1 * tree_count_1_2);
}

#[cfg(test)]
mod day03_spec {
    use super::*;

    const PUZZLE_INPUT: &str =
    "..##.......\n\
     #...#...#..\n\
     .#....#..#.\n\
     ..#.#...#.#\n\
     .#...##..#.\n\
     ..#.##.....\n\
     .#.#.#....#\n\
     .#........#\n\
     #.##...#...\n\
     #...##....#\n\
     .#..#...#.#";

    mod build {
        use super::*;

        #[test]
        fn should_build_a_puzzle() {
            let puzzle = Puzzle::build(&mut PUZZLE_INPUT.lines().map(|s| s.to_owned())).unwrap();
            assert_eq!(puzzle.width, 11);
            assert_eq!(puzzle.height, 11);
            assert!(puzzle.trees.contains(&(3,0)));
            assert!(!puzzle.trees.contains(&(0,3)));
            assert!(puzzle.trees.contains(&(1,2)));
            assert!(puzzle.trees.contains(&(10,10)));
        }
    }

    mod traverse {
        use super::*;

        #[test]
        fn should_count_trees() {
            let puzzle = Puzzle::build(&mut PUZZLE_INPUT.lines().map(|s| s.to_owned())).unwrap();

            assert_eq!(puzzle.traverse(3, 1), 7);
        }
    }
}