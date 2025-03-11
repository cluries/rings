pub enum HttpMethod {
    GET,
    POST,
    DELETE,
    PUT,
    HEAD,
    OPTIONS,
    TRACE,
    PATCH,
}


impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => { "GET" }
            HttpMethod::POST => { "POST" }
            HttpMethod::DELETE => { "DELETE" }
            HttpMethod::PUT => { "PUT" }
            HttpMethod::HEAD => { "HEAD" }
            HttpMethod::OPTIONS => { "OPTIONS" }
            HttpMethod::TRACE => { "TRACE" }
            HttpMethod::PATCH => { "PATCH" }
        }
    }

    pub fn is(&self, method: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(method)
    }
}