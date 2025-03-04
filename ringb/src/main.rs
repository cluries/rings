mod cargo;

fn main() {
    let _ = cargo::cargo(cargo::Flags {
        toml: "/Users/cluries/Workspace/iusworks/rings/Cargo.toml".to_string(),
        write: true,
        dependencies_into_workspace: true,
    });
}
