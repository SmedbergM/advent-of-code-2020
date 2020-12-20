use std::io::prelude::*;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

#[macro_use]
extern crate lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Rule {
    Literal(char),
    Just(Vec<Rc<Rule>>),
    Or(Vec<Rc<Rule>>, Vec<Rc<Rule>>),

    // Dirty trick: In Part 2, rule 0 is equivalent to one of the form R+(R^n S^n),
    // i.e. apply rule R m times and S n times, where m > n >= 1.
    // So we replace it as such by hand.
    Rep(Rc<Rule>, Rc<Rule>)
}

impl Rule {
    fn partial_match<'a>(&self, msg: &'a str) -> Option<&'a str> {
        let f = |acc: Option<&'a str>, r: &Rc<Rule>| {
            acc.and_then(|s| r.partial_match(s))
        };
        match self {
            Rule::Literal(c) => {
                msg.chars().nth(0).and_then(|h| {
                    if *c == h {
                        Some(&msg[1..])
                    } else {
                        None
                    }
                })
            },
            Rule::Just(subrules) => {
                subrules.iter().fold(Some(msg), f)
            },
            Rule::Or(alt0, alt1) => {
                alt0.iter().fold(Some(msg), f)
                    .or_else(|| alt1.iter().fold(Some(msg), f))
            },
            Rule::Rep(r, s) => { // dead code
                r.partial_match(msg).and_then(|tail| s.partial_match(tail))
            }
        }
    }

    fn total_match(&self, msg: &str) -> bool {
        match self {
            Rule::Rep(r, s) => {
                let mut slice: &str = msg;
                let mut r_count = 0;
                while let Some(tail) = r.partial_match(slice) {
                    r_count += 1;
                    slice = tail;
                    let mut s_slice = tail;
                    let mut s_count = 0;
                    while let Some(s_tail) = s.partial_match(s_slice) {
                        s_count += 1;
                        if s_count >= r_count {
                            break
                        } else if s_tail.is_empty() {
                            return true
                        } else {
                            s_slice = s_tail;
                        }
                    }

                }
                return false
            },
            _ => match self.partial_match(msg) {
                Some("") => true,
                _ => false
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Rules(BTreeMap<u8, Rc<Rule>>);

struct RulesBuilder {
    just_rules: BTreeMap<u8, Vec<u8>>,
    or_rules: BTreeMap<u8, (Vec<u8>, Vec<u8>)>,
    rules: BTreeMap<u8, Rc<Rule>> // will always contain all the literal rules we know about
}

impl RulesBuilder {
    fn new() -> RulesBuilder {
        let just_rules = BTreeMap::new();
        let or_rules = BTreeMap::new();
        let rules = BTreeMap::new();
        RulesBuilder { just_rules, or_rules, rules }
    }

    fn add_line(&mut self, line: &str) {
        lazy_static! {
            static ref LITERAL_PAT: Regex = Regex::new(r#"(\d+):\s*"([a-z])""#).unwrap();
            static ref JUST_PAT: Regex = Regex::new(r"(\d+): ([\s\d]+)$").unwrap();
            static ref REF_PAT: Regex = Regex::new(r"(\d+): ([\s\d]+) \| ([\s\d]+)").unwrap();
            static ref WHITESPACE_PAT: Regex = Regex::new(r"\s+").unwrap();
        }

        fn split_u8(s: &str) -> Vec<u8> {
            WHITESPACE_PAT.split(s).flat_map(|w| u8::from_str_radix(w, 10).ok()).collect()
        }

        if let Some(caps) = LITERAL_PAT.captures(line) {
            let rule_idx = u8::from_str_radix(&caps[1], 10).unwrap();
            let c = caps[2].chars().nth(0).unwrap();
            self.rules.insert(rule_idx, Rc::new(Rule::Literal(c)));
        } else if let Some(caps) = JUST_PAT.captures(line) {
            let rule_idx = u8::from_str_radix(&caps[1], 10).unwrap();
            let dependent: Vec<u8> = split_u8(&caps[2]);
            self.just_rules.insert(rule_idx, dependent);

        } else if let Some(caps) = REF_PAT.captures(line) {
            let rule_idx = u8::from_str_radix(&caps[1], 10).unwrap();
            let alt0: Vec<u8> = split_u8(&caps[2]);
            let alt1: Vec<u8> = split_u8(&caps[3]);

            self.or_rules.insert(rule_idx, (alt0, alt1));
        } else {
            eprintln!("Unexpected line {}", line);
        }
    }

    fn build(mut self) -> Option<Rules> {
        let mut queue = VecDeque::new();

        for rule_idx in self.just_rules.keys() {
            queue.push_back(*rule_idx);
        }
        for rule_idx in self.or_rules.keys() {
            queue.push_back(*rule_idx);
        }

        while let Some(rule_idx) = queue.pop_front() {

            enum Resolver {
                Invalid,
                Unresolved,
                Resolved(Vec<Rc<Rule>>)
            }
            fn resolve_rules(this: &mut RulesBuilder, queue: &mut VecDeque<u8>, indices: &Vec<u8>) -> Resolver {
                let mut rules: Vec<Rc<Rule>> = vec!();
                let mut all_dependencies_resolved = true;

                for idx in indices {
                    match this.rules.get(idx) {
                        None => {
                            all_dependencies_resolved = false;
                            if this.just_rules.contains_key(idx) || this.or_rules.contains_key(idx) {
                                queue.push_front(*idx);
                            } else {
                                eprintln!("Unknown dependent rules {:?}", indices);
                                return Resolver::Invalid
                            }
                        },
                        Some(rule) => {
                            rules.push(Rc::clone(rule));
                        }
                    }
                }

                if all_dependencies_resolved {
                    return Resolver::Resolved(rules)
                } else {
                    return Resolver::Unresolved
                }
            }

            if self.rules.contains_key(&rule_idx) {
                // then we've already handled this rule
            } else if let Some(rs) = self.just_rules.get(&rule_idx) {
                let rs = rs.clone();
                match resolve_rules(&mut self, &mut queue, &rs) {
                    Resolver::Invalid => return None,
                    Resolver::Unresolved => {
                        queue.push_back(rule_idx); continue
                    },
                    Resolver::Resolved(rules) => self.rules.insert(rule_idx, Rc::new(Rule::Just(rules)))
                };
            } else if let Some((alt0, alt1)) = self.or_rules.get(&rule_idx) {
                let alt0 = alt0.clone();
                let alt1 = alt1.clone();
                match (
                    resolve_rules(&mut self, &mut queue, &alt0),
                    resolve_rules(&mut self, &mut queue, &alt1)
                ) {
                    (Resolver::Invalid, _) => return None,
                    (_, Resolver::Invalid) => return None,
                    (Resolver::Unresolved, _) => {
                        queue.push_back(rule_idx); continue
                    },
                    (_, Resolver::Unresolved) => {
                        queue.push_back(rule_idx); continue
                    },
                    (Resolver::Resolved(rules0), Resolver::Resolved(rules1)) => {
                        self.rules.insert(rule_idx, Rc::new(Rule::Or(rules0, rules1)));
                    }
                }
            }
        }

        Some(Rules(self.rules))
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut builder = RulesBuilder::new();
    for line in stdin.lock().lines().flatten() {
        if line.is_empty() {
            break
        }
        builder.add_line(&line);
    }

    let rules = builder.build().unwrap();
    let rule0 = rules.0.get(&0).unwrap();
    println!("Parsed {} rules.", rules.0.len());

    let rule0_recursive = {
        let rule42 = rules.0.get(&42).unwrap();
        let rule31 = rules.0.get(&31).unwrap();
        Rule::Rep(Rc::clone(&rule42), Rc::clone(&rule31))  
    };

    let mut m = 0;
    let mut m_recursive = 0;
    for line in stdin.lock().lines().flatten() {
        if rule0.total_match(&line) {
            m += 1;
        }
        if rule0_recursive.total_match(&line) {
            m_recursive += 1;
        }
    }

    println!("{} lines match rule 0", m);
    println!("{} lines match the recursive version of rule 0", m_recursive);
}

#[cfg(test)]
mod day19_spec {
    use super::*;

    mod rules_builder {
        use super::*;

        #[test]
        fn add_line_test() {
            let mut builder = RulesBuilder::new();

            let line = "1: \"a\"";
            builder.add_line(line);

            assert_eq!(builder.rules.get(&1), Some(&Rc::new(Rule::Literal('a'))));

            let line = "0: 4 1 5";
            builder.add_line(line);
            assert_eq!(builder.just_rules.get(&0), Some(&vec!(4, 1, 5)));

            let line = "2: 1 3 | 3 1";
            builder.add_line(line);
            assert_eq!(builder.or_rules.get(&2), Some(&(vec!(1, 3), vec!(3, 1))))
        }

        #[test]
        fn build_test_negative() {
            let line0 = "0: 1 2";
            let line1 = "1: \"a\"";
            let line2 = "2: 1 3 | 3 1";
            
            let mut builder = RulesBuilder::new();
            builder.add_line(line0);

            assert_eq!(builder.build(), None);
            
            let mut builder = RulesBuilder::new();
            builder.add_line(line0);
            builder.add_line(line1);
            assert_eq!(builder.build(), None);

            let mut builder = RulesBuilder::new();
            builder.add_line(line0);
            builder.add_line(line1);
            builder.add_line(line2);
            assert_eq!(builder.build(), None);
        }

        #[test]
        fn build_test() {
            let line0 = "0: 1 2";
            let line1 = "1: \"a\"";
            let line2 = "2: 1 3 | 3 1";
            let line3 = "3: \"b\"";

            let mut builder = RulesBuilder::new();
            builder.add_line(line1);
            let rules = builder.build().unwrap().0;
            assert_eq!(rules.len(), 1);
            assert_eq!(rules.get(&1), Some(&Rc::new(Rule::Literal('a'))));

            let mut builder = RulesBuilder::new();
            builder.add_line(line0);
            builder.add_line(line1);
            builder.add_line(line2);
            builder.add_line(line3);
            let rules = builder.build().unwrap().0;
            assert_eq!(rules.len(), 4);
            let rule1 = Rc::new(Rule::Literal('a'));
            let rule3 = Rc::new(Rule::Literal('b'));
            let rule2 = Rc::new(Rule::Or(vec!(Rc::clone(&rule1), Rc::clone(&rule3)), vec!(Rc::clone(&rule3), Rc::clone(&rule1))));
            let rule0 = Rc::new(Rule::Just(vec!(Rc::clone(&rule1), Rc::clone(&rule2))));

            assert_eq!(rules.get(&1), Some(&rule1));
            assert_eq!(rules.get(&3), Some(&rule3));
            assert_eq!(rules.get(&2), Some(&rule2));
            assert_eq!(rules.get(&0), Some(&rule0));

            let line0 = "0: 4 1 5";
            let line1 = "1: 2 3 | 3 2";
            let line2 = "2: 4 4 | 5 5";
            let line3 = "3: 4 5 | 5 4";
            let line4 = "4: \"a\"";
            let line5 = "5: \"b\"";

            let mut builder = RulesBuilder::new();
            for line in vec!(line0, line1, line2, line3, line4, line5) {
                builder.add_line(line);
            }
            let rules = builder.build().unwrap().0;
            assert_eq!(rules.len(), 6);
        }
    }

    mod rule {
        use super::*;

        #[test]
        fn partial_match_test() {
            let rule_a = Rc::new(Rule::Literal('a'));
            assert_eq!(rule_a.partial_match("a"), Some(""));
            assert_eq!(rule_a.partial_match("abc"), Some("bc"));
            assert_eq!(rule_a.partial_match("bc"), None);

            let rule_b = Rc::new(Rule::Literal('b'));
            let rule_ab = Rule::Just(vec!(Rc::clone(&rule_a), Rc::clone(&rule_b)));
            assert_eq!(rule_ab.partial_match("a"), None);
            assert_eq!(rule_ab.partial_match("ab"), Some(""));
            assert_eq!(rule_ab.partial_match("abc"), Some("c"));
            assert_eq!(rule_ab.partial_match("bc"), None);

            let rule_a_or_b = Rule::Or(
                vec!(Rc::clone(&rule_a)),
                vec!(Rc::clone(&rule_b))
            );
            assert_eq!(rule_a_or_b.partial_match("a"), Some(""));
            assert_eq!(rule_a_or_b.partial_match("b"), Some(""));
            assert_eq!(rule_a_or_b.partial_match("ab"), Some("b"));
            assert_eq!(rule_a_or_b.partial_match("ba"), Some("a"));
            assert_eq!(rule_a_or_b.partial_match("c"), None);
            assert_eq!(rule_a_or_b.partial_match("cba"), None);
        }

        #[test]
        fn total_match_test() {
            let rule5 = Rc::new(Rule::Literal('b'));
            let rule4 = Rc::new(Rule::Literal('a'));
            let rule3 = Rc::new(Rule::Or(
                vec!(Rc::clone(&rule4), Rc::clone(&rule5)),
                vec!(Rc::clone(&rule5), Rc::clone(&rule4))
            ));
            let rule2 = Rc::new(Rule::Or(
                vec!(Rc::clone(&rule4), Rc::clone(&rule4)),
                vec!(Rc::clone(&rule5), Rc::clone(&rule5))
            ));
            let rule1 = Rc::new(Rule::Or(
                vec!(Rc::clone(&rule2), Rc::clone(&rule3)),
                vec!(Rc::clone(&rule3), Rc::clone(&rule2))
            ));
            let rule0 = Rc::new(Rule::Just(
                vec!(Rc::clone(&rule4), Rc::clone(&rule1), Rc::clone(&rule5))
            ));
            assert_eq!(rule0.total_match("ababbb"), true);
            assert_eq!(rule0.total_match("abbbab"), true);
            assert_eq!(rule0.total_match("bababa"), false);
            assert_eq!(rule0.total_match("aaabbb"), false);
            assert_eq!(rule0.total_match("aaaabbb"), false);

        }
    }

    #[test]
    fn recursive_rule_test() {
        let rule_lines = vec!(
            "42: 9 14 | 10 1",
            "9: 14 27 | 1 26",
            "10: 23 14 | 28 1",
            "1: \"a\"",
            "11: 42 31",
            "5: 1 14 | 15 1",
            "19: 14 1 | 14 14",
            "12: 24 14 | 19 1",
            "16: 15 1 | 14 14",
            "31: 14 17 | 1 13",
            "6: 14 14 | 1 14",
            "2: 1 24 | 14 4",
            "0: 8 11",
            "13: 14 3 | 1 12",
            "15: 1 | 14",
            "17: 14 2 | 1 7",
            "23: 25 1 | 22 14",
            "28: 16 1",
            "4: 1 1",
            "20: 14 14 | 1 15",
            "3: 5 14 | 16 1",
            "27: 1 6 | 14 18",
            "14: \"b\"",
            "21: 14 1 | 1 14",
            "25: 1 1 | 1 14",
            "22: 14 14",
            "8: 42",
            "26: 14 22 | 1 20",
            "18: 15 15",
            "7: 14 5 | 1 21",
            "24: 14 1",
        );
        let mut builder = RulesBuilder::new();

        for line in rule_lines {
            builder.add_line(line);
        }

        let rules = builder.build().unwrap();
        let rule0 = rules.0.get(&0).unwrap();
        let rule42 = rules.0.get(&42).unwrap();
        let rule31 = rules.0.get(&31).unwrap();
        let rule0_recursive = Rule::Rep(Rc::clone(&rule42), Rc::clone(&rule31));


        let mut m0 = 0;
        let mut m1 = 0;
        for msg in vec!(
            "abbbbbabbbaaaababbaabbbbabababbbabbbbbbabaaaa",
            "bbabbbbaabaabba",
            "babbbbaabbbbbabbbbbbaabaaabaaa",
            "aaabbbbbbaaaabaababaabababbabaaabbababababaaa",
            "bbbbbbbaaaabbbbaaabbabaaa",
            "bbbababbbbaaaaaaaabbababaaababaabab",
            "ababaaaaaabaaab",
            "ababaaaaabbbaba",
            "baabbaaaabbaaaababbaababb",
            "abbbbabbbbaaaababbbbbbaaaababb",
            "aaaaabbaabaaaaababaa",
            "aaaabbaaaabbaaa",
            "aaaabbaabbaaaaaaabbbabbbaaabbaabaaa",
            "babaaabbbaaabaababbaabababaaab",
            "aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba",
        ) {
            if rule0.total_match(msg) {
                m0 += 1;
            }
            if rule0_recursive.total_match(msg) {
                m1 += 1;
            }
        }
        assert_eq!(m0, 3);
        assert_eq!(m1, 12);
    }
}