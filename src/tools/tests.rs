pub mod tools {
    use std::path::PathBuf;

    pub fn project_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
    
    pub fn src_dir() -> PathBuf {
        project_dir().join("src")
    }
    
   
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_current_working_path() {
        println!("{}", tools::src_dir().to_string_lossy().to_string());
    }
}
