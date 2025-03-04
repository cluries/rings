use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml_edit::{DocumentMut, Formatted, Item};

const STR_DEPENDENCIES: &str = "dependencies";
const STR_WORKSPACE: &str = "workspace";
const STR_MEMBERS: &str = "members";
const STR_VERSION: &str = "version";


#[derive(Debug)]
pub struct Flags {
    pub toml: String,
    pub write: bool,
    pub dependencies_into_workspace: bool,
}

impl Flags {
    pub fn toml_path(&self) -> &Path {
        Path::new(&self.toml)
    }
}

pub fn cargo(flags: Flags) {
    let content = fs::read_to_string(flags.toml_path()).expect(
        &format!("failed to read cargo toml from {:?}", flags.toml)
    );
    let doc = dependencies(&flags, &content);
    let content = doc.to_string();

    if flags.write {
        fs::write(flags.toml_path(), content).expect(
            &format!("failed write cargo toml to: {:?}", flags.toml)
        );
    } else {
        println!("{}", doc.to_string());
    }
}

fn dependencies(flags: &Flags, content: &str) -> DocumentMut {
    let mut document = DocumentMut::from_str(content).expect("failed to parse content as toml document");

    let senseless = |document: &mut DocumentMut| {
        let mut workspace_existing_dependencies: Vec<String> = Vec::new();

        let workspace = refactor_workspace(document.clone(), &mut workspace_existing_dependencies);
        let dependencies = refactor_dependencies(document.clone(), &workspace_existing_dependencies);

        document[STR_WORKSPACE] = workspace.into();
        document[STR_DEPENDENCIES] = dependencies.into();

        workspace_existing_dependencies
    };

    let workspace_existing_dependencies = senseless(&mut document);

    if flags.dependencies_into_workspace {
        let rdoc = document.clone();
        let workspace_dependencies = document
            .get_mut(STR_WORKSPACE).unwrap().as_table_mut().unwrap()
            .get_mut(STR_DEPENDENCIES).unwrap().as_table_mut().unwrap();
        let dependencies = rdoc.get(STR_DEPENDENCIES).unwrap().as_table().unwrap();

        if dependencies.iter().filter(
            |(key, _)| !workspace_existing_dependencies.contains(&key.to_string())
        ).map(
            |(key, val)| workspace_dependencies[key] = val.clone()
        ).count() > 0 {
            senseless(&mut document);
        }
    }

    document
}


fn refactor_dependencies(doc: DocumentMut, workspace_existing_dependencies: &Vec<String>) -> toml_edit::Table {
    let mut build_dependencies = toml_edit::Table::new();
    if doc.contains_key(STR_DEPENDENCIES) {
        let dependencies = doc.get(STR_DEPENDENCIES).unwrap().as_table().unwrap();
        let mut depend_keys = dependencies.iter().map(|(name, _)| name.to_string()).collect::<Vec<String>>();

        depend_keys.sort();

        for key in depend_keys.iter() {
            let table = if workspace_existing_dependencies.contains(key) {
                let mut table = toml_edit::Table::new();
                table.insert(STR_WORKSPACE, toml_edit::value(true));
                table
            } else {
                let depend = dependencies.get(key).unwrap();
                if depend.is_str() {
                    let mut table = toml_edit::Table::new();
                    table.insert(STR_VERSION, depend.clone());
                    table
                } else {
                    depend.clone().into_table().expect("dependencies are not a table")
                }
            };

            build_dependencies.insert(key, table.into_inline_table().into());
        }
    }

    build_dependencies
}

fn refactor_workspace(doc: DocumentMut, workspace_existing_dependencies: &mut Vec<String>) -> toml_edit::Table {
    let mut build_workspace = toml_edit::Table::new();

    if doc.contains_key(STR_WORKSPACE) {
        let origin_workspace = doc.get(STR_WORKSPACE).unwrap().as_table().unwrap();

        if origin_workspace.contains_key(STR_MEMBERS) {
            let members = origin_workspace.get(STR_MEMBERS).unwrap().as_array()
                .expect("workspace.members must be array");

            let mut members = members.iter()
                .filter(|x| x.is_str())
                .map(|x| x.as_str().expect("members item must be string").to_string())
                .collect::<Vec<String>>();

            members.sort();

            let mut build_members = toml_edit::Array::new();
            for (i, member) in members.iter().enumerate() {
                build_members.insert(
                    i,
                    toml_edit::Value::String(Formatted::new(member.to_string())),
                );
            }

            build_workspace[STR_MEMBERS] = build_members.into();
        }

        if origin_workspace.contains_key(STR_DEPENDENCIES) {
            let origin_workspace_dependencies = origin_workspace.get(STR_DEPENDENCIES).unwrap().as_table().expect("workspace.dependencies must be table");

            let mut workspace_depend_keys = origin_workspace_dependencies.iter()
                .map(|(name, _)| name.to_string())
                .collect::<Vec<String>>();

            workspace_depend_keys.sort();

            let mut build_dependencies = toml_edit::Table::new();
            for key in workspace_depend_keys.iter() {
                workspace_existing_dependencies.push(key.clone());
                let depend = origin_workspace_dependencies.get(key).unwrap();
                build_dependencies.insert(key, depend_single_version_tabled(depend));
            }

            build_workspace[STR_DEPENDENCIES] = build_dependencies.into();
        }
    }

    build_workspace
}


fn depend_single_version_tabled(depend: &Item) -> Item {
    if !depend.is_str() {
        return depend.clone();
    }

    let mut table = toml_edit::Table::new();
    table.insert(
        STR_VERSION,
        toml_edit::value(depend.as_str().unwrap().to_string()),
    );
    table.into()
}
