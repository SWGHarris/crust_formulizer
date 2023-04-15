extern crate yaml_rust;
use std::{collections::{HashMap, HashSet}};
use yaml_rust::{YamlLoader};
use crate::csv_cell::{CSVCell, self};

#[derive(Debug)]
// TODO: refactor struct to be an enum for clarity
// Flour(String, f64)
// NonFlour(String, f64)
struct ComponentIngredient(String, f64, bool);
impl ComponentIngredient {
    fn to_percentage(&mut self, tot_flour: f64) {
        self.1 = self.1 / tot_flour;
    }
}

// DoughComponent struct is composed of ingredients. Ingredient name's may
// reference other components. All components must be referenced by another segment
// except for the final segment (always named "mix"), which may not be
// referenced by any other segment.
#[derive(Debug)]
struct DoughComponent {
    name: String,
    csv_col_index: Option<u32>,
    tot_flour: f64,
    tot_percent: f64,
    ingredients: HashMap<String, ComponentIngredient>
}


#[derive(Debug)]
pub struct DoughFormula {
    name: String,
    tot_flour: csv_cell::CSVCell, // represents total flour (as percentage)
    components: HashMap<String, DoughComponent>,
    flour: HashSet<String>,
    non_flour: HashSet<String>,
}


// impl DoughFormula {
//     fn calc_tot_flour(&mut self) {
//         let mix = self.components.get("mix").expect("Formula must include mix");
//         calc_tot_flour_rec(mix, String::from(""));
//     }
// }


pub fn yaml_to_dough_formula(filename: String) -> DoughFormula {
    let docs = YamlLoader::load_from_str(&filename).unwrap();
    let doc = &docs[0];
    let formula_name = doc["name"].as_str().unwrap().to_string();
    let mut formula = DoughFormula { 
        name: formula_name,
        tot_flour: CSVCell { expr: (), row: (), col: () },
        components: HashMap::new(),
        flour: HashSet::new(),
        non_flour: HashSet::new()
    };

    // convert yaml to struct DoughFormula
    for (_, s) in doc["components"].as_vec().unwrap().iter().enumerate() {
        let name = s["name"].as_str().unwrap().to_string();
        let mut segment: DoughComponent = DoughComponent {
            name: name.clone(),
            csv_col_index: None,
            tot_flour: 0.0,
            tot_percent: 0.0,
            ingredients: HashMap::new()
        };

        for ing in s["ingredients"].as_vec().unwrap() {
            let name     = ing[0].as_str().unwrap().to_string();
            let mass     = ing[1].as_f64().unwrap();
            let is_flour = ing[2].as_bool().unwrap();
            if is_flour {
                formula.flour.insert(name.clone());
                segment.tot_flour += mass
            }
            else {
                formula.non_flour.insert(name.clone());
            }
            segment.ingredients.insert(name.clone(), ComponentIngredient(name, mass, is_flour));
        }
        formula.components.insert(name, segment);
    }

     // convert to percentages if needed
     let by_percent = doc["by_percent"].as_bool().unwrap();
     for (_,seg) in formula.components.iter_mut() {
         for (_,ing) in seg.ingredients.iter_mut() {
             if !by_percent { ing.to_percentage(seg.tot_flour); }
             seg.tot_percent += ing.1;
         }
     }

    // build columns representing percentages
    let mut cols: Vec<Vec<String>> = Vec::new();
    let mut ingredients: Vec<String> = formula.flour.clone().into_iter().collect();
    let mut non_flour: Vec<String> = formula.non_flour.clone().into_iter().collect();
    ingredients.sort();
    non_flour.sort();
    ingredients.append(&mut non_flour);
    for (j,(name, seg)) in formula.components.iter_mut().enumerate() {
        // initialize new column
        let mut seg_col: Vec<String> = Vec::with_capacity(ingredients.len() + 4); 
        seg_col.push(name.clone());
        seg_col.push(String::from("%"));
        // assign column index
        seg.csv_col_index = Some(j as u32);
        for (_, ing_name) in ingredients.iter().enumerate() {
            if seg.ingredients.contains_key(ing_name) {
                let p = seg.ingredients[ing_name].1.to_string();
                seg_col.push(p);
            }
        }
        cols.push(seg_col.clone());
    }

    // determine term for flour contribution
    let col_0 = formula.components.len()*2 + 2;
    let col_1 = formula.components.len()*2 + 3;
    let row_0 = 2;
    let row_n = row_0 + ingredients.len();
    let flour_term = csv_sumproduct_cells(row_0 as u32, row_n as u32, col_0 as u32, col_1 as u32);

    formula
}



// takes row, col as u32 and return cell label
// if fix_cell then returns fixed value (e.g. $H$7))
// takes u32 and returns the column label
// ex: 4 -> D
// ex: (4 + 26) -> DD
pub fn to_csv_col(col: u32) -> String {
    let c: u8 = b'A' + ((col % 26) as u8);
    let n = (col / 26) + 1;
    let mut col_label = String::from("");
    for _ in 0..n {
        let c_str = (c as char).to_string();
        col_label.push_str(&c_str);
     }
     col_label
}


// takes row, col as u32 and return cell label
// if fix_cell then returns fixed value (e.g. $H$7))
fn to_csv_cell(row: u32, col: u32, fix_cell: bool) -> String {
    let row_str = (row + 1).to_string();
    let mut result: String = Default::default();
    let fix = String::from("$");
    if fix_cell {
        result.push_str(&fix);
        result.push_str(&to_csv_col(col));
        result.push_str(&fix);
        result.push_str(&row_str);
    } else {
        result.push_str(&to_csv_col(col));
        result.push_str(&row_str);
    }
    result
}

// upper limit is exclusive bound
// ex: to_range(3, 10, 5) =>
pub fn to_csv_range(from_row: u32, to_row: u32, col: u32) -> String {
    let mut result = to_csv_cell(from_row, col, true);
    result.push_str(&String::from(":"));
    result.push_str(&to_csv_cell(to_row - 1, col, true));
    result
}

// takes col and range and returns formatted String
// to_row is exclusive
// ex: sum_cells(3, 10, 5) => SUM(D3:D9)
pub fn csv_sum_cells(from_row: u32, to_row: u32, col: u32) -> String {
    let mut result = String::from("SUM(");
    result.push_str(&to_csv_range(from_row, to_row, col));
    result.push_str(&String::from(")"));
    result
}

pub fn csv_sumproduct_cells(from_row: u32, to_row: u32, col_a: u32, col_b: u32) -> String {
    let mut result = String::from("SUMPRODUCT(");
    result.push_str(&to_csv_range(from_row, to_row, col_a));
    result.push_str(&String::from(","));
    result.push_str(&to_csv_range(from_row, to_row, col_b));
    result.push_str(&String::from(")"));
    result
}