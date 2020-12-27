use std::io::prelude::*;

use mod_exp::mod_exp;

const Q: u64 = 2020_1227;

// computes (x * y) mod Q
// We need u64 since Q is 25 bits, so the product of x and y could have as much as
// 50 bits before reduction
const fn mod_mult(x: u64, y: u64) -> u64 {
    (x % Q) * (y % Q) % Q
}

// computes the discrete log of x (mod Q) where the base is b
// I.e. solves the equation b^n = x (mod Q)
// Very naive -- just brute force since Q is small
const fn log_q(b: u64, x: u64) -> Option<u64> {
    let mut n = 0;
    let mut pow = 1;

    while n < Q {
        if pow == x {
            return Some(n)
        }
        n += 1;
        pow = mod_mult(b, pow);
    }

    None
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdin_lines = stdin.lock().lines();

    let public_key_1: u64 = stdin_lines.next().and_then(|result| {
        result.ok().and_then(|line| u64::from_str_radix(&line, 10).ok())
    }).unwrap();
    let public_key_2: u64 = stdin_lines.next().and_then(|result| {
        result.ok().and_then(|line| u64::from_str_radix(&line, 10).ok())
    }).unwrap();

    println!("Card public key: {}\nDoor public key: {}", public_key_1, public_key_2);

    let loop_size_1: u64 = log_q(7, public_key_1).unwrap();
    let loop_size_2: u64 = log_q(7, public_key_2).unwrap();

    println!("Card secret key: {}\nDoor secret key: {}", loop_size_1, loop_size_2);

    let handshake_1: u64 = mod_exp(public_key_1, loop_size_2, Q);
    let handshake_2: u64 = mod_exp(public_key_2, loop_size_1, Q);

    println!("Handshake 1: {}\nHandshake 2: {}", handshake_1, handshake_2);
}

#[cfg(test)]
mod day25_spec {
    use super::*;

    #[test]
    fn log_q_test() {
        let k = 5764801;
        assert_eq!(log_q(7, k), Some(8));

        let k = 17807724;
        assert_eq!(log_q(7, k), Some(11));
    }
}