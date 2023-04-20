extern crate yaml_rust;
use core::num;
use std::{collections::{HashMap, HashSet, VecDeque}, fs, path::Component};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use yaml_rust::{YamlLoader};
use crate::csv_cell::{CSVCell, self, CellPosition, CellArray, CellValue, CellExpr,};

const ROW_OFFSET: usize = 1;
const COL_OFFSET: usize = 2;
const MIX: &str = "mix";

#[derive(Debug)]
enum Ingredient {
    Flour(Decimal),
    NonFlour(Decimal)
}

// Ingredient name's may reference other components. 
// All components but "mix" must be referenced by another segment
// "mix" may not be referenced
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

pub fn yaml_to_dough_formula(filename: String) -> DoughFormula {
    let filename: String = fs::read_to_string(filename).expect("Unable to read file");
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


    // obtain ordering of components and ingredients
    let mut flour: Vec<String> = formula.flour.clone().into_iter().collect();
    let mut non_flour: Vec<String> = formula.non_flour.clone().into_iter().collect();
    flour.sort();
    non_flour.sort();
    non_flour.append(&mut flour);
    let ingredients = flour;
    let mut dup_check = ingredients.clone();
    dup_check.sort();
    if dup_check.len() != ingredients.len() {
        panic!("Duplicate component names are not allowed");
    }

    let components: Vec<String> = formula.components.keys().cloned().collect();
    if components.iter().filter(|&c| (*c == String::from(MIX))).count() != 1 {
        panic!("Must be exactly one component named 'mix'");
    }

    // dfs to check for cycle in component-ingredient graph rooted at 'mix'
    dfs_components(MIX, &formula.components, &mut HashSet::new(), &mut HashSet::new());
    let mut components = bfs_components(&formula.components);
    components.reverse();

    // let component_percentages = component_percentages(&components, ingredients.len());
    // println!("{:#?}", component_percentages);

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


// returns a tuple of CellPositions corresponding to ing_name's position in the table
fn ingredient_to_cell_pos(ing_name: &String,
                          comp_ordering: &Vec<String>,
                          ing_ordering: &Vec<String>) -> (CellPosition,CellPosition) {
    let ing_pos  = ing_ordering.iter().position(|name| *name == *ing_name).unwrap();
    let comp_pos = comp_ordering.iter().position(|name| *name == *ing_name).unwrap();
    let row = (ing_pos + ROW_OFFSET) as u32;
    let col = (comp_pos + COL_OFFSET) as u32;
    let percent_pos = CellPosition {row, col, fix_row: false, fix_col: false};
    let value_pos = CellPosition {row, col: col + 1, fix_row: false, fix_col: false};
    (percent_pos, value_pos)
}

// returns a HashMap<String, CSVCell> that maps component names to the
// CSVCell associated with the position and expression for the component's
// percentage total
fn component_percentages(comp_ordering: &Vec<String>,
                        num_ingredients: usize)
                        -> HashMap<String, CSVCell>{
    let mut result: HashMap<String, CSVCell> = HashMap::new();
    for (index, comp_name) in comp_ordering.iter().enumerate() {
        let total_position = CellPosition {
            row: (ROW_OFFSET + num_ingredients) as u32 + 1,
            col: (index + COL_OFFSET) as u32 + 1,
            fix_row: false,
            fix_col: false
        };
        let from = CellPosition {
            row: total_position.row - num_ingredients as u32,
            col: total_position.col, fix_row: false, fix_col: false };
        let to = CellPosition {
            row: total_position.row - 1 as u32,
            col: total_position.col, fix_row: false, fix_col: false };
        let sum_array = CellArray::new(from, to);
        let total_val = CellValue::Expr(CellExpr::Sum(sum_array));
        let total_cell = CSVCell { value: total_val, position: total_position };
        result.insert(comp_name.to_string(), total_cell);
    }
    result
}

// DFS on the component-ingredient graph
//  - use to obtain spreadsheet formula for cell that represents overall total flour
//  - will panic on finding cycle => invalid formula
//  - will panic if it does not visit all components
fn dfs_components(current: &str, 
                    components: &HashMap<String, DoughComponent>,
                    visited: &mut HashSet<String>,
                    on_path: &mut HashSet<String>) {
    visited.insert(current.to_string());
    on_path.insert(current.to_string());
    let comp = components.get(current).expect("dfs: cannot call on non-component");
    for (ing_name, ing) in &comp.ingredients {
        if components.contains_key(ing_name) {
            if on_path.contains(ing_name) {
                panic!("Component may not be self referencing (directly or indirectly)");
            } 
            dfs_components(ing_name, components, visited, on_path);
        } 
    }
    if current == MIX && components.len() != visited.len() {
        panic!("mix must reference all components directly or indirectly");
    }
    on_path.remove(current);
}

// BFS on the component-ingredient graph
//  - use to obtain ordering of components
fn bfs_components(components: &HashMap<String, DoughComponent>) -> Vec<String> {
    let mut comp_order: Vec<String> = Vec::new();
    let mut queue: VecDeque<&DoughComponent> = VecDeque::new();
    comp_order.push(MIX.to_string());
    queue.push_back(components.get(MIX).unwrap());
    while !queue.is_empty() {
        let comp = queue.pop_front().unwrap();
        for (ing_name, _) in &comp.ingredients {
            if components.contains_key(ing_name) && !comp_order.contains(ing_name) {
                let next_comp = &components[ing_name];
                comp_order.push(ing_name.to_string());
                queue.push_back(&next_comp);
            }
        }
    }
    comp_order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        yaml_to_dough_formula(String::from("./test_valid_1.yaml"));
        yaml_to_dough_formula(String::from("./test_valid_branches.yaml"));
    }

    #[test]
    #[should_panic(expected = "self referencing")]
    fn test_cycle() {
        yaml_to_dough_formula(String::from("./test_cycle.yaml"));
    }

    #[test]
    #[should_panic(expected = "mix must reference all components")]
    fn test_disconnected() {
        yaml_to_dough_formula(String::from("./test_disconnected.yaml"));
    }

}