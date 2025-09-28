// use std::sync::OnceLock;

pub struct Runtime {
    async_runtime: tokio::runtime::Runtime,
    application_names: (String, String), // fullname, shortname
}

// static SHARED: std::sync::OnceLock<Runtime> = OnceLock::new();

// pub fn shared() -> mut &'static Runtime {
//     SHARED.get_or_init(|| Runtime { async_runtime: ToKioRuntime::new().unwrap(), application_names: ("".to_string(), "".to_string()) })
// }

impl Runtime {
    pub fn application_name(&self) -> &str {
        &self.application_names.0
    }

    pub fn application_short(&self) -> &str {
        &self.application_names.1
    }
}
