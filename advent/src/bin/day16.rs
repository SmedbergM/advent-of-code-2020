use std::io::prelude::*;
use std::collections::VecDeque;

#[macro_use]
extern crate lazy_static;

use regex::Regex;

struct FieldRule {
    field_name: String,
    r0_min: usize,
    r0_max: usize,
    r1_min: usize,
    r1_max: usize
}

impl FieldRule {
    fn parse(line: &str) -> Option<FieldRule> {
        lazy_static! {
            static ref RULE_PAT: Regex = Regex::new(r"(.+): (\d+)-(\d+) or (\d+)-(\d+)").unwrap();
        }
        
        let r: Option<FieldRule> = RULE_PAT.captures(line).and_then(|caps| {
            let field_name = caps[1].to_owned();
            usize::from_str_radix(&caps[2], 10).ok().and_then(|r0_min| {
            usize::from_str_radix(&caps[3], 10).ok().and_then(|r0_max| {
            usize::from_str_radix(&caps[4], 10).ok().and_then(|r1_min| {
            usize::from_str_radix(&caps[5], 10).ok().map(|r1_max| {
                FieldRule { field_name, r0_min, r0_max, r1_min, r1_max }
            }) }) }) })
        });

        r
    }

    fn valid(&self, x: usize) -> bool {
       (self.r0_min <= x && x <= self.r0_max) || (self.r1_min <= x && x <= self.r1_max)
    }
}

struct Ticket(Vec<usize>);

impl Ticket {
    fn parse(line: &str) -> Ticket {
        let fields: Vec<usize> = line.split(',').flat_map(|s| usize::from_str_radix(s, 10)).collect();
        Ticket(fields)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    // Checks fields against provided rules. If any field is out-of-bounds on all rules,
    // the index of that field will be in the returned Vec.
    fn is_corrupt(&self, rules: &Vec<FieldRule>) -> Option<Vec<usize>> {
        let mut oob = vec!();

        'f: for (field_idx, field) in self.0.iter().enumerate() {
            for rule in rules {
                if rule.valid(*field) {
                    continue 'f
                }
            }
            oob.push(field_idx);
        }

        if oob.is_empty() {
            None
        } else {
            Some(oob)
        }
    }
}

fn identify_fields<'a>(field_rules: &'a Vec<FieldRule>, valid_tickets: &Vec<Ticket>) -> Option<Vec<&'a FieldRule>> {
    #[derive(Clone, Copy)]
    enum Candidate {
        Eliminated,
        Possible,
        Committed
    }

    enum Instruction {
        Eliminate(usize, usize), // field_idx, rule_idx
        Commit(usize, usize)
    }

    struct Candidates{
        cs: Vec<Vec<Candidate>>,
        queue: VecDeque<Instruction>
    }

    impl std::fmt::Display for Candidates {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut s = String::new();

            for row in &self.cs {
                for j in (0..=row.len()).step_by(8) {
                    for c in &row[j..(j+8).min(row.len())] {
                        match c {
                            Candidate::Eliminated => s.push('.'),
                            Candidate::Possible => s.push('*'),
                            Candidate::Committed => s.push('C')
                        }    
                    }
                    s.push(' ');
                }
                s.push('\n');
            }

            write!(formatter, "{}", s)
        }
    }

    impl Candidates {
        fn new(n: usize) -> Candidates {
            let cs = vec![vec![Candidate::Possible; n]; n];
            let queue = VecDeque::new();
            Candidates { cs, queue }
        }

        fn eliminate(&mut self, field_idx: usize, rule_idx: usize) -> Result<(), String> {
            match self.cs[field_idx].get_mut(rule_idx) {
                Some(Candidate::Committed) => return Err("Inconsistency".to_owned()),
                Some(Candidate::Eliminated) => return Ok(()),
                Some(p@Candidate::Possible) => {
                    *p = Candidate::Eliminated;
                    if let SearchResult::Single(other_rule_idx) = self.search_row(field_idx) {
                        self.queue.push_back(Instruction::Commit(field_idx, other_rule_idx));
                    }
                    if let SearchResult::Single(other_field_idx) = self.search_column(rule_idx) {
                        self.queue.push_back(Instruction::Commit(other_field_idx, rule_idx));
                    }
                },
                None => return Err("Out of bounds?".to_owned())
            }

            self.clear_queue()
        }

        fn commit(&mut self, field_idx: usize, rule_idx: usize) -> Result<(), String> {
            match self.cs[field_idx].get_mut(rule_idx) {
                Some(Candidate::Eliminated) => return Err("Inconsistency".to_owned()),
                Some(Candidate::Committed) => return Ok(()),
                Some(p@Candidate::Possible) => {
                    *p = Candidate::Committed;
                    for idx in 0..self.cs.len() {
                        if idx != field_idx {
                            self.queue.push_back(Instruction::Eliminate(idx, rule_idx));
                        }
                        if idx != rule_idx {
                            self.queue.push_back(Instruction::Eliminate(field_idx, idx));
                        }
                    }
                },
                None => return Err("Out of bounds?".to_owned())
            }

            self.clear_queue()
        }

        fn clear_queue(&mut self) -> Result<(), String> {
            while let Some(instruction) = self.queue.pop_front() {
                match instruction {
                    Instruction::Eliminate(field_idx, rule_idx) => {
                        if let Err(msg) = self.eliminate(field_idx, rule_idx) {
                            return Err(msg)
                        }
                    },
                    Instruction::Commit(field_idx, rule_idx) => {
                        if let Err(msg) = self.commit(field_idx, rule_idx) {
                            return Err(msg)
                        }
                    }
                }
            }

            Ok(())
        }

        fn search_row(&self, field_idx: usize) -> SearchResult {
            let mut r = SearchResult::Empty;

            for (rule_idx, c) in self.cs[field_idx].iter().enumerate() {
                match (&r, c) {
                    (_, Candidate::Committed) => return SearchResult::Committed(rule_idx),
                    (SearchResult::Empty, Candidate::Possible) =>
                        r = SearchResult::Single(rule_idx),
                    (SearchResult::Single(_), Candidate::Possible) =>
                        // Gotcha alert! Because possibles may not be cleared at the same time as commits are made,
                        // we might end up returning SearchResult::Committed
                        r = SearchResult::Multiple,
                    _ => ()
                }
            }

            r
        }

        fn search_column(&self, rule_idx: usize) -> SearchResult {
            let mut r = SearchResult::Empty;

            for field_idx in 0..self.cs.len() {
                match (&r, self.cs[field_idx][rule_idx]) {
                    (_, Candidate::Committed) => return SearchResult::Committed(field_idx),
                    (SearchResult::Empty, Candidate::Possible) => r = SearchResult::Single(field_idx),
                    (SearchResult::Single(_), Candidate::Possible) => r = SearchResult::Multiple,
                    _ => ()
                }
            }

            r
        }
    }

    enum SearchResult {
        Empty,
        Single(usize), // used when we have not yet realized that we have eliminated all but one in the row/column
        Committed(usize),
        Multiple
    }

    let mut candidates = Candidates::new(field_rules.len());

    for ticket in valid_tickets {
        for (field_idx, field) in ticket.0.iter().enumerate() {
            for (rule_idx, rule) in field_rules.iter().enumerate() {
                if !rule.valid(*field) {
                    if let Err(_) = candidates.eliminate(field_idx, rule_idx) {
                        return None
                    }
                }
            }
        }
    }

    println!("Candidates:\n{}", candidates);

    let mut ret: Vec<&'a FieldRule> = vec!();

    'r: for row in candidates.cs {
        for (rule_idx, c) in row.iter().enumerate() {
            if let Candidate::Committed = c {
                ret.push(&field_rules[rule_idx]);
                continue 'r
            }
        }
        return None
    }

    Some(ret)
}

fn eat_line<J>(j: &mut J, expected: &str) where J: Iterator<Item=String> {
    if let Some(line) = j.next() {
        if line.as_str() != expected {
            eprintln!("Unexpected input line {}", line)
        }
    } else {
        eprintln!("Unexpected EOF!")
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdin_lines = stdin.lock().lines().flatten();

    println!("Getting field rules:");
    let mut field_rules: Vec<FieldRule> = vec!();
    while let Some(line) = stdin_lines.next() {
        if line.is_empty() {
            break
        } else {
            for field_rule in FieldRule::parse(&line) {
                field_rules.push(field_rule);
            }
        }
    }
    println!("{} field rules parsed.", field_rules.len());

    eat_line(&mut stdin_lines, "your ticket:");
    let my_ticket = stdin_lines.next().map(|line| Ticket::parse(&line)).unwrap();
    if my_ticket.len() != field_rules.len() {
        eprintln!("Length mismatch: {} field rules but {} fields.", field_rules.len(), my_ticket.len());
    }

    eat_line(&mut stdin_lines, "");
    eat_line(&mut stdin_lines, "nearby tickets:");

    let mut nearby_tickets = vec!();
    let mut scanning_error_rate = 0;
    for line in stdin_lines {
        let ticket = Ticket::parse(&line);
        if ticket.len() != field_rules.len() {
            eprintln!("Length mismatch: {} field rules but {} fields.", field_rules.len(), ticket.len());
        }

        if let Some(bad_indices) = ticket.is_corrupt(&field_rules) {
            for idx in bad_indices {
                scanning_error_rate += ticket.0[idx]
            }
        } else {
            nearby_tickets.push(ticket)
        }
    }
    println!("{} valid nearby tickets parsed; scanning error rate: {}", nearby_tickets.len(), scanning_error_rate);

    let ordered_fields = identify_fields(&field_rules, &nearby_tickets).unwrap();

    let mut prod = 1;

    for (field, rule) in my_ticket.0.iter().zip(ordered_fields) {
        if rule.field_name.starts_with("departure") {
            prod *= field
        }
    }

    println!("Departure fields product: {}", prod);
}

#[cfg(test)]
mod day16_spec {
    use super::*;

    mod field_rule {
        use super::*;

        #[test]
        fn parse_test() {
            let field_rule = FieldRule::parse("class: 1-3 or 5-7").unwrap();
            assert_eq!(field_rule.field_name, "class");
            assert_eq!(field_rule.r0_min, 1);
            assert_eq!(field_rule.r0_max, 3);
            assert_eq!(field_rule.r1_min, 5);
            assert_eq!(field_rule.r1_max, 7);

            let field_rule = FieldRule::parse("departure time: 29-483 or 491-963").unwrap();
            assert_eq!(field_rule.field_name, "departure time");
        }

        #[test]
        fn valid_test() {
            let class = FieldRule::parse("class: 1-3 or 5-7").unwrap();

            assert!(class.valid(6));
            assert!(!class.valid(4));
        }
    }


}