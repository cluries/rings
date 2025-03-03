use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml_edit::{DocumentMut, Formatted, Item};

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


    let rdoc = doc.clone();

    let mut build_workspace = toml_edit::Table::new();

    let mut workspace_depends_keys: Vec<String> = vec![];

    if rdoc.contains_key("workspace") {
        let workspace = rdoc.get("workspace").unwrap().as_table().unwrap();

        if workspace.contains_key("members") {
            let members = workspace.get("members").unwrap().as_array().unwrap();
            let mut members: Vec<String> = members.iter().filter(
                |x| x.is_str()
            ).map(
                |x| x.as_str().unwrap().to_string()
            ).collect::<Vec<String>>();

            members.sort();

            let mut build_members = toml_edit::Array::new();
            for (i, member) in members.iter().enumerate() {
                build_members.insert(i, toml_edit::Value::String(Formatted::new(member.to_string())));
            }

            build_workspace["members"] = build_members.into();
        }

        if workspace.contains_key("dependencies") {
            let dependencies = workspace.get("dependencies").unwrap().as_table().unwrap();

            let mut depend_keys = dependencies.iter().map(
                |(name, _)| {
                    name.to_string()
                }
            ).collect::<Vec<String>>();
            depend_keys.sort();

            let mut build_depends = toml_edit::Table::new();
            for key in depend_keys.iter() {
                workspace_depends_keys.push(key.clone());
                let depend = dependencies.get(key).unwrap();
                build_depends.insert(key, single_version_tabled(depend));
            }

            build_workspace["dependencies"] = build_depends.into();
        }
    }

    doc["workspace"] = build_workspace.into();


    let mut build_depends = toml_edit::Table::new();
    if rdoc.contains_key("dependencies") {
        let dependencies = rdoc.get("dependencies").unwrap().as_table().unwrap();
        let mut depend_keys = dependencies.iter().map(|(name, _)| name.to_string()).collect::<Vec<String>>();
        depend_keys.sort();

        for key in depend_keys.iter() {
            let table = if workspace_depends_keys.contains(key) {
                let mut table = toml_edit::Table::new();
                table.insert("workspace", toml_edit::value(true));
                table
            } else {
                let depend = dependencies.get(key).unwrap();
                if depend.is_str() {
                    let mut table = toml_edit::Table::new();
                    table.insert("version", depend.clone());
                    table
                } else {
                    depend.clone().into_table().unwrap()
                }
            };


            build_depends.insert(key, table.into());
        }
    }

    doc["dependencies"] = build_depends.clone().into();
    print!("{}", doc.to_string());

    0
}


fn single_version_tabled(depend: &Item) -> Item {
    if depend.is_str() {
        let mut table = toml_edit::Table::new();
        table.insert("version", toml_edit::value(depend.as_str().unwrap().to_string()));
        table.into()
    } else {
        depend.clone()
    }
}