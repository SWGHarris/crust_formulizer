extern crate yaml_rust;
use std::collections::HashMap;
use yaml_rust::{YamlLoader, YamlEmitter};

// ideas
// 1. use db to contain formulas
// 2. allow for other types of input

// third argument is true if given the percentage of ingredient

#[derive(Debug)]
struct Ingredient(String, f64, bool);
impl Ingredient {
    fn new_from_mass(name: String, mass: f64, flour_mass: f64, is_flour: bool) -> Self {
        Self(name, mass / flour_mass, is_flour)
    }

    fn is_flour(&self) -> bool {
        self.2
    }
}


#[derive(Debug)]
struct DoughSegment {
    name: String,
    order: u32,
    ingredients: Vec<Ingredient>
}

#[derive(Debug)]
struct Formula {
    name: String,
    dough_segments: HashMap<String, DoughSegment>
}

fn main() {
    let s =
        "
        name: bread title
        segments:
            -
                name: poolish
                ingredients:
                    - [all purpose flour, 100.0, true]
                    - [water,             100.0, false]
                    - [yeast,               1.0, false]
            - 
                name: mix
                ingredients:
                    - [all purpose flour, 50.0, true]
                    - [whole wheat flour, 50.0, true]
                    - [water,             80.0, false]
                    - [poolish,           10.0, false]
        ";

    let docs = YamlLoader::load_from_str(s).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];
    let formula_name = doc["name"].as_str().unwrap().to_string();
    let mut formula: Formula = Formula {name: formula_name, dough_segments: HashMap::new()};
    for (i, s) in doc["segments"].as_vec().unwrap().iter().enumerate() {
        let name = s["name"].as_str().unwrap().to_string();
        let mut segment: DoughSegment = DoughSegment {name: name, order: i as u32, ingredients: Vec::new()};
        for ing in s["ingredients"].as_vec().unwrap() {
            let name     = ing[0].as_str().unwrap().to_string();
            let mass     = ing[1].as_f64().unwrap();
            let is_flour = ing[2].as_bool().unwrap();
            segment.ingredients.push(Ingredient(name, mass, is_flour));
        }
        println!("{:?}", segment);
    }

    //let mut out_str = String::new();
    // {
    //    let mut emitter = YamlEmitter::new(&mut out_str);
    //    emitter.dump(doc).unwrap(); // dump the YAML object to a String
    // }
    // println!("{}", out_str);    
}
