use axum::http::HeaderMap;

/// 简单的 Cookie 解析器
pub struct CookieJar {
    cookies: std::collections::HashMap<String, String>,
}

impl CookieJar {
    /// 从请求头创建 CookieJar
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let mut cookies = std::collections::HashMap::new();
        
        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some((key, value)) = cookie.split_once('=') {
                        cookies.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }
        
        Self { cookies }
    }
    
    /// 获取指定名称的 cookie
    pub fn get(&self, name: &str) -> Option<Cookie> {
        self.cookies.get(name).map(|value| Cookie {
            name: name.to_string(),
            value: value.clone(),
        })
    }
}

/// Cookie 结构
pub struct Cookie {
    name: String,
    value: String,
}

impl Cookie {
    /// 获取 cookie 值
    pub fn value(&self) -> &str {
        &self.value
    }
    
    /// 获取 cookie 名称
    pub fn name(&self) -> &str {
        &self.name
    }
}