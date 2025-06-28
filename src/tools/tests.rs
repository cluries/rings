pub mod tools {
    use std::path::PathBuf;

    pub fn project_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    pub fn src_dir() -> PathBuf {
        project_dir().join("src")
    }

    pub fn set_config_env() -> String {
        let binding = project_dir().join("config");
        let dir = binding.as_os_str().to_str().unwrap();
        std::env::set_var("REBT_CONFIG_PATH", dir);
        dir.to_string()
    }

    pub async fn init_logger() {
        set_config_env();
        crate::log::logging_initialize().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::tests::tools::{init_logger, set_config_env};
    use tracing::info;

    #[tokio::test]
    async fn test_set_config_env() {
        init_logger().await;
        info!("{}", set_config_env());
    }

    #[test]
    pub fn test_current_working_path() {
        println!("{}", tools::src_dir().to_string_lossy().to_string());
    }
}
