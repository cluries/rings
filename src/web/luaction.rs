pub struct LuaAction {
    methods: Vec<crate::web::define::HttpMethod>,
    code: String,

    init: Option<Box<dyn Fn()>>,
}

pub struct LuaActionContext {
    
}


impl LuaAction {
    pub fn new(methods: Vec<crate::web::define::HttpMethod>, code: String) -> LuaAction {
        Self { methods, code, init: None }
    }

    pub fn run(&mut self) {}
}

impl LuaActionContext {
    pub fn new() -> Self {
        Self {}
    }
}


