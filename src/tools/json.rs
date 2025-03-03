use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Enc;
pub struct Dec;

impl Enc {
    pub fn en<T: Serialize>(obj: &T) -> Result<String, crate::erx::Erx> {
        serde_json::to_string(obj).map_err(crate::erx::smp)
    }

    pub fn ens<T: Serialize>(obj: &T) -> String {
        serde_json::to_string(obj).unwrap_or(Default::default())
    }

    pub fn pretty<T: Serialize>(obj: &T) -> Result<String, crate::erx::Erx> {
        serde_json::to_string_pretty(obj).map_err(crate::erx::smp)
    }
}

impl Dec {
    pub fn de<T: DeserializeOwned>(json: &str) -> Result<T, crate::erx::Erx> {
        serde_json::from_str(json).map_err(crate::erx::smp)
    }

    pub async fn file<T: DeserializeOwned>(filename: &str) -> Result<T, crate::erx::Erx> {
        let fc = crate::tools::file::FileContent(filename.to_string());
        fc.json().await
    }
}
