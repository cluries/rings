use config::{Config, Value};
use serde::{Deserialize, Serialize};
///  struct GetDefault;
///  struct GetOption;
///  struct Has;
///
///  fn settings() -> &'static RwLock<Config>
///  fn rebit() -> &'static RwLock<Rebit>
///
///  struct Rebit
use std::cmp::PartialEq;
use std::fmt;
use std::str::FromStr;
use std::sync::{OnceLock, RwLock};

/// get settings
pub fn settings() -> &'static RwLock<Config> {
    static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
    CONFIG.get_or_init(|| RwLock::new(init_config()))
}

/// get rebit instance
pub fn rebit() -> &'static RwLock<Rebit> {
    static REBIT: OnceLock<RwLock<Rebit>> = OnceLock::new();
    REBIT.get_or_init(|| {
        RwLock::new(|| -> Rebit {
            let r = settings().read().unwrap().clone().try_deserialize::<Rebit>();
            if cfg!(test) {
                Rebit {
                    name: "Rebit".to_string(),
                    short: "REBT".to_string(),
                    debug: true,
                    webs: Default::default(),
                    model: Model { backends: vec![] },
                    log: None,
                }
            } else {
                r.unwrap_or_else(|e| panic!("rebit loading error: {}", e))
            }
        }())
    })
}

// #[cfg(not(test))]
fn init_config() -> Config {
    //development production testing
    let run_mode = std::env::var("REBT_RUN_MODE").unwrap_or("development".to_string());

    tracing::info!("REBT_RUN_MODE={}", run_mode);

    let config_path = std::env::var("REBT_CONFIG_PATH").unwrap_or("config".to_string());
    // while !crate::tools::file::File(config_path.clone()).is_directory() {
    //     //TODO
    //     break;
    //     // let workdir = env::current_dir().unwrap().to_str().unwrap().to_string();
    // }

    tracing::info!("Config file path: {}", config_path);

    let conf = config::File::with_name(&format!("{config_path}/config.yml")).required(false);
    let mode = config::File::with_name(&format!("{config_path}/{run_mode}.yml")).required(false);
    let local = config::File::with_name(&format!("{config_path}/local.yml")).required(false);

    let mut builder = Config::builder().add_source(conf).add_source(mode).add_source(local);
    #[cfg(test)]
    {
        use crate::tools::tests::tools::project_dir;

        let tests_load = format!("{}/tests_load.yml", project_dir().to_string_lossy());
        tracing::info!("test mode, loading: {}", tests_load);
        println!("test mode, loading: {}", tests_load);

        builder = builder.add_source(config::File::with_name(tests_load.as_str()).required(false));
    }

    builder = builder.add_source(config::Environment::with_prefix("REBT"));

    builder.build().unwrap()
}

// #[cfg(test)]
// fn init_config() -> Config {
//     Config::builder().build().unwrap()
// }

/// make getter for settings, if not found, return default value
macro_rules! make_setting_getter_default {
    ($name:ident, $type:ty, $getter:ident) => {
        pub fn $name(k: &str, default: $type) -> $type {
            // let settings = settings().read();
            match settings().read() {
                Ok(guard) => guard.$getter(k).unwrap_or(default),
                Err(_) => default,
            }
        }
    };
}

/// make getter for settings, return Option value
macro_rules! make_setting_getter_option {
    ($name:ident, $type:ty, $getter:ident) => {
        pub fn $name(k: &str) -> Option<$type> {
            // let settings = settings().read();
            match settings().read() {
                Ok(guard) => guard.$getter(k).ok(),
                Err(_) => None,
            }
        }
    };
}

/// make getter for settings
macro_rules! make_setting_getter {
    ($name:ident, $type:ty, $getter:ident) => {
        impl GetDefault {
            make_setting_getter_default!($name, $type, $getter);
        }

        impl GetOption {
            make_setting_getter_option!($name, $type, $getter);
        }
    };
}

//get or default
pub struct GetDefault;
pub struct GetOption;
pub struct Has;

make_setting_getter!(string, String, get_string);
make_setting_getter!(boolean, bool, get_bool);
make_setting_getter!(int, i64, get_int);
make_setting_getter!(float, f64, get_float);
make_setting_getter!(table, std::collections::HashMap<String, Value>, get_table);
make_setting_getter!(array, Vec<Value>, get_array);

impl Has {
    pub fn has<T: for<'a> serde::Deserialize<'a>>(k: &str) -> bool {
        let settings = settings().read();
        match settings {
            Ok(guard) => guard.get::<T>(k).is_ok(),
            Err(_) => false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rebit {
    pub name: String,
    pub short: String,
    pub debug: bool,
    pub webs: Vec<Web>,
    pub model: Model,
    pub log: Option<Log>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Log {
    pub level: String,
    pub console: bool,
    pub dirs: String,
}

pub type NestedMap = std::collections::HashMap<String, std::collections::HashMap<String, String>>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Web {
    pub name: String,
    pub bind: Option<String>,
    pub port: u16,
    pub middleware: Option<NestedMap>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Eq)]
pub enum BackendKind {
    Redis,
    Postgres,
}

impl Default for Log {
    fn default() -> Self {
        Log {
            level: "trace".to_string(),
            console: true,
            dirs: "./logs".to_string(), //./logs
        }
    }
}

impl Default for Web {
    fn default() -> Self {
        Self { name: format!("Web-{}", crate::tools::rand::rand_str(8)), bind: None, port: 80, middleware: None }
    }
}

impl Default for Rebit {
    fn default() -> Self {
        Self {
            name: "Rings".to_string(),
            short: "RING".to_string(),
            debug: true,
            webs: Default::default(),
            model: Model { backends: vec![] },
            log: Default::default(),
        }
    }
}

impl FromStr for BackendKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "redis" => Ok(BackendKind::Redis),
            "postgres" => Ok(BackendKind::Postgres),
            _ => Err(format!("unknown backend kind: {}", s)),
        }
    }
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BackendKind::Redis => write!(f, "Redis"),
            BackendKind::Postgres => write!(f, "Postgre"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Backend {
    pub name: String,
    pub kind: BackendKind,
    pub readonly: bool,
    pub connect: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Model {
    pub backends: Vec<Backend>,
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tests() {
        print!("{:?}", settings().read().unwrap().clone().try_deserialize::<std::collections::HashMap<String, Value>>().unwrap());
        print!("{:?}", GetOption::array("webs"));
    }
}
