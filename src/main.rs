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
    let filename: String = fs::read_to_string("./test.yaml").expect("Unable to read file");

    // convert yaml to structs
    let formula: DoughFormula = dough::yaml_to_dough_formula(filename);
    println!("{:?}", formula);
//    ingredients.insert(0, String::from("ingredient")); 
//    cols.insert(0, ingredients.clone());
//
//    for (j, col) in cols.enumerate() {
//        if j % 2 == 0 && j != 0 {
//            let mut mass_col: Vec<String> = Vec::with_capacity(ingredients.len() + 4);
//            mass_col.append(&mut Vec::from([String::from(""), String::from("grams")]));
//            for (i, cell) in col.enumerate() {
//                let mut func: String = String::from("=");
//                func.push(&to_cell(i, j - 1, false));
//                func.push(&String::from("/"));
//                func.push(&to_cell(ingredients.len() + 2, j - 1, true));
//                func.push(&String::from("*"));
//
//            }
//        }
//    }
//

}
