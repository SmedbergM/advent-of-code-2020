use std::io;
use std::io::prelude::*;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use std::rc::Rc;

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

struct BaggageRegulations { 
    regulations: BTreeMap<Rc<Bag>, BaggageRegulation2>
}

impl BaggageRegulations {
    fn new() -> BaggageRegulations {
        BaggageRegulations{ regulations: BTreeMap::new() }
    }

    fn insert_line(&mut self, line: &str) {
        lazy_static! {
            static ref LINE_PAT: Regex = Regex::new(r"(\w+) (\w+) bags contain (.+).").unwrap();
            static ref CONTENTS_PAT: Regex = Regex::new(r"(\d+) (\w+) (\w+) bag").unwrap();
        }

        for caps0 in LINE_PAT.captures(line) {
            let outer_bag = Bag::new(&caps0[1], &caps0[2]);
            let outer_bag_boxed = Rc::new(outer_bag);

            let child_refs: Vec<(Rc<Bag>, usize)> = CONTENTS_PAT.captures_iter(&caps0[3]).flat_map(|caps1| {
                usize::from_str_radix(&caps1[1], 10).map(|n| {
                    let child_bag = Rc::new(Bag::new(&caps1[2], &caps1[3]));
                    let child_regulation = self.regulations.entry(child_bag.clone()).or_insert(BaggageRegulation2::new());
                    child_regulation.is_contained_by.insert(outer_bag_boxed.clone());
                    (child_bag, n)
                })
            }).collect();

            // then add all children to outer_bag
            let outer_regulation = self.regulations.entry(outer_bag_boxed).or_insert(BaggageRegulation2::new());
            for (child_bag, n) in child_refs {
                outer_regulation.must_contain.insert(child_bag, n);
            }
        }
    }

    fn build<J>(lines: &mut J) -> BaggageRegulations
    where J: Iterator<Item=String> {
        let mut regs = BaggageRegulations::new();
        for line in lines {
            regs.insert_line(&line);
        }

        regs
    }

    fn walk_out_from(&self, bag: &Bag) -> BTreeSet<&Bag> {
        let mut r: BTreeSet<&Bag> = BTreeSet::new();
        let mut q = VecDeque::new();

        q.push_back(bag);

        while let Some(outer_bag) = q.pop_front() {
            for regulation in self.regulations.get(outer_bag) {
                for parent in &regulation.is_contained_by {
                    r.insert(parent);
                    q.push_back(parent);
                }
            }
        }

        r
    }

    fn transitive_contents(&self, bag: &Bag) -> BTreeMap<&Bag, usize> {
        let mut r: BTreeMap<&Bag, usize> = BTreeMap::new();
        let mut q: VecDeque<(&Bag, usize)> = VecDeque::new();

        for regulation in self.regulations.get(bag) {
            for (child, &n) in &regulation.must_contain {
                q.push_back((child, n))
            }
        }

        while let Some((child, n0)) = q.pop_front() {
            *r.entry(child).or_insert(0) += n0;
            for regulation in self.regulations.get(child) {
                for (grandchild, n1) in &regulation.must_contain {
                    q.push_back((grandchild, n0 * n1));
                }
            }
        }

        r
    }
}

struct BaggageRegulation2 {
    must_contain: BTreeMap<Rc<Bag>, usize>,
    is_contained_by: BTreeSet<Rc<Bag>>
}

impl BaggageRegulation2 {
    fn new() -> BaggageRegulation2 {
        BaggageRegulation2 { must_contain: BTreeMap::new(), is_contained_by: BTreeSet::new() }
    }
}


fn main() {
    let stdin = io::stdin();
    let baggage_regulations = BaggageRegulations::build(&mut stdin.lock().lines().flatten());
    println!("Parsed {} baggage regulations.", baggage_regulations.regulations.len());

    let my_bag = Bag::new("shiny", "gold");
    let can_contain_my_bag = baggage_regulations.walk_out_from(&my_bag);
    println!("{} bags can contain my shiny gold bag.", can_contain_my_bag.len());

    let my_contents = baggage_regulations.transitive_contents(&my_bag);
    let my_contents_total: usize = my_contents.values().sum();
    println!("My bag must contain {} other bags.", my_contents_total);
}

#[cfg(test)]
mod day07_spec {
    use super::*;

    fn get_regulation<'a>(regs: &'a BaggageRegulations, adj: &str, color: &str) -> Option<&'a BaggageRegulation2> {
        regs.regulations.get(&Bag::new(adj, color))
    }

    fn get_required_contents(outer: &BaggageRegulation2, adj: &str, color: &str) -> usize {
        *outer.must_contain.get(&Bag::new(adj, color)).unwrap_or(&0)
    }

    #[test]
    fn baggage_regulations_build() {
        let input = "light red bags contain 1 bright white bag, 2 muted yellow bags.\n\
        dark orange bags contain 3 bright white bags, 4 muted yellow bags.\n\
        bright white bags contain 1 shiny gold bag.\n\
        muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.\n\
        shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.\n\
        dark olive bags contain 3 faded blue bags, 4 dotted black bags.\n\
        vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.\n\
        faded blue bags contain no other bags.\n\
        dotted black bags contain no other bags.\n";
        
        let regs = BaggageRegulations::build(&mut input.lines().map(|s| s.to_owned()));
        let light_red_reg = get_regulation(&regs, "light", "red").unwrap();
        assert!(light_red_reg.is_contained_by.is_empty());
        assert_eq!(*light_red_reg.must_contain.get(&Bag::new("bright", "white")).unwrap(), 1);
        assert_eq!(*light_red_reg.must_contain.get(&Bag::new("muted", "yellow")).unwrap(), 2);

        let muted_yellow_reg = get_regulation(&regs, "muted", "yellow").unwrap();
        assert!(muted_yellow_reg.is_contained_by.contains(&Bag::new("light", "red")));
        assert!(muted_yellow_reg.is_contained_by.contains(&Bag::new("dark", "orange")));
        assert_eq!(get_required_contents(&muted_yellow_reg, "shiny", "gold"), 2);
        assert_eq!(get_required_contents(&muted_yellow_reg, "faded", "blue"), 9);

        let faded_blue_reg = regs.regulations.get(&Bag::new("faded", "blue")).unwrap();
        assert!(faded_blue_reg.is_contained_by.contains(&Bag::new("vibrant", "plum")));
        assert_eq!(faded_blue_reg.is_contained_by.len(), 3);
        assert_eq!(get_required_contents(&faded_blue_reg, "dotted", "black"), 0);
        assert_eq!(get_required_contents(&faded_blue_reg, "muted", "yellow"), 0);
    }

    #[test]
    fn walk_out_from_test() {
        let input = "light red bags contain 1 bright white bag, 2 muted yellow bags.\n\
        dark orange bags contain 3 bright white bags, 4 muted yellow bags.\n\
        bright white bags contain 1 shiny gold bag.\n\
        muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.\n\
        shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.\n\
        dark olive bags contain 3 faded blue bags, 4 dotted black bags.\n\
        vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.\n\
        faded blue bags contain no other bags.\n\
        dotted black bags contain no other bags.\n";
        
        let regs = BaggageRegulations::build(&mut input.lines().map(|s| s.to_owned()));

        let shiny_gold_containing_bags = regs.walk_out_from(&Bag::new("shiny", "gold"));
        assert_eq!(shiny_gold_containing_bags.len(), 4);
    }

    #[test]
    fn transitive_contents_test() {
        let input = "shiny gold bags contain 2 dark red bags.\n\
        dark red bags contain 2 dark orange bags.\n\
        dark orange bags contain 2 dark yellow bags.\n\
        dark yellow bags contain 2 dark green bags.\n\
        dark green bags contain 2 dark blue bags.\n\
        dark blue bags contain 2 dark violet bags.\n\
        dark violet bags contain no other bags.";
        
        let regs = BaggageRegulations::build(&mut input.lines().map(|s| s.to_owned()));

        let tc = regs.transitive_contents(&Bag::new("shiny", "gold"));
        assert_eq!(tc.get(&Bag::new("dark", "red")), Some(&2));
        assert_eq!(tc.get(&Bag::new("dark", "orange")), Some(&4));
        let tc_sum: usize = tc.values().sum();
        assert_eq!(tc_sum, 126);
    }
}