use std::cmp::PartialEq;
use std::fmt;
use std::str::FromStr;
use std::sync::{OnceLock, RwLock};
use config::Config;
use serde_derive::{Deserialize, Serialize};


pub fn settings() -> &'static RwLock<Config> {
    static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
    CONFIG.get_or_init(|| RwLock::new(init_config()))
}


pub fn rebit() -> &'static RwLock<Rebit> {
    static REBIT: OnceLock<RwLock<Rebit>> = OnceLock::new();
    REBIT.get_or_init(||
        RwLock::new(
            || -> Rebit {
                let r = settings().read().unwrap().clone().try_deserialize::<Rebit>();
                if cfg!(test) {
                    Rebit {
                        name: "Rebit".to_string(),
                        short: "REBT".to_string(),
                        debug: true,
                        web: Default::default(),
                        model: Model { backends: vec![] },
                        log: None,
                    }
                } else {
                    r.unwrap_or_else(|e| panic!("rebit loading error: {}", e))
                }
            }()
        )
    )
}


#[cfg(not(test))]
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

    let conf = config::File::with_name(&format!("{config_path}/config.yml"));
    let mode = config::File::with_name(&format!("{config_path}/{run_mode}.yml")).required(false);
    let local = config::File::with_name(&format!("{config_path}/local.yml")).required(false);

    let settings = Config::builder().add_source(conf).add_source(mode).add_source(local).add_source(config::Environment::with_prefix("REBT")).build().unwrap();
    settings
}

#[cfg(test)]
fn init_config() -> Config {
    Config::builder().build().unwrap()
}



macro_rules! make_setting_getter_default {
    ($name:ident, $type:ty, $getter:ident) => {
        pub fn $name(k: &str, default: $type) -> $type {
            // let settings = settings().read();
            match settings().read() {
                Ok(guard) => guard.$getter(k).unwrap_or(default),
                Err(_) => default
            }
        }
    };
}

macro_rules! make_setting_getter_option {
    ($name:ident, $type:ty, $getter:ident) => {
        pub fn $name(k: &str) -> Option<$type> {
            // let settings = settings().read();
            match settings().read() {
                Ok(guard) => guard.$getter(k).ok(),
                Err(_) => None
            }
        }
    };
}


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

impl Has {
    pub fn has<T: for<'a> serde::Deserialize<'a>>(k: &str) -> bool {
        let settings = settings().read();
        match settings {
            Ok(guard) => guard.get::<T>(k).is_ok(),
            Err(_) => false
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rebit {
    pub name: String,
    pub short: String,
    pub debug: bool,
    pub web: Web,
    pub model: Model,
    pub log: Option<Log>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Log {
    pub level: String,
    pub console: bool,
    pub dirs: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Web {
    pub port: u16,
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
        Self {
            port: 80,
        }
    }
}

impl Default for Rebit {
    fn default() -> Self {
        Self {
            name: "Rings".to_string(),
            short: "RING".to_string(),
            debug: true,
            web: Default::default(),
            model: Model {
                backends: vec![]
            },
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

