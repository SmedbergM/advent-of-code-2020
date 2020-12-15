use std::io::prelude::*;
use std::collections::HashMap;

fn elf_memory_game(inits: &Vec<usize>, turns: usize) -> usize {
    let mut last_occurrence: HashMap<usize, usize> = HashMap::new();
    let mut current: usize = match inits.first() {
        None => {
            eprintln!("Elf memory game cannot be played without seed numbers!");
            return 0;
        },
        Some(s) => {
            let mut c = *s;
            let mut prev = None;
            for (idx, seed) in inits.iter().enumerate() {
                if let Some(p) = prev {
                    last_occurrence.insert(p, idx - 1);
                }
                c = *seed;
                prev = Some(c);
            }

            c
        }
    };

    for idx in (inits.len() - 1)..(turns - 1) {
        match last_occurrence.insert(current, idx) {
            None => current = 0,
            Some(prev_idx) => current = idx - prev_idx
        }        
    }
    
    current
}

fn main() {
    let stdin = std::io::stdin();
    let seeds: Vec<usize> = stdin.lock().lines().flatten()
        .flat_map(|line| {
            let us: Vec<usize> = line.split(',')
                .flat_map(|w| usize::from_str_radix(w, 10).ok()).collect();
            us
        })
        .collect();
    let turns = 2020;
    let result = elf_memory_game(&seeds, turns);
    println!("{}th number in the game: {}", turns, result);

    let turns = 30_000_000;
    let result = elf_memory_game(&seeds, turns);
    println!("{}th number in the game: {}", turns, result);
}

#[cfg(test)]
mod day15_spec {
    use super::*;

    #[test]
    fn elf_game_test() {
        assert_eq!(elf_memory_game(&vec!(1,3,2), 2020), 1);
        assert_eq!(elf_memory_game(&vec!(2,1,3), 2020), 10);
        assert_eq!(elf_memory_game(&vec!(1,2,3), 2020), 27);
        assert_eq!(elf_memory_game(&vec!(2,3,1), 2020), 78);
        assert_eq!(elf_memory_game(&vec!(3,2,1), 2020), 438);
        assert_eq!(elf_memory_game(&vec!(3,1,2), 2020), 1836);
    }
}