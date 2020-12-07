use std::io;
use std::io::prelude::*;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[macro_use]
extern crate lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Bag{ adj: String, color: String }

impl Bag {
    fn new(a: &str, c: &str) -> Bag {
        Bag { adj: a.to_owned(), color: c.to_owned() }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BaggageRegulation {
    outer: Bag,
    contents: BTreeMap<Bag, usize>
}

impl BaggageRegulation {
    fn contains(&self, bag: &Bag) -> bool {
        self.contents.contains_key(bag)
    }
}

fn parse_line(line: &str) -> Option<BaggageRegulation> {
    lazy_static! {
        static ref LINE_PAT: Regex = Regex::new(r"(\w+) (\w+) bags contain (.+).").unwrap();
        static ref CONTENTS_PAT: Regex = Regex::new(r"(\d+) (\w+) (\w+) bag").unwrap();
    }

    LINE_PAT.captures(line).map(|cap| {
        let outer = Bag::new(&cap[1], &cap[2]);
        let mut contents = BTreeMap::new();

        for cap2 in CONTENTS_PAT.captures_iter(&cap[3]) {
            for n in usize::from_str_radix(&cap2[1], 10) {
                let bag = Bag::new(&cap2[2], &cap2[3]);
                contents.insert(bag, n);
            }
        }

        BaggageRegulation { outer, contents }
    })
}

fn transitive_containers<'a>(regulations: &'a Vec<BaggageRegulation>, bag: &Bag) -> BTreeSet<&'a Bag> {
    let mut tcs = BTreeSet::new();
    let mut intermediate_containers = VecDeque::new();
    intermediate_containers.push_back(bag);

    while let Some(ic) = intermediate_containers.pop_front() {
        for regulation in regulations {
            if regulation.contains(ic) {
                tcs.insert(&regulation.outer);
                intermediate_containers.push_back(&regulation.outer)
            }
        }
    }

    tcs
}

fn transitive_contents<'a>(regulations: &'a Vec<BaggageRegulation>, outermost: &Bag) -> BTreeMap<&'a Bag, usize> {
    let regulations_by_bag: BTreeMap<&Bag, &BaggageRegulation> = regulations.iter().map(|r| {
        (&r.outer, r)
    }).collect();
    let mut tcs: BTreeMap<&'a Bag, usize> = BTreeMap::new();
    let mut q: VecDeque<(&Bag, usize)> = VecDeque::new();

    for regulation in regulations_by_bag.get(outermost) {
        for (c, n) in &regulation.contents {
            tcs.insert(c, *n);
            q.push_back((c, *n))
        }
    }

    while let Some((bag, n)) = q.pop_front() {
        for regulation in regulations_by_bag.get(bag) {
            for (c, n2) in &regulation.contents {
                *tcs.entry(c).or_insert(0) += n * n2;
                q.push_back((c, n * n2))
            }
        }
    }

    tcs
}


fn main() {
    let stdin = io::stdin();
    let baggage_regulations: Vec<BaggageRegulation> = stdin.lock().lines().flatten().flat_map(|line| parse_line(&line)).collect();
    println!("Parsed {} baggage regulations.", baggage_regulations.len());

    let my_bag = Bag::new("shiny", "gold");
    let my_wrappers = transitive_containers(&baggage_regulations, &my_bag);
    println!("{} bags can contain my shiny gold bag.", my_wrappers.len());

    let my_contents = transitive_contents(&baggage_regulations, &my_bag);
    let my_contents_total: usize = my_contents.values().sum();
    println!("My bag must contain {} other bags.", my_contents_total);
}

#[cfg(test)]
mod day07_spec {
    use super::*;

    fn to_regulation(a: &str, c: &str, cs: Vec<(usize, &str, &str)>) -> BaggageRegulation {
        let contents = cs.iter().map(|(n, a1, c1)| {
            (Bag::new(a1, c1), *n)
        }).collect();
        BaggageRegulation {
            outer: Bag::new(a, c), contents
        }
    }

    #[test]
    fn parse_line_test() {
        let line = "light red bags contain 1 bright white bag, 2 muted yellow bags.";
        assert_eq!(parse_line(&line), Some(to_regulation("light", "red", vec!(
            (1, "bright", "white"),
            (2, "muted", "yellow")
        ))));

        let line = "dark orange bags contain 3 bright white bags, 4 muted yellow bags.";
        assert_eq!(parse_line(&line), Some(to_regulation("dark", "orange", vec!(
            (3, "bright", "white"),
            (4, "muted", "yellow")
        ))));

        let line = "bright white bags contain 1 shiny gold bag.";
        assert_eq!(parse_line(&line), Some(to_regulation("bright", "white", vec!(
            (1, "shiny", "gold")
        ))));

        let line = "faded blue bags contain no other bags.";
        assert_eq!(parse_line(&line), Some(BaggageRegulation {
            outer: Bag::new("faded", "blue"),
            contents: BTreeMap::new()
        }));
    }

    #[test]
    fn transitive_containers_test() {
        let my_bag = Bag::new("shiny", "gold");
        let regulations = vec!(
            to_regulation("light", "red", vec!(
                (1, "bright", "white"),
                (2, "muted", "yellow")
            )),
            to_regulation("dark", "orange", vec!(
                (3, "bright", "white"),
                (4, "muted", "yellow")
            )),
            to_regulation("bright", "white", vec!(
                (1, "shiny", "gold")
            )),
            to_regulation("muted", "yellow", vec!(
                (2, "shiny", "gold"),
                (9, "faded", "blue")
            )),
            to_regulation("shiny", "gold", vec!(
                (1, "dark", "olive"),
                (2, "vibrant", "plum")
            )),
            to_regulation("dark", "olive", vec!(
                (3, "faded", "blue"),
                (4, "dotted", "black")
            )),
            to_regulation("vibrant", "plum", vec!(
                (5, "faded", "blue"),
                (6, "dotted", "black")
            )),
            to_regulation("faded", "blue", vec!()),
            to_regulation("dotted", "black", vec!())
        );
        let allowed = transitive_containers(&regulations, &my_bag);
        println!("Containing bags: {:?}", allowed);
        assert_eq!(allowed.len(), 4);
    }

    #[test]
    fn transitive_contents_test() {
        let my_bag = Bag::new("shiny", "gold");
        let regulations = vec!(
            to_regulation("shiny", "gold", vec!(
                (2, "dark", "red")
            )),
            to_regulation("dark", "red", vec!(
                (2, "dark", "orange")
            )),
            to_regulation("dark", "orange", vec!(
                (2, "dark", "yellow")
            )),
            to_regulation("dark", "yellow", vec!(
                (2, "dark", "green")
            )),
            to_regulation("dark", "green", vec!(
                (2, "dark", "blue")
            )),
            to_regulation("dark", "blue", vec!(
                (2, "dark", "violet")
            )),
            to_regulation("dark", "violet", vec!())
        );
        let tc = transitive_contents(&regulations, &my_bag);
        let content_size: usize = tc.values().fold(0, |acc, u|(acc + u));
        assert_eq!(content_size, 126)
    }
}