use reqwest;
use reqwest::Response;
use std::{str::FromStr, time::Duration};

use crate::web::url::join as url_join;

pub struct ClientBuilder {
    base: String,
    headers: reqwest::header::HeaderMap,
    user_agent: Option<String>,
    proxy: Option<String>,
    no_tls_verify: bool,
}

pub struct Client {
    base: String,
    cli: reqwest::Client,
}


static DEFAULT_USER_AGENT: &str = "Rings/1.0.0 (V1; Linux ; en-US; Iusworks.inc;)";

impl ClientBuilder {
    pub fn new(base: String) -> ClientBuilder {
        let headers = reqwest::header::HeaderMap::new();
        let proxy = None;
        let user_agent = None;
        let no_tls_verify = false;

        ClientBuilder { base, headers, user_agent, proxy, no_tls_verify }
    }

    pub fn set_user_agent(&mut self, agent: String) -> &mut Self {
        self.user_agent = Some(agent);
        self
    }

    pub fn add_header(&mut self, key: &str, value: &str) -> &mut Self {
        let key = reqwest::header::HeaderName::from_str(key).unwrap();
        let value = reqwest::header::HeaderValue::from_str(value).unwrap();
        self.headers.insert(key, value);
        self
    }

    pub fn use_json(&mut self) -> &mut Self {
        let val = reqwest::header::HeaderValue::from_static("application/json");
        self.headers.insert(reqwest::header::ACCEPT, val.clone());
        self.headers.insert(reqwest::header::CONTENT_TYPE, val.clone());

        self
    }

    pub fn no_tls_verify(&mut self) -> &mut Self {
        self.no_tls_verify = true;
        self
    }

    pub fn enable_tls_verify(&mut self) -> &mut Self {
        self.no_tls_verify = false;
        self
    }

    pub fn set_proxy(&mut self, proxy: String) -> &mut Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn build(self) -> Client {
        let mut builder = reqwest::Client::builder();
        if let Some(proxy) = &self.proxy {
            let proxy = reqwest::Proxy::all(proxy).expect("proxy set failed");
            builder = builder.proxy(proxy);
        }


        if let Some(user_agent) = &self.user_agent {
            builder = builder.user_agent(user_agent);
        } else {
            builder = builder.user_agent(DEFAULT_USER_AGENT);
        }

        builder = builder.default_headers(self.headers.clone());
        builder = builder.timeout(Duration::from_secs(10));
        builder = builder.redirect(reqwest::redirect::Policy::none());

        if self.no_tls_verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let cli = builder.build().expect("http client build failed");
        let base = self.base.clone();

        Client { base, cli }
    }
}


impl Client {
    fn human_error<T: ToString>(error: T) -> String {
        error.to_string()
    }

    // get
    pub async fn get(&self, path: &str) -> Result<String, String> {
        // let url = url_join(&self.base, path);
        // let response = self.cli.get(url).send().await.map_err(|e| e.to_string())?;

        Self::_response_untyped(
            self.cli.get(
                url_join(&self.base, path)
            ).send().await.map_err(Self::human_error)?
        ).await
    }

    pub async fn post(&self, path: &str, body: String) -> Result<String, String> {
        let url = url_join(&self.base, path);
        let response = self.cli.post(url).body(body).send().await.map_err(Self::human_error)?;
        Self::_response_untyped(response).await
    }

    pub async fn put(&self, path: &str, body: String) -> Result<String, String> {
        let url = url_join(&self.base, path);
        let response = self.cli.put(url).body(body).send().await.map_err(Self::human_error)?;
        Self::_response_untyped(response).await
    }

    pub async fn delete(&self, path: &str) -> Result<String, String> {
        let url = url_join(&self.base, path);
        let response = self.cli.delete(url).send().await.map_err(Self::human_error)?;
        Self::_response_untyped(response).await
    }

    pub async fn head(&self, path: &str) -> Result<String, String> {
        let url = url_join(&self.base, path);
        let response = self.cli.head(url).send().await.map_err(Self::human_error)?;
        Self::_response_untyped(response).await
    }

    pub async fn get_typed<T>(&self, path: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = url_join(&self.base, path);
        let response = self.cli.get(url).send().await.map_err(Self::human_error)?;
        Self::_response_typed(response).await
    }

    pub async fn post_typed<ResponseT, RequestT>(&self, path: &str, params: &RequestT) -> Result<ResponseT, String>
    where
        ResponseT: serde::de::DeserializeOwned,
        RequestT: serde::Serialize + ?Sized,
    {
        let url = url_join(&self.base, path);
        let response = self.cli.post(url).json(params).send().await.map_err(Self::human_error)?;
        Self::_response_typed(response).await
    }


    pub async fn put_typed<ResponseT, RequestT>(&self, path: &str, params: &RequestT) -> Result<ResponseT, String>
    where
        ResponseT: serde::de::DeserializeOwned,
        RequestT: serde::Serialize + ?Sized,
    {
        let url = url_join(&self.base, path);
        let response = self.cli.put(url).json(params).send().await.map_err(Self::human_error)?;
        Self::_response_typed(response).await
    }

    pub async fn delete_typed<T>(&self, path: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = url_join(&self.base, path);
        let response = self.cli.delete(url).send().await.map_err(Self::human_error)?;
        Self::_response_typed(response).await
    }

    pub async fn head_typed<T>(&self, path: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = url_join(&self.base, path);
        let response = self.cli.head(url.as_str()).send().await.map_err(Self::human_error)?;
        Self::_response_typed(response).await
    }


    pub async fn get_valued(&self, path: &str) -> Result<serde_json::Value, String> {
        self.get_typed(path).await
    }

    pub async fn post_valued<T>(&self, path: &str, params: &T) -> Result<serde_json::Value, String>
    where
        T: serde::Serialize + ?Sized,
    {
        self.post_typed(path, params).await
    }

    pub async fn put_valued<T>(&self, path: &str, params: &T) -> Result<serde_json::Value, String>
    where
        T: serde::Serialize + ?Sized,
    {
        self.put_typed(path, params).await
    }

    pub async fn delete_valued(&self, path: &str) -> Result<serde_json::Value, String> {
        self.delete_typed(
            path
        ).await
    }
    pub async fn head_valued(&self, path: &str) -> Result<serde_json::Value, String> {
        self.head_typed(path).await
    }

    async fn _response_untyped(response: Response) -> Result<String, String> {
        let status = response.status();
        let body = response.text().await.map_err(Self::human_error)?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(body)
        }
    }

    async fn _response_typed<T>(response: Response) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        if response.status().is_success() {
            Ok(
                response.json::<T>().await.map_err(Self::human_error)?
            )
        } else {
            Err(
                response.text().await.map_err(Self::human_error)?
            )
        }
    }
}

