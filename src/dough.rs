extern crate yaml_rust;
use std::{collections::{HashMap, HashSet}};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use yaml_rust::{YamlLoader};
use crate::csv_cell::{CSVCell, self};

#[derive(Debug)]
enum Ingredient {
    Flour(Decimal),
    NonFlour(Decimal)
}

// DoughComponent struct is composed of ingredients. Ingredient name's may
// reference other components. All components must be referenced by another segment
// except for the final segment (always named "mix"), which may not be
// referenced by any other segment.
#[derive(Debug)]
struct DoughComponent {
    name: String,
    ingredients: HashMap<String, Ingredient>
}

#[derive(Debug)]
pub struct DoughFormula {
    name: String,
    components: HashMap<String, DoughComponent>,
    flour: HashSet<String>,
    non_flour: HashSet<String>,
}

impl DoughFormula {
    fn find_tot_flour
}

pub fn yaml_to_dough_formula(filename: String) -> DoughFormula {
    let docs = YamlLoader::load_from_str(&filename).unwrap();
    let doc = &docs[0];
    
    // calculate flour totals for each segment
    let mut component_flour: HashMap<String, Decimal> = HashMap::new();
    for s in doc["components"].as_vec().unwrap() {
        let name = s["name"].as_str().unwrap().to_string();
        for ing in s["ingredients"].as_vec().unwrap() {
            let is_flour = ing[2].as_bool().unwrap();
            if is_flour {
                let mass: Decimal = Decimal::from_f64(ing[1].as_f64().unwrap()).unwrap();
                if !component_flour.contains_key(&name) { component_flour.insert(name.clone(), dec!(0)); }
                let new_component_flour = component_flour[&name] + mass;
                component_flour.insert(name.clone(), new_component_flour);
            }
        }
    }

    // initialize struct
    let formula_name = doc["name"].as_str().unwrap().to_string();
    let mut formula = DoughFormula {
        name: formula_name,
        components: HashMap::new(),
        flour: HashSet::new(),
        non_flour: HashSet::new()
    };

    // convert yaml to struct DoughFormula
    for (_, s) in doc["components"].as_vec().unwrap().iter().enumerate() {
        let seg_name = s["name"].as_str().unwrap().to_string();
        let mut seg: DoughComponent = DoughComponent {
            name: seg_name.clone(),
            ingredients: HashMap::new()
        };

        for ing in s["ingredients"].as_vec().unwrap() {
            let ing_name   = ing[0].as_str().unwrap().to_string();
            let mass      = ing[1].as_f64().unwrap();
            let is_flour = ing[2].as_bool().unwrap();
            let percentage = Decimal::from_f64(mass).unwrap() / component_flour[&seg_name];
            let new_ing: Ingredient;
            if is_flour {
                formula.flour.insert(ing_name.clone());
                new_ing = Ingredient::Flour(percentage);
            } else {
                formula.non_flour.insert(ing_name.clone());
                new_ing = Ingredient::NonFlour(percentage);
            }
            seg.ingredients.insert(ing_name,new_ing);
        }
        formula.components.insert(seg_name, seg);
    }

    // build columns representing percentages
    // let mut cols: Vec<Vec<String>> = Vec::new();
    // let mut ingredients: Vec<String> = formula.flour.clone().into_iter().collect();
    // let mut non_flour: Vec<String> = formula.non_flour.clone().into_iter().collect();
    // ingredients.sort();
    // non_flour.sort();
    // ingredients.append(&mut non_flour);
    // for (j,(name, seg)) in formula.components.iter_mut().enumerate() {
    //     // initialize new column
    //     let mut seg_col: Vec<String> = Vec::with_capacity(ingredients.len() + 4); 
    //     seg_col.push(name.clone());
    //     seg_col.push(String::from("%"));
    //     // assign column index
    //     seg.csv_col_index = Some(j as u32);
    //     for (_, ing_name) in ingredients.iter().enumerate() {
    //         if seg.ingredients.contains_key(ing_name) {
    //             let p = seg.ingredients[ing_name].1.to_string();
    //             seg_col.push(p);
    //         }
    //     }
    //     cols.push(seg_col.clone());
    // }

    // determine term for flour contribution
    // let col_0 = formula.components.len()*2 + 2;
    // let col_1 = formula.components.len()*2 + 3;
    // let row_0 = 2;
    // let row_n = row_0 + ingredients.len();
    // let flour_term = csv_sumproduct_cells(row_0 as u32, row_n as u32, col_0 as u32, col_1 as u32);

    formula
}



