use std::io::prelude::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[macro_use]
extern crate lazy_static;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Ingredient(String);

impl Ingredient {
    fn new(s: &str) -> Ingredient {
        Ingredient(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Allergen(String);

impl Allergen {
    fn new(s: &str) -> Allergen {
        Allergen(s.to_string())
    }
}

struct Food {
    ingredients: BTreeSet<Ingredient>,
    allergens: BTreeSet<Allergen>
}

impl Food {
    fn parse(line: &str) -> Option<Food> {
        lazy_static!{
            static ref FOOD_PAT: Regex = Regex::new(r"(.*) \(contains (.*)\)").unwrap();
            static ref WS_PAT: Regex = Regex::new(r",?\s+").unwrap();
        }

        FOOD_PAT.captures(line).map(|caps| {
            let ingredients = WS_PAT.split(&caps[1]).map(|s| Ingredient::new(s)).collect();
            let allergens = WS_PAT.split(&caps[2]).map(|s| Allergen::new(s)).collect();
            Food { ingredients, allergens }
        })

    }
}

struct AllergenCandidates {
    cs: BTreeMap<Allergen, AllergenSource>
}

impl AllergenCandidates {
    fn new() -> AllergenCandidates {
        let cs = BTreeMap::new();
        AllergenCandidates{ cs }
    }

    fn add_food(&mut self, food: &Food) -> Result<(), String> {
        // newly committed ingredients to be cleared out of other candidates
        let mut queue: VecDeque<Ingredient> = VecDeque::new();
        let mut r = Ok(());
        for allergen in &food.allergens {
            if let Err(_) = r {
                return r
            }
            self.cs.entry(allergen.clone())
            .and_modify(|src| {
                match src {
                    AllergenSource::Maybe(ingrs) => {
                        let next_ingrs: BTreeSet<Ingredient> = ingrs.intersection(&food.ingredients)
                            .map(|j| j.clone())
                            .collect();
                        match next_ingrs.len() {
                            0 => {
                                let msg = format!("No remaining candidates for allergen {:?}", allergen);
                                r = Err(msg);
                            },
                            1 => {
                                let tox = next_ingrs.iter().nth(0).unwrap();
                                *src = AllergenSource::Definitely(tox.clone());
                                queue.push_back(tox.clone());
                            },
                            _ => *ingrs = next_ingrs
                        }

                    },
                    AllergenSource::Definitely(ingr) if !&food.ingredients.contains(ingr) => {
                        let msg = format!("Allergen {:?} previously identified as {:?} not contained in ingredient list {:?}",
                            allergen, ingr, food.ingredients);
                        r = Err(msg)
                    }
                    _ => ()
                }
            }).or_insert({
                AllergenSource::Maybe(food.ingredients.clone())
            });
        };

        while let Some(j) = queue.pop_front() {
            for (allergen, src) in self.cs.iter_mut() {
                if let AllergenSource::Maybe(ingrs) = src {
                    ingrs.remove(&j);
                    match ingrs.len() {
                        0 => {
                            let msg = format!("No ingredient left matches allergen {:?}", allergen);
                            return Err(msg)
                        },
                        1 => {
                            let tox = ingrs.iter().nth(0).unwrap();
                            queue.push_back(tox.clone());
                            *src = AllergenSource::Definitely(tox.clone());
                        },
                        _ => ()
                    }
                }
            }
        }
        r
    }

    // Returns all ingredients in the input which are neither definitely nor maybe the source of an allergen
    fn safe_ingredients<'a>(&self, ingredients: &BTreeSet<&'a Ingredient>) -> BTreeSet<&'a Ingredient> {
        let mut ingredients = ingredients.clone();
        for src in self.cs.values() {
            match src {
                AllergenSource::Definitely(ingr) => {
                    ingredients.remove(ingr);
                },
                AllergenSource::Maybe(ingrs) => {
                    for ingr in ingrs {
                        ingredients.remove(ingr);
                    }
                }
            }
        };
        ingredients
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AllergenSource {
    Definitely(Ingredient),
    Maybe(BTreeSet<Ingredient>)
}

fn main() {
    let stdin = std::io::stdin();

    let mut occurrences: BTreeMap<Ingredient, usize> = BTreeMap::new();
    let mut foods: Vec<Food> = vec!();
    let mut allergen_sources = AllergenCandidates::new();

    for line in stdin.lock().lines().flatten() {
        let food = Food::parse(&line).unwrap();
        for ingredient in &food.ingredients {
            *occurrences.entry(ingredient.clone()).or_insert(0) += 1;
        }
        allergen_sources.add_food(&food).unwrap();
        foods.push(food)
    }

    let all_ingredients: BTreeSet<&Ingredient> = occurrences.keys().collect();
    let hypoallergenics = allergen_sources.safe_ingredients(&all_ingredients);
    let hypoallergenic_count: usize = hypoallergenics.iter().flat_map(|j| occurrences.get(j)).sum();
    println!("{} hypoallergenic ingredients identified, with {} total usages.", hypoallergenics.len(), hypoallergenic_count);

    let canonical_dangerous_ingredient_list: String = allergen_sources.cs.iter()
    .fold(String::new(), |mut acc, (allergen, src)| {
        match src {
            AllergenSource::Definitely(ingredient) => {
                if acc.is_empty() {
                    acc.push_str(&ingredient.0);
                    acc
                } else {
                    acc.push(',');
                    acc.push_str(&ingredient.0);
                    acc
                }
            },
            _ => panic!("Unresolved allergen {:?}", allergen)
        }
    });
    println!("Canonical dangerous ingredient list:\n{}", canonical_dangerous_ingredient_list);
}

#[cfg(test)]
mod day21_spec {
    use super::*;

    fn into_set<T: Ord, F>(ws: Vec<&str>, f: F) -> BTreeSet<T> where F: Fn(&str) -> T {
        ws.iter().map(|s| f(s)).collect()
    }

    #[test]
    fn parse_test() {
        let line = "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)";
        let food = Food::parse(line).unwrap();

        assert_eq!(food.ingredients, into_set(vec!(
            "mxmxvkd", "kfcds", "sqjhc", "nhms"
        ), Ingredient::new));
        assert_eq!(food.allergens, into_set(vec!(
            "dairy", "fish"
        ), Allergen::new));

        let line = "trh fvjkl sbzzf mxmxvkd (contains dairy)";
        let food = Food::parse(line).unwrap();

        assert_eq!(food.ingredients, into_set(vec!(
            "trh", "fvjkl", "sbzzf", "mxmxvkd"
        ), Ingredient::new));
        assert_eq!(food.allergens, into_set(vec!("dairy"), Allergen::new));
    }

    #[test]
    fn add_food_test() {
        let mut allergen_sources = AllergenCandidates::new();
        let food = Food::parse("mxmxvkd kfcds sqjhc nhms (contains dairy, fish)").unwrap();
        let dairy = Allergen::new("dairy");
        let fish = Allergen::new("fish");
        allergen_sources.add_food(&food).unwrap();

        assert_eq!(allergen_sources.cs.len(), 2);
        let expected_src = {
            let ingredients = into_set(vec!("mxmxvkd", "kfcds", "sqjhc", "nhms"), Ingredient::new);
            AllergenSource::Maybe(ingredients)
        };
        assert_eq!(allergen_sources.cs.get(&dairy), Some(&expected_src));
        assert_eq!(allergen_sources.cs.get(&&fish), Some(&expected_src));

        let food = Food::parse("trh fvjkl sbzzf mxmxvkd (contains dairy)").unwrap();
        allergen_sources.add_food(&food).unwrap();
        assert_eq!(allergen_sources.cs.get(&dairy), Some(&AllergenSource::Definitely(Ingredient::new("mxmxvkd"))));
        let expected_src = AllergenSource::Maybe(into_set(vec!("kfcds", "sqjhc", "nhms"), Ingredient::new));
        assert_eq!(allergen_sources.cs.get(&fish), Some(&expected_src));

        let food = Food::parse("sqjhc fvjkl (contains soy)").unwrap();
        allergen_sources.add_food(&food).unwrap();
        let soy = Allergen::new("soy");
        assert_eq!(allergen_sources.cs.len(), 3);
        let expected_src = AllergenSource::Maybe(into_set(vec!("sqjhc", "fvjkl"), Ingredient::new));
        assert_eq!(allergen_sources.cs.get(&soy), Some(&expected_src));

        let food = Food::parse("sqjhc mxmxvkd sbzzf (contains fish)").unwrap();
        allergen_sources.add_food(&food).unwrap();
        assert_eq!(allergen_sources.cs.get(&soy), Some(&AllergenSource::Definitely(Ingredient::new("fvjkl"))));
        assert_eq!(allergen_sources.cs.get(&fish), Some(&AllergenSource::Definitely(Ingredient::new("sqjhc"))));

        // TODO: there is a subtle bug in this implementation, where if an ingredient has already
        // been committed, new allergens will not eliminate it from their candidate list
    }

    #[test]
    fn safe_ingredients_test() {
        let allergen_sources = {
            let mut cs = BTreeMap::new();
            cs.insert(Allergen::new("peanut"), AllergenSource::Definitely(Ingredient::new("sqjhc")));
            cs.insert(Allergen::new("gluten"), AllergenSource::Definitely(Ingredient::new("fvjkl")));
            let maybe_garlic = into_set(vec!("aaa", "bbb"), Ingredient::new);
            cs.insert(Allergen::new("garlic"), AllergenSource::Maybe(maybe_garlic));
            AllergenCandidates { cs }
        };
        let all_ingredients = into_set(vec!(
            "mxmxvkd", "kfcds", "sqjhc", "nhms",
            "trh", "fvjkl", "sbzzf", "mxmxvkd",
            "sqjhc", "fvjkl",
            "sqjhc", "mxmxvkd", "sbzzf"
        ), Ingredient::new);
        let safe_ingredients = all_ingredients.iter().filter(|j| {
            j.0 != "sqjhc" && j.0 != "fvjkl"
        }).collect();

        assert_eq!(
            allergen_sources.safe_ingredients(&all_ingredients.iter().collect()),
            safe_ingredients
        );
    }
}