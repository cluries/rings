use crate::tools::fs;
// lua use  action_get_home, action_post_auth, action_put_update
pub struct LuaAction {
    prefix: String,  // URL Prefix
    scripts_location: String, // Lua scripts location
}


async fn a() {}

pub struct LuaActionContext {}


impl LuaAction {
    pub async fn new(prefix: String, scripts_location: String) -> LuaAction {
        if prefix.ends_with("*") {
            panic!("prefix must not end with '*'");
        }


        let path = fs::join_path(vec![fs::working_dir().unwrap().to_str().unwrap(), scripts_location.as_str()]);
        if !fs::Is(path.clone()).dir().await {
            panic!("cannot read scripts dir: {}", path);
        }


        Self { prefix, scripts_location }
    }

    pub fn route(&self) -> axum::Router {
        let pattern = &format!("{}*", self.prefix);
        axum::Router::new().route(pattern, axum::routing::any(a))
    }

    pub fn load(&mut self) {}
}

impl LuaActionContext {
    pub fn new() -> Self {
        Self {}
    }
}

unsafe impl Sync for LuaAction {}
unsafe impl Send for LuaAction {}

