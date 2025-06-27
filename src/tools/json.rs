use crate::erx::{smp, ResultE};
use serde::{de::DeserializeOwned, Serialize};

pub struct Enc;
pub struct Dec;

impl Enc {
    pub fn en<T: Serialize>(obj: &T) -> ResultE<String> {
        serde_json::to_string(obj).map_err(smp)
    }

    pub fn ens<T: Serialize>(obj: &T) -> String {
        serde_json::to_string(obj).unwrap_or(Default::default())
    }

    pub fn pretty<T: Serialize>(obj: &T) -> ResultE<String> {
        serde_json::to_string_pretty(obj).map_err(smp)
    }
}

impl Dec {
    pub fn de<T: DeserializeOwned>(json: &str) -> ResultE<T> {
        serde_json::from_str(json).map_err(smp)
    }

    pub async fn file<T: DeserializeOwned>(filename: &str) -> ResultE<T> {
        let fc = crate::tools::fs::Content(filename.to_string());
        fc.json().await
    }
}
