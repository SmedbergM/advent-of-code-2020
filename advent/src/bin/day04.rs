#[macro_use]
extern crate lazy_static;

use std::io;
use std::io::prelude::*;

use regex::Regex;
use std::collections::BTreeMap;

fn validate_passport_keys(p: &BTreeMap<String, String>) -> bool {
    p.contains_key("byr") &&
        p.contains_key("iyr") &&
        p.contains_key("eyr") &&
        p.contains_key("hgt") &&
        p.contains_key("hcl") &&
        p.contains_key("ecl") &&
        p.contains_key("pid")
}

fn validate_passport_values(p: &BTreeMap<String, String>) -> bool {
    
    let byr_valid = p.get("byr")
    .and_then(|byr| usize::from_str_radix(byr, 10).ok())
    .map_or(false, |byr| 1920 <= byr && byr <= 2002);

    let iyr_valid: bool = p.get("iyr")
    .and_then(|iyr| usize::from_str_radix(iyr, 10).ok())
    .map_or(false, |iyr| 2010 <= iyr && iyr <= 2020);

    let eyr_valid: bool = p.get("eyr")
    .and_then(|eyr| usize::from_str_radix(eyr, 10).ok())
    .map_or(false, |eyr| 2020 <= eyr && eyr <= 2030);

    lazy_static! {
        static ref HGT_PAT: Regex = Regex::new(r"(\d+)(cm|in)").unwrap();
        static ref HCL_PAT: Regex = Regex::new(r"#[0-9a-f]{6}").unwrap();
    }

    enum Height {
        In(usize),
        Cm(usize)
    }

    impl Height {
        fn is_valid(&self) -> bool {
            match self {
                Height::Cm(h) => 150 <= *h && *h <= 193,
                Height::In(h) => 59 <= *h && *h <= 76        
            }
        }
    }

    let hgt_valid: bool = p.get("hgt")
    .and_then(|hgt| HGT_PAT.captures(hgt))
    .and_then(|caps| match &caps[2] {
        "cm" => usize::from_str_radix(&caps[1], 10).ok().map(|h| Height::Cm(h)),
        "in" => usize::from_str_radix(&caps[1], 10).ok().map(|h| Height::In(h)),
        _ => None
    }).map_or(false, |hgt| hgt.is_valid());

    let hcl_valid: bool = p.get("hcl").map_or(false, |hcl| HCL_PAT.is_match(hcl));

    let ecl_valid: bool = p.get("ecl").map_or(false, |ecl| match ecl.as_str() {
        "amb" | "blu" | "brn" | "gry" | "grn" | "hzl" | "oth" => true,
        _ => false
    });

    let pid_valid: bool = p.get("pid")
    .filter(|pid| pid.len() == 9)
    .and_then(|pid| usize::from_str_radix(pid, 10).ok())
    .is_some();

    byr_valid && iyr_valid && eyr_valid && hgt_valid && hcl_valid && ecl_valid && pid_valid
}


struct MapStream<J: Iterator<Item=String>> {
    lines: J
}

impl<J: Iterator<Item=String>> Iterator for MapStream<J> {
    type Item = BTreeMap<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        fn extract_pairs(line: String) -> BTreeMap<String, String> {
            lazy_static! {
                static ref KV_PAT: Regex = Regex::new(r"(\w{3}):([\w#]+)").unwrap();
            }
            KV_PAT.captures_iter(&line).map(|cap| {
                (cap[1].to_owned(), cap[2].to_owned())
            }).collect()
        }

        let mut p = BTreeMap::new();

        loop {
            match self.lines.next() {
                Some(line) if line.is_empty() && !p.is_empty() => {
                    return Some(p)
                },
                Some(line) => {
                    p.extend(extract_pairs(line))
                },
                None => {
                    return Some(p).filter(|m| !m.is_empty())
                }
            }
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let map_stream = MapStream {
        lines: stdin.lock().lines().flatten()
    };
    let (total_passports, correct_keys, valid_values) = {
        let (mut t, mut k, mut v) = (0, 0, 0);
        for p in map_stream {
            t += 1;
            k += validate_passport_keys(&p) as usize;
            v += validate_passport_values(&p) as usize;
        }
        (t, k, v)
    };
    println!("Total passports: {}. Correct keys: {}; valid values: {}", total_passports, correct_keys, valid_values);
}

#[cfg(test)]
mod day04_spec {
    use super::*;

    fn vec_to_map(v: Vec<(&str, &str)>) -> BTreeMap<String, String> {
        v.iter().map(|p| (p.0.to_owned(), p.1.to_owned())).collect()
    }

    #[test]
    fn parse_test_1() {
        let input = "ecl:gry pid:860033327 eyr:2020 hcl:#fffffd\n\
        byr:1937 iyr:2017 cid:147 hgt:183cm";

        let expected: BTreeMap<String, String> = vec_to_map(vec!(
            ("ecl", "gry"),
            ("pid", "860033327"),
            ("eyr", "2020"),
            ("hcl", "#fffffd"),
            ("byr", "1937"),
            ("iyr", "2017"),
            ("cid", "147"),
            ("hgt", "183cm")
        ));
        
        let mut stream = MapStream { lines: input.lines().map(|s| s.to_owned()) };
        assert_eq!(stream.next(), Some(expected));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn parse_test_2() {
        let input = "iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884\n\
        hcl:#cfa07d byr:1929\n\
        \n\
        hcl:#ae17e1 iyr:2013\n\
        eyr:2024\n\
        ecl:brn pid:760753108 byr:1931\n\
        hgt:179cm\n\
        \n\
        \n\
        hcl:#cfa07d eyr:2025 pid:166559648
        iyr:2011 ecl:brn hgt:59in";

        let expected0: BTreeMap<String, String> = vec_to_map(vec!(
            ("iyr","2013"),
            ("ecl","amb"),
            ("cid","350"),
            ("eyr","2023"),
            ("pid","028048884"),
            ("hcl","#cfa07d"),
            ("byr","1929")
        ));
        let expected1: BTreeMap<String, String> = vec_to_map(vec!(
            ("hcl","#ae17e1"),
            ("iyr","2013"),
            ("eyr","2024"),
            ("ecl","brn"),
            ("pid","760753108"),
            ("byr","1931"),
            ("hgt","179cm")
        ));
        let expected2: BTreeMap<String, String> = vec_to_map(vec!(
            ("hcl","#cfa07d"),
            ("eyr","2025"),
            ("pid","166559648"),
            ("iyr","2011"),
            ("ecl","brn"),
            ("hgt","59in"),
        ));

        let mut stream = MapStream { lines: input.lines().map(|s| s.to_owned()) };
        assert_eq!(stream.next(), Some(expected0));
        assert_eq!(stream.next(), Some(expected1));
        assert_eq!(stream.next(), Some(expected2));
        assert_eq!(stream.next(), None);
    }

    mod validate_passport_keys {
        use super::*;

        #[test]
        fn should_check_7_keys() {
            let p = vec_to_map(vec!(
                ("ecl", "gry"),
                ("pid", "860033327"),
                ("eyr", "2020"),
                ("hcl", "#fffffd"),
                ("byr", "1937"),
                ("iyr", "2017"),
                ("cid", "147"),
                ("hgt", "183cm")    
            ));
            assert!(validate_passport_keys(&p));

            let p = vec_to_map(vec!(
                ("iyr","2013"),
                ("ecl","amb"),
                ("cid","350"),
                ("eyr","2023"),
                ("pid","028048884"),
                ("hcl","#cfa07d"),
                ("byr","1929")
            ));
            assert!(!validate_passport_keys(&p));

            let p = vec_to_map(vec!(
                ("hcl","#ae17e1"),
                ("iyr","2013"),
                ("eyr","2024"),
                ("ecl","brn"),
                ("pid","760753108"),
                ("byr","1931"),
                ("hgt","179cm")
            ));
            assert!(validate_passport_keys(&p));

            let p = vec_to_map(vec!(
                ("hcl","#cfa07d"),
                ("eyr","2025"),
                ("pid","166559648"),
                ("iyr","2011"),
                ("ecl","brn"),
                ("hgt","59in"),
            ));
            assert!(!validate_passport_keys(&p));
        }
    }

    mod validate_passport_values {
        use super::*;

        #[test]
        fn should_pass_good_values() {
            let p = vec_to_map(vec!(
                ("pid","087499704"),
                ("hgt","74in"),
                ("ecl","grn"),
                ("iyr","2012"),
                ("eyr","2030"),
                ("byr","1980"),
                ("hcl","#623a2f"),
            ));
            assert!(validate_passport_values(&p));

            let p = vec_to_map(vec!(
                ("eyr","2029"),
                ("ecl","blu"),
                ("cid","129"),
                ("byr","1989"),
                ("iyr","2014"),
                ("pid","896056539"),
                ("hcl","#a97842"),
                ("hgt","165cm"),
            ));
            assert!(validate_passport_values(&p));

            let p = vec_to_map(vec!(
                ("hcl","#888785"),
                ("hgt","164cm"),
                ("byr","2001"),
                ("iyr","2015"),
                ("cid","88"),
                ("pid","545766238"),
                ("ecl","hzl"),
                ("eyr","2022"),
            ));
            assert!(validate_passport_values(&p));

            let p = vec_to_map(vec!(
                ("iyr","2010"),
                ("hgt","158cm"),
                ("hcl","#b6652a"),
                ("ecl","blu"),
                ("byr","1944"),
                ("eyr","2021"),
                ("pid","093154719"),
            ));
            assert!(validate_passport_values(&p));
        }

        #[test]
        fn should_fail_bad_values() {
            let p = vec_to_map(vec!(
                ("eyr","1972"),
                ("cid","100"),
                ("hcl","#18171d"),
                ("ecl","amb"),
                ("hgt","170"),
                ("pid","186cm"),
                ("iyr","2018"),
                ("byr","1926"),
            ));
            assert!(!validate_passport_values(&p));

            let p = vec_to_map(vec!(
                ("iyr","2019"),
                ("hcl","#602927"),
                ("eyr","1967"),
                ("hgt","170cm"),
                ("ecl","grn"),
                ("pid","012533040"),
                ("byr","1946"),
            ));
            assert!(!validate_passport_values(&p));

            let p = vec_to_map(vec!(
                ("hcl","dab227"),
                ("iyr","2012"),
                ("ecl","brn"),
                ("hgt","182cm"),
                ("pid","021572410"),
                ("eyr","2020"),
                ("byr","1992"),
                ("cid","277"),
            ));
            assert!(!validate_passport_values(&p));

            let p = vec_to_map(vec!(
                ("hgt","59cm"),
                ("ecl","zzz"),
                ("eyr","2038"),
                ("hcl","74454a"),
                ("iyr","2023"),
                ("pid","3556412378"),
                ("byr","2007"),
            ));
            assert!(!validate_passport_values(&p));
        }
    }
}
