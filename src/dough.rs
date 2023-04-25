extern crate yaml_rust;
use crate::csv_cell::{self, BinOp, CSVCell, CellArray, CellExpr, CellPosition, CellValue, csv_cells_to_grid};
use core::num;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use std::{
    boxed,
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::Component,
    result,
};
use yaml_rust::YamlLoader;

const ROW_OFFSET: usize = 2;
const COL_OFFSET: usize = 1;
const MIX: &str = "mix";

#[derive(Debug)]
enum Ingredient {
    Flour(Decimal),
    NonFlour(Decimal),
}

// Ingredient name's may reference other components.
// All components but "mix" must be referenced by another segment
// "mix" may not be referenced
#[derive(Debug)]
struct DoughComponent {
    name: String,
    ingredients: HashMap<String, Ingredient>,
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
                if !component_flour.contains_key(&name) {
                    component_flour.insert(name.clone(), dec!(0));
                }
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
        non_flour: HashSet::new(),
    };

    // convert yaml to struct DoughFormula
    for (_, s) in doc["components"].as_vec().unwrap().iter().enumerate() {
        let seg_name = s["name"].as_str().unwrap().to_string();
        let mut seg: DoughComponent = DoughComponent {
            name: seg_name.clone(),
            ingredients: HashMap::new(),
        };

        for ing in s["ingredients"].as_vec().unwrap() {
            let ing_name = ing[0].as_str().unwrap().to_string();
            let mass = ing[1].as_f64().unwrap();
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
            seg.ingredients.insert(ing_name, new_ing);
        }
        formula.components.insert(seg_name, seg);
    }

    // obtain ordering of components and ingredients
    let mut flour: Vec<String> = formula.flour.clone().into_iter().collect();
    let mut non_flour: Vec<String> = formula.non_flour.clone().into_iter().collect();
    flour.sort();
    flour.reverse();
    non_flour.sort();
    non_flour.reverse();
    flour.append(&mut non_flour);
    let ingredient_order = flour;
    let mut dup_check = ingredient_order.clone();
    dup_check.sort();
    if dup_check.len() != ingredient_order.len() {
        panic!("Duplicate component names are not allowed");
    }

    let components: Vec<String> = formula.components.keys().cloned().collect();
    if components
        .iter()
        .filter(|&c| (*c == String::from(MIX)))
        .count()
        != 1
    {
        panic!("Must be exactly one component named 'mix'");
    }

    // dfs to check for cycle in component graph rooted at 'mix'
    let mut component_order = bfs_components(&formula.components);
    component_order.reverse();
    validate_structure(
        MIX,
        &formula.components,
        &mut HashSet::new(),
        &mut HashSet::new(),
    );

    let component_totals: HashMap<String, CSVCell> = component_totals(&component_order, ingredient_order.len());
    let component_percentages: HashMap<String, HashMap<String, CSVCell>> = component_percentages(&formula.components,
                                                                                                    &component_order, 
                                                                                                    &ingredient_order);
    
    let mut component_percentages_vec: Vec<CSVCell> = component_percentages.iter()
                                                                       .flat_map(|(_,v)| v.values().cloned())
                                                                       .collect();
    let mut ingredient_labels = ingredient_label_cells(&ingredient_order, component_order.len());


    let mut comp_totals_vec: Vec<CSVCell> = component_totals.values().cloned().collect();
    
    // Testing CSVCell
    let mut test_cells = Vec::new();
    test_cells.append(&mut  comp_totals_vec);
    test_cells.append(&mut component_percentages_vec);
    test_cells.append(&mut ingredient_labels);
    let test_grid_just_totals = csv_cells_to_grid(&test_cells);
    println!("{}",  test_grid_just_totals);
    
    formula
}


// returns a Vec<CSVCell> that represents the cells for ingredient labels
fn ingredient_label_cells(
    ing_ordering: &Vec<String>,
    num_components: usize
) -> Vec<CSVCell> {
    let mut result: Vec<CSVCell> = Vec::new();
    for (index, ing_name) in ing_ordering.iter().enumerate() {
        let label_position_1 = CellPosition {
            row: (ROW_OFFSET + index) as u32,
            col: 0,
            fix_row: false,
            fix_col: false,
        };

        let label_position_2 = CellPosition {
            row: label_position_1.row,
            col: 1 + 2*num_components as u32,
            fix_row: false,
            fix_col: false,
        };

        let label_cell_1 = CSVCell {
            value: CellValue::Str(ing_name.to_string()),
            position: label_position_1.clone(),
        };

        let label_cell_2 = CSVCell {
            value: CellValue::Expr(CellExpr::CellRef(label_position_1)),
            position: label_position_2,
        };

        result.push(label_cell_1);
        result.push(label_cell_2);
    }
    result
}

// returns a HashMap<String, CSVCell> that maps component names to the
// CSVCell associated with the position and expression for the component's
// percentage total
fn component_totals(
    comp_ordering: &Vec<String>,
    num_ingredients: usize,
) -> HashMap<String, CSVCell> {
    let mut result: HashMap<String, CSVCell> = HashMap::new();
    for (index, comp_name) in comp_ordering.iter().enumerate() {
        let total_position = CellPosition {
            row: (ROW_OFFSET + num_ingredients) as u32,
            col: (2*index + COL_OFFSET) as u32,
            fix_row: false,
            fix_col: false,
        };
        let from = CellPosition {
            row: total_position.row - num_ingredients as u32,
            col: total_position.col,
            fix_row: false,
            fix_col: false,
        };
        let to = CellPosition {
            row: total_position.row - 1 as u32,
            col: total_position.col,
            fix_row: false,
            fix_col: false,
        };
        let sum_array = CellArray::new(from, to);
        let total_val = CellValue::Expr(CellExpr::Sum(sum_array));
        let total_cell = CSVCell {
            value: total_val,
            position: total_position,
        };
        result.insert(comp_name.to_string(), total_cell);
    }
    result
}

// returns a HashMap that maps component names to another HashMap
// The inner hashmap associated the component ingredients to their
// percentage amount (as provided by input)
fn component_percentages(
    components: &HashMap<String, DoughComponent>,
    component_order: &Vec<String>,
    ingredient_order: &Vec<String>,
) -> HashMap<String, HashMap<String, CSVCell>> {
    let mut result: HashMap<String, HashMap<String, CSVCell>> = HashMap::new();
    for (col, comp_name) in component_order.iter().enumerate() {
        let comp: &DoughComponent = &components[comp_name];
        let mut comp_percents: HashMap<String,CSVCell> = HashMap::new();
        for (row, ing_name) in ingredient_order.iter().enumerate() {
            if comp.ingredients.contains_key(ing_name) {
                let ing_val = match comp.ingredients[ing_name] {
                    Ingredient::Flour(x) => x,
                    Ingredient::NonFlour(x) => x,
                };
    
                let percent_position = CellPosition {
                    row: (ROW_OFFSET + row) as u32,
                    col: (2*col + COL_OFFSET) as u32,
                    fix_row: false,
                    fix_col: false,
                };
    
                let percent_cell = CSVCell {
                    value: CellValue::Expr(CellExpr::Percentage(ing_val)),
                    position: percent_position.clone(),
                };
    
                comp_percents.insert(ing_name.to_string(),percent_cell);
            }
        }
        result.insert(comp_name.to_string(), comp_percents);
    }
    result
}

fn component_mass(
    components: &HashMap<String, DoughComponent>,
    component_percentages: &HashMap<String, HashMap<String, CSVCell>>,
) -> HashMap<String, CellExpr> {
    let mut component_masses: HashMap<String, CellExpr> = HashMap::new();
    component_mass_aux(MIX,
                        components, 
                        component_percentages, 
                        &mut component_masses, 
                        &mut VecDeque::new(), 
                        &mut HashSet::new(), 
                        &mut HashSet::new());
    component_masses
}

// DFS on the component-ingredient graph
//  - use to get the cell expressions that represents the actual
//    proportion of each component.
//  - must be called on the root (MIX)
fn component_mass_aux(
    current: &str,
    components: &HashMap<String, DoughComponent>,
    component_percentages: &HashMap<String, HashMap<String, CSVCell>>,
    component_masses: &mut HashMap<String, CellExpr>,
    ref_stack: &mut VecDeque<CellExpr>, // hold CellRefs along path to current
    visited: &mut HashSet<String>,
    on_path: &mut HashSet<String>,
) -> () {

    // obtain expression for current and insert to component_masses
    if current != MIX {
        // obtain product of refs on stack
        let mut expr_iter = ref_stack.iter();
        let mut prod_expr = expr_iter.next().unwrap().clone();
        for expr in expr_iter {
            let next_expr = CellExpr::BinaryOp(BinOp::Mult, Box::new(expr.clone()), Box::new(prod_expr.clone()));
            prod_expr = next_expr;
        }

        // if current component already visited => add prod_expr to previous expression
        if component_masses.contains_key(current) {
            let prev_expr: &CellExpr = &component_masses[current];
            let next_expr: CellExpr = CellExpr::BinaryOp(BinOp::Add, Box::new(prev_expr.clone()), Box::new(prod_expr));
            component_masses.insert(current.to_string(), next_expr);
        } else {
            component_masses.insert(current.to_string(), prod_expr);
        }
    }

    visited.insert(current.to_string());
    on_path.insert(current.to_string());

    // check for cycle and iterate over recursive calls to each parent component
    let component = &components[current];
    for (ing_name, _) in &component.ingredients {
        if components.contains_key(ing_name) {
            if on_path.contains(ing_name) {
                panic!("Component may not be self referencing (directly or indirectly)");
            }

            let ing_percentage = component_percentages[current][ing_name]
                                                    .position
                                                    .to_fixed(true, true);
            let ing_ref = CellExpr::CellRef(ing_percentage);
            ref_stack.push_front(ing_ref);
            component_mass_aux(current, components, component_percentages, component_masses, ref_stack, visited, on_path);
            ref_stack.pop_front();
        }
    }
    if current == MIX && components.len() != visited.len() {
        panic!("mix must reference all components directly or indirectly");
    }
    on_path.remove(current);
}

// DFS on the component-ingredient graph
//  - will panic on finding cycle => invalid formula
//  - will panic if it does not visit all components
fn validate_structure(
    current: &str,
    components: &HashMap<String, DoughComponent>,
    visited: &mut HashSet<String>,
    on_path: &mut HashSet<String>,
) {
    visited.insert(current.to_string());
    on_path.insert(current.to_string());
    let comp = components
        .get(current)
        .expect("dfs: cannot call on non-component");
    for (ing_name, _) in &comp.ingredients {
        if components.contains_key(ing_name) {
            if on_path.contains(ing_name) {
                panic!("Component may not be self referencing (directly or indirectly)");
            }
            validate_structure(ing_name, components, visited, on_path);
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
