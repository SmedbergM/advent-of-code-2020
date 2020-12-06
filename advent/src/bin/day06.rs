use std::io;
use std::io::prelude::*;

use std::collections::BTreeSet;

fn count_group_questions<J>(lines: &mut J) -> (usize, usize)
where J: Iterator<Item=String> {
    let mut t_any = 0; // total number of questions answered yes by ANY group member
    let mut t_all = 0;
    let mut qs_any: BTreeSet<char> = BTreeSet::new();

    enum QsAll {
        Nil, // When we finish with a group
        P(BTreeSet<char>)
    }

    impl QsAll {
        fn len(&self) -> usize {
            match self {
                QsAll::Nil => 0,
                QsAll::P(qs) => qs.len()
            }
        }
    }

    let mut qs_all = QsAll::Nil;

    loop {
        match lines.next() {
            None => {
                t_any += qs_any.len();
                t_all += qs_all.len();
                break
            },
            Some(line) if line.is_empty() => {
                t_any += qs_any.len();
                t_all += qs_all.len();
                qs_any.clear();
                qs_all = QsAll::Nil;
            },
            Some(line) => {
                qs_any.extend(line.chars());
                match &mut qs_all {
                    QsAll::Nil => {
                        qs_all = QsAll::P(line.chars().collect());
                    },
                    QsAll::P(ref mut qs) => {
                        let next_qs = qs.intersection(&line.chars().collect()).map(|c| *c).collect();
                        *qs = next_qs;
                    }
                }
            }
        }
    }

    (t_any, t_all)
}

fn main() {
    let stdin = io::stdin();
    let (q_any, q_all) = count_group_questions(&mut stdin.lock().lines().flatten());
    println!("Total questions answered yes by ANY group member: {}", q_any);
    println!("Total questions answered yes by ALL group members: {}", q_all);
}

#[cfg(test)]
mod day06_spec {
    use super::*;

    const TEST_INPUT: &str = 
    "abc\n\
    \n\
    a\n\
    b\n\
    c\n\
    \n\
    ab\n\
    ac\n\
    \n\
    a\n\
    a\n\
    a\n\
    a\n\
    \n\
    b\n";

    #[test]
    fn question_count_test() {
        assert_eq!(count_group_questions(&mut TEST_INPUT.lines().map(|s| s.to_owned())), (11, 6))    ;
    }
}