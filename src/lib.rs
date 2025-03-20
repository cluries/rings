// Don't change this value, it will be replaced by the commit build time
pub static COMMIT_BUILD: &'static str = "20250308100426";

// Don't change this value, it will be replaced by the version
pub static VERSION: &'static str = "0.1.0 - Dev";

pub mod any;
pub mod balanced;
pub mod conf;
pub mod erx;
pub mod fns;
pub mod id;
pub mod log;
pub mod macros;
pub mod migrate;
pub mod model;
pub mod object;
pub mod rings;
pub mod scheduler;
pub mod service;
pub mod tools;
pub mod types;
pub mod web;


/// Re-export
pub mod rexp {
    pub use aes;
    pub use async_openai;
    pub use async_trait;
    pub use axum;
    pub use base64;
    pub use block_padding;
    pub use cbc;
    pub use cfb_mode;
    pub use chrono;
    pub use config;
    pub use ctr;
    pub use futures_util;
    pub use hex;
    pub use hmac;
    pub use lazy_static;
    pub use log;
    pub use mlua;
    pub use ofb;
    pub use pem;
    pub use percent_encoding;
    pub use rand;
    pub use redis;
    pub use regex;
    pub use reqwest;
    pub use rsa;
    pub use sea_orm;
    pub use sea_orm_migration;
    pub use serde;
    pub use serde_json;
    pub use sha1;
    pub use tokio;
    pub use tokio_cron_scheduler;
    pub use tower;
    pub use tower_http;
    pub use tracing;
    pub use tracing_appender;
    pub use tracing_serde;
    pub use tracing_subscriber;
    pub use url;
}




