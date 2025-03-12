use mlua::{
    Error as LuaError, Function as LuaFunction, Lua as LuaLua,
};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct LuaBridge {
    code: String,
    lua: Arc<Mutex<LuaLua>>,
    rust_functions: Arc<Mutex<HashMap<String, Box<dyn Fn(&LuaLua) -> mlua::Result<LuaFunction> + Send + Sync>>>>,
}


fn re<T: ToString>(e: T) -> LuaError {
    mlua::Error::RuntimeError(e.to_string())
}

impl LuaBridge {
    pub fn new(code: String) -> Self {
        LuaBridge {
            code,
            lua: Arc::new(Mutex::new(LuaLua::new())),
            rust_functions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_function<F>(&mut self, name: &str, func: F) -> mlua::Result<()>
    where
        F: Fn(&LuaLua) -> mlua::Result<LuaFunction> + Send + Sync + 'static,
    {
        let _ = self.rust_functions
            .try_lock()
            .map_err(re)?
            .insert(name.into(), Box::new(func))
            .ok_or(re(format!("unable register rust function: {}", name)))?;
        Ok(())
    }

    pub fn execute(&self) -> mlua::Result<()> {
        let lua = self.lua.try_lock().map_err(re)?;
        let globals = lua.globals();
        for (name, func) in self.rust_functions.try_lock().map_err(re)?.iter() {
            globals.set(name.clone(), func(&lua)?)?;
        }

        lua.load(&self.code).exec().map(|_| Ok(()))?
    }


    pub fn get_global<T: mlua::FromLua>(&self, name: &str) -> mlua::Result<T> {
        let lua = self.lua.try_lock().map_err(re)?;
        let globals = lua.globals();
        globals.get(name)
    }

    pub fn set_global<T: mlua::IntoLua>(&self, name: &str, value: T) -> mlua::Result<()> {
        let lua = self.lua.try_lock().map_err(re)?;
        let globals = lua.globals();
        globals.set(name, value)
    }

    pub fn call_function<A, R>(&self, name: &str, args: A) -> mlua::Result<R>
    where
        A: mlua::IntoLuaMulti,
        R: mlua::FromLuaMulti,
    {
        let lua = self.lua.try_lock().map_err(re)?;
        let globals = lua.globals();
        let func: LuaFunction = globals.get(name)?;
        func.call(args)
    }
}

unsafe impl Send for LuaBridge {}
unsafe impl Sync for LuaBridge {}


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

        let _ = bridge.register_function("test_func", |lua| {
            lua.create_function(|_, (a, b): (i64, i64)| {
                let a = a + 200;
                let r = a * 2 + b;

                Ok(format!("{} + {} + 2", get_name(), r))
            })
        });


        // 执行Lua代码
        bridge.execute().unwrap();


        // 测试调用函数
        let sum: String = bridge.call_function("generateName", ("11", 22)).unwrap();
        println!("{}", sum);
    }
}
