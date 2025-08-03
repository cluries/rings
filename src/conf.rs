use config::{Config, Value};
use serde::{Deserialize, Serialize};
///  struct GetDefault;
///  struct GetOption;
///  struct Has;
///
///  fn settings() -> & 'static RwLock<Config>
///  fn rebit() -> &' static RwLock<Rebit>
///
///  struct Rebit
use std::cmp::PartialEq;
use std::fmt;
use std::str::FromStr;
use std::sync::{OnceLock, RwLock};

//get or default
pub struct GetDefault;
pub struct GetOption;
pub struct Has;

/// get settings
/// it's not recommand to call settings() directly
/// use rebit get Rebit instance or use GetOption::xxx | GetDefaults::xxx | Has::has
///
/// # Returns
/// * `&'static RwLock<Config>` - config instance
pub fn settings() -> &'static RwLock<Config> {
    static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
    CONFIG.get_or_init(|| RwLock::new(init_config()))
}

/// get extends config with config file name
/// # Arguments
/// * `file` - config file name (full path please)
/// # Returns
/// * `&'static Config` - config instance
/// # Panics
/// * if file not found or invalid
pub fn extends(file: &str) -> &'static Config {
    static EXTENDS: OnceLock<RwLock<std::collections::HashMap<&'static str, Config>>> = OnceLock::new();
    let extends = EXTENDS.get_or_init(|| RwLock::new(std::collections::HashMap::new()));

    match extends.read() {
        Ok(read) => {
            if read.contains_key(file) {
                let c = read.get(file).unwrap();
                return unsafe { &*(c as *const Config) };
            }
        },
        Err(er) => {
            panic!("{}", er)
        },
    }

    match extends.write() {
        Ok(mut write) => {
            let c = {
                let conf = config::File::with_name(file).required(true);
                let builder = Config::builder().add_source(conf);
                builder.build().unwrap()
            };

            let key = Box::leak(file.to_owned().into_boxed_str());
            write.insert(key, c);
        },
        Err(er) => {
            panic!("{}", er)
        },
    }

    unsafe { &*(extends.read().unwrap().get(file).unwrap() as *const Config) }
}

/// get rebit instance
/// # Returns
/// * `&'static RwLock<Rebit>` - rebit instance
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
                    web: Default::default(),
                    model: Model { backends: None },
                    log: None,
                    extends: None,
                }
            } else {
                r.unwrap_or_else(|e| panic!("rebit loading error: {}", e))
            }
        }())
    })
}

/// init config
/// # Returns
/// * `Config` - config instance
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

        let tests_load = format!("{}/tests/using-test-config.yml", project_dir().to_string_lossy());
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

make_setting_getter!(string, String, get_string);
make_setting_getter!(boolean, bool, get_bool);
make_setting_getter!(int, i64, get_int);
make_setting_getter!(float, f64, get_float);
make_setting_getter!(table, std::collections::HashMap<String, Value>, get_table);
make_setting_getter!(array, Vec<Value>, get_array);

impl GetOption {
    pub fn get<'de, T: Deserialize<'de>>(key: &str) -> Option<T> {
        match settings().read() {
            Ok(guard) => guard.get(key).ok(),
            Err(_) => None,
        }
    }
}
impl GetDefault {
    pub fn get<'de, T: Deserialize<'de>>(key: &str, default: T) -> T {
        match settings().read() {
            Ok(guard) => guard.get(key).unwrap_or(default),
            Err(_) => default,
        }
    }
}

impl Has {
    pub fn has<T: for<'a> serde::Deserialize<'a>>(k: &str) -> bool {
        let settings = settings().read();
        match settings {
            Ok(guard) => guard.get::<T>(k).is_ok(),
            Err(_) => false,
        }
    }
}

/// Rebit config
/// # Fields
/// * `name` - rebit name
/// * `short` - rebit short name
/// * `debug` - rebit debug mode
/// * `web` - rebit web config
/// * `model` - rebit model config
/// * `log` - rebit log config
/// * `extends` - rebit extends config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rebit {
    pub name: String,
    pub short: String,
    pub debug: bool,
    pub web: Dict<Web>,
    pub model: Model,
    pub log: Option<Log>,
    pub extends: Option<DictString>,
}

/// Rebit log config
/// # Fields
/// * `level` - rebit log level
/// * `console` - rebit log console
/// * `dirs` - rebit log dirs   
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Log {
    pub level: String,
    pub console: bool,
    pub dirs: String,
}

/// HashMap<String, T>
pub type Dict<T> = std::collections::HashMap<String, T>;

/// HashMap<String, HashMap<String,T>>
pub type DDict<T> = Dict<Dict<T>>;

/// HashMap<String, String>
pub type DictString = Dict<String>;

/// HashMap<String, HashMap<String, String>>
pub type DDictString = DDict<String>;

/// Rebit web config
/// # Fields
/// * `port` - rebit web port
/// * `bind` - rebit web bind
/// * `middleware` - rebit web middleware
/// * `options` - rebit web options
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Web {
    pub port: u16,
    pub bind: Option<String>,
    pub middleware: Option<DDictString>,
    pub options: Option<DictString>,
}

/// BackendKind
/// # Fields
/// * `Redis` - redis backend
/// * `Postgres` - postgres backend
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
        Self { bind: None, port: 80, middleware: None, options: None }
    }
}

impl Default for Rebit {
    fn default() -> Self {
        Self {
            name: "Rings".to_string(),
            short: "RING".to_string(),
            debug: false,
            web: Default::default(),
            model: Model { backends: None },
            log: Default::default(),
            extends: Default::default(),
        }
    }
}

impl FromStr for BackendKind {
    type Err = String;

    /// parse string to BackendKind
    /// # Arguments
    /// * `s` - string
    /// # Returns
    /// * `Result<Self, Self::Err>` - BackendKind       
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
    pub kind: BackendKind,
    pub readonly: bool,
    pub connect: String,
    pub options: Option<DictString>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Model {
    pub backends: Option<Dict<Backend>>,
}

impl Model {
    pub fn backend(&self, name: &str) -> Option<Backend> {
        match &self.backends {
            None => None,
            Some(backends) => backends.get(name).cloned(),
        }
    }
}

impl Rebit {
    pub fn has_backend(&self) -> bool {
        match &self.model.backends {
            None => false,
            Some(bs) => bs.len() > 0,
        }
    }

    pub fn get_backend(&self, name: &str) -> Option<Backend> {
        self.model.backend(name)
    }

    /// has web config
    /// # Returns
    /// * `bool` - true if has web config
    pub fn has_web(&self) -> bool {
        self.web.len() > 0
    }

    /// get web config
    /// # Arguments
    /// * `name` - web name
    /// # Returns
    /// * `Option<Web>` - web config
    pub fn get_web(&self, name: &str) -> Option<Web> {
        self.web.get(name).cloned()
    }

    /// get web middleware
    /// # Arguments
    /// * `name` - web name
    /// * `middleware_name` - middleware name
    /// # Returns
    /// * `Option<DictString>` - middleware options
    pub fn web_middleware(&self, name: &str, middleware_name: &str) -> Option<DictString> {
        self.get_web(name).and_then(|web| web.middleware).and_then(|mw| mw.get(middleware_name).cloned())
    }

    /// get extend value
    /// # Arguments
    /// * `name` - extend name
    /// # Returns
    /// * `Option<String>` - extend value
    pub fn get_extend(&self, name: &str) -> Option<String> {
        self.extends.as_ref()?.get(name).cloned()
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::tests::tools::project_dir;

    #[test]
    fn tests() {
        // print!("{:?}", settings().read().unwrap().clone().try_deserialize::<std::collections::HashMap<String, Value>>().unwrap());
        println!("{:?}", GetOption::string("name"));
        println!("{:?}", GetOption::table("model.backends.postgre"));
    }

    #[test]
    fn test_path_value() {
        #[derive(Debug, Deserialize, Serialize, Clone)]
        struct Port {
            pub tcp: i32,
            pub udp: i32,
        }

        println!("{:?}", GetOption::get::<Vec<Port>>("providers.cnpc.management.ports"));
    }

    /// test extends
    #[test]
    fn test_extends() {
        let tests_load = format!("{}/tests/using-test-config.yml", project_dir().to_string_lossy());
        let c = extends(&tests_load);
        println!("{:?}", c);
    }
}
