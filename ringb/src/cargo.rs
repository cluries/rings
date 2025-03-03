use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml_edit::{
    DocumentMut,
};

pub fn dependencies(toml_path: &Path) -> i32 {
    let content = fs::read_to_string(toml_path).unwrap();

    let _ = process_dependencies(&content);
    //
    // let c = toml::to_string(&val).unwrap();
    // println!("{}", c);
    0
}


fn process_dependencies(content: &str) -> i32 {
    let mut doc = DocumentMut::from_str(content).unwrap();
    
    let mut deps = doc.get_mut("dependencies").unwrap();
   
    if &doc.contains_key("workspace") {
        
    }
    
    

    print!("{}", deps);
    
    0   
}