mod cargo;

fn main() {
    let _ = cargo::dependencies(std::path::Path::new(
        "/Users/cluries/Workspace/iusworks/rings/Cargo.toml",
    ));
}
