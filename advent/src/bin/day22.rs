use std::io::prelude::*;
use std::collections::{VecDeque, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn eat_line<J>(j: &mut J, expected: &str) where J: Iterator<Item=String> {
    if let Some(line) = j.next() {
        if line.as_str() != expected {
            eprintln!("Unexpected input line {}", line)
        }
    } else {
        eprintln!("Unexpected EOF!")
    }
}

fn read_deck<J>(j: &mut J) -> Result<Vec<usize>, String> where J: Iterator<Item=String> {
    let mut deck = vec!();

    while let Some(line) = j.next() {
        if line.is_empty() {
            break
        } else {
            match usize::from_str_radix(&line, 10) {
                Ok(card) => deck.push(card),
                err => {
                    let msg = format!("{:?}: Cannot parse {} as number", err, line);
                    return Err(msg)
                }
            }
        }
    }

    Ok(deck)
}

fn score(deck: &VecDeque<usize>) -> usize {
    deck.iter().rev().enumerate().fold(0, |acc, (idx, card)| {
        acc + (card * (1 + idx))
    })
}

#[derive(Debug, PartialEq, Eq)]
enum Player {
    Player1, Player2
}

fn play_combat(deck_1: &Vec<usize>, deck_2: &Vec<usize>) -> (Player, usize) {
    let mut deck_1: VecDeque<usize> = deck_1.iter().map(|p| *p).collect();
    let mut deck_2: VecDeque<usize> = deck_2.iter().map(|p| *p).collect();

    while !deck_1.is_empty() && !deck_2.is_empty() {
        let card_1 = deck_1.pop_front().unwrap();
        let card_2 = deck_2.pop_front().unwrap();

        if card_1 > card_2 {
            deck_1.push_back(card_1);
            deck_1.push_back(card_2);
        } else {
            deck_2.push_back(card_2);
            deck_2.push_back(card_1);
        }
    }

    if deck_1.is_empty() {
        (Player::Player2, score(&deck_2))
    } else {
        (Player::Player1, score(&deck_1))
    }
}

fn hash(dq_1: &VecDeque<usize>, dq_2: &VecDeque<usize>) -> u64 {
    let mut hasher = DefaultHasher::new();
    dq_1.hash(&mut hasher);
    dq_2.hash(&mut hasher);
    hasher.finish()
}

fn play_recursive_combat(deck_1: &Vec<usize>, deck_2: &Vec<usize>) -> (Player, usize) {
    let mut game_number = 0;
    fn rc(deck_1: &[usize], deck_2: &[usize], game_number: &mut usize) -> (Player, usize) {
        *game_number += 1;
        let gn = *game_number;
        let mut rn = 0;
        let mut previous_hashes = HashSet::new();
        let mut dq_1: VecDeque<usize> = deck_1.iter().map(|p| *p).collect();
        let mut dq_2: VecDeque<usize> = deck_2.iter().map(|p| *p).collect();
        while !dq_1.is_empty() && !dq_2.is_empty() {
            rn += 1;
            if !previous_hashes.insert(hash(&dq_1, &dq_2)) {
                // then we have already played this game
                println!("Game {} has encountered a hash collision", gn);
                return (Player::Player1, 0)
            }
            let card_1 = dq_1.pop_front().unwrap() as usize;
            let card_2 = dq_2.pop_front().unwrap() as usize;
            let winner: Player;
            if dq_1.len() >= card_1 && dq_2.len() >= card_2 {
                let dq_1_slice = dq_1.make_contiguous();
                let dq_2_slice = dq_2.make_contiguous();
                println!("Game {} spawning a recursive game to determine winner of round {}",
                    gn, rn);
                let w = rc(&dq_1_slice[..card_1], &dq_2_slice[..card_2], game_number);
                winner = w.0;
            } else if card_1 > card_2 {
                winner = Player::Player1;
            } else {
                winner = Player::Player2;
            }
            match winner {
                Player::Player1 => {
                    println!("Player 1 wins round {} of game {}", rn, gn);
                    dq_1.push_back(card_1);
                    dq_1.push_back(card_2);
                },
                Player::Player2 => {
                    println!("Player 2 wins round {} of game {}", rn, gn);
                    dq_2.push_back(card_2);
                    dq_2.push_back(card_1);
                }
            }
        }
        if dq_2.is_empty() {
            println!("Player 1 wins game {}", gn);
            println!("Winning deck: {:?}", dq_1);
            (Player::Player1, score(&dq_1))
        } else {
            println!("Player 2 wins game {}", gn);
            println!("Winning deck: {:?}", dq_2);
            (Player::Player2, score(&dq_2))
        }
    }

    rc(&deck_1[..], &deck_2[..], &mut game_number)
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdin_lines = stdin.lock().lines().flatten();
    eat_line(&mut stdin_lines, "Player 1:");

    let deck_1: Vec<usize> = read_deck(&mut stdin_lines).unwrap();

    eat_line(&mut stdin_lines, "Player 2:");
    let deck_2: Vec<usize> = read_deck(&mut stdin_lines).unwrap();

    let winner = play_combat(&deck_1, &deck_2);
    println!("Player {:?} wins Combat with a score of {}", winner.0, winner.1);

    let winner = play_recursive_combat(&deck_1, &deck_2);
    println!("Player {:?} wins Recursive Combat with a score of {}", winner.0, winner.1);
}

#[cfg(test)]
mod day22_spec {
    use super::*;

    #[test]
    fn combat_test() {
        let deck_1 = vec!(9, 2, 6, 3, 1);
        let deck_2 = vec!(5, 8, 4, 7, 10);

        let winner = play_combat(&deck_1, &deck_2);
        assert_eq!(winner.0, Player::Player2);
        assert_eq!(winner.1, 306)
    }

    #[test]
    fn recursive_combat_loop_test() {
        let deck_1 = vec!(43, 19);
        let deck_2 = vec!(2, 29, 14);
        let winner = play_recursive_combat(&deck_1, &deck_2);
        assert_eq!(winner.0, Player::Player1);
        assert_eq!(winner.1, 0);
    }

    #[test]
    fn recursive_combat_test() {
        let deck_1 = vec!(9, 2, 6, 3, 1);
        let deck_2 = vec!(5, 8, 4, 7, 10);

        let winner = play_recursive_combat(&deck_1, &deck_2);
        assert_eq!(winner.0, Player::Player2);
        assert_eq!(winner.1, 291);
    }
}