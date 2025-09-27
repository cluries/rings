use mlua::{/*Error as LuaError,*/ Function as LuaFunction, Lua as LuaLua};

use std::collections::HashMap;

#[allow(clippy::type_complexity)]
pub struct LuaBridge {
    code: String,
    lua: LuaLua,
    rust_functions: HashMap<String, Box<dyn Fn(&LuaLua) -> mlua::Result<LuaFunction>>>,
}

// fn re<T: ToString>(e: T) -> LuaError {
//     mlua::Error::RuntimeError(e.to_string())
// }

impl LuaBridge {
    pub fn new(code: String) -> Self {
        LuaBridge { code, lua: LuaLua::new(), rust_functions: HashMap::new() }
    }

    pub fn register_function<F>(&mut self, name: &str, func: F) -> mlua::Result<()>
    where
        F: Fn(&LuaLua) -> mlua::Result<LuaFunction> + Send + Sync + 'static,
    {
        self.rust_functions.insert(name.into(), Box::new(func));
        Ok(())
    }

    pub fn execute(&self) -> mlua::Result<()> {
        let globals = self.lua.globals();
        for (name, func) in self.rust_functions.iter() {
            globals.set(name.clone(), func(&self.lua)?)?;
        }

        self.lua.load(&self.code).exec().map(|_| Ok(()))?
    }

    pub fn get_global<T: mlua::FromLua>(&self, name: &str) -> mlua::Result<T> {
        let globals = self.lua.globals();
        globals.get(name)
    }

    pub fn set_global<T: mlua::IntoLua>(&self, name: &str, value: T) -> mlua::Result<()> {
        let globals = self.lua.globals();
        globals.set(name, value)
    }

    pub fn call_function<A, R>(&self, name: &str, args: A) -> mlua::Result<R>
    where
        A: mlua::IntoLuaMulti,
        R: mlua::FromLuaMulti,
    {
        let globals = self.lua.globals();
        let func: LuaFunction = globals.get(name)?;
        func.call(args)
    }
}

// unsafe impl Send for LuaBridge {}
// unsafe impl Sync for LuaBridge {}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_name() -> String {
        "lua-bridge".to_string()
    }

    #[test]
    fn test() {
        use crate::tools::tests::tools as stools;
        let lua_code = std::fs::read_to_string(stools::src_dir().join("tools/lua.lua").to_str().unwrap()).unwrap();
        let mut bridge = LuaBridge::new(lua_code);

        let c = bridge.register_function("test_func", |lua| {
            lua.create_function(|_, (a, b): (i64, i64)| {
                let a = a + 200;
                let r = a * 2 + b;

                Ok(format!("{} + {} + 2", get_name(), r))
            })
        });

        c.unwrap();

        // bridge.set_global("counter", 100).expect("TODO: panic message");

        // 执行Lua代码
        bridge.execute().unwrap();

        // 测试调用函数
        for _ in 0..10 {
            let sum: String = bridge.call_function("generateName", ("11", 22)).unwrap();
            // bridge.set_global("counter", 100).expect("TODO: panic message");
            println!("{}", sum);
        }
    }
}
