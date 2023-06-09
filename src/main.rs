extern crate yaml_rust;
use baking_formula::dough;
use baking_formula::dough::DoughFormula;
use std::fs;

// ideas
// 1. use mongodb to contain formulas
// 2. allow for other types of input
// 3. create simple front-end
// 4. the data structure for Formula can be built recursively

fn main() {
    // read in data
    let formula: DoughFormula =
        dough::yaml_to_dough_formula("./test_valid_1.yaml".to_string());
    // println!("{:#?}", formula);
}
