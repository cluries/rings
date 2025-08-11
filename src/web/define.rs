#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum HttpCode {
    // 1xx: 信息响应
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,

    // 2xx: 成功响应
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    IMUsed = 226,

    // 3xx: 重定向
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx: 客户端错误
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    // 5xx: 服务器错误
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,

    UnDefined = 0,
}

impl Into<&'static str> for HttpMethod {
    fn into(self) -> &'static str {
        self.as_str()
    }
}

impl Into<String> for HttpMethod {
    fn into(self) -> String {
        self.as_str().to_string()
    }
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PUT => "PUT",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::PATCH => "PATCH",
        }
    }

    pub fn is(&self, method: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(method)
    }

    pub fn from_str(method: &str) -> Option<HttpMethod> {
        match method.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "DELETE" => Some(HttpMethod::DELETE),
            "PUT" => Some(HttpMethod::PUT),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "TRACE" => Some(HttpMethod::TRACE),
            "PATCH" => Some(HttpMethod::PATCH),
            _ => None,
        }
    }
}
impl Into<crate::erx::Erx> for HttpCode {
    fn into(self) -> crate::erx::Erx {
        crate::erx::Erx::new(self.message())
    }
}

impl HttpCode {
    pub fn okay(code: i32) -> bool {
        code >= 200 && code < 300
    }

    pub fn ok(&self) -> bool {
        HttpCode::okay(self.code())
    }

    pub fn from_code(code: i32) -> Self {
        match code {
            100 => HttpCode::Continue,
            101 => HttpCode::SwitchingProtocols,
            102 => HttpCode::Processing,
            103 => HttpCode::EarlyHints,
            200 => HttpCode::OK,
            201 => HttpCode::Created,
            202 => HttpCode::Accepted,
            203 => HttpCode::NonAuthoritativeInformation,
            204 => HttpCode::NoContent,
            205 => HttpCode::ResetContent,
            206 => HttpCode::PartialContent,
            207 => HttpCode::MultiStatus,
            208 => HttpCode::AlreadyReported,
            226 => HttpCode::IMUsed,
            300 => HttpCode::MultipleChoices,
            301 => HttpCode::MovedPermanently,
            302 => HttpCode::Found,
            303 => HttpCode::SeeOther,
            304 => HttpCode::NotModified,
            305 => HttpCode::UseProxy,
            307 => HttpCode::TemporaryRedirect,
            308 => HttpCode::PermanentRedirect,
            400 => HttpCode::BadRequest,
            401 => HttpCode::Unauthorized,
            402 => HttpCode::PaymentRequired,
            403 => HttpCode::Forbidden,
            404 => HttpCode::NotFound,
            405 => HttpCode::MethodNotAllowed,
            406 => HttpCode::NotAcceptable,
            407 => HttpCode::ProxyAuthenticationRequired,
            408 => HttpCode::RequestTimeout,
            409 => HttpCode::Conflict,
            410 => HttpCode::Gone,
            411 => HttpCode::LengthRequired,
            412 => HttpCode::PreconditionFailed,
            413 => HttpCode::PayloadTooLarge,
            414 => HttpCode::URITooLong,
            415 => HttpCode::UnsupportedMediaType,
            416 => HttpCode::RangeNotSatisfiable,
            417 => HttpCode::ExpectationFailed,
            418 => HttpCode::ImATeapot,
            421 => HttpCode::MisdirectedRequest,
            422 => HttpCode::UnprocessableEntity,
            423 => HttpCode::Locked,
            424 => HttpCode::FailedDependency,
            425 => HttpCode::TooEarly,
            426 => HttpCode::UpgradeRequired,
            428 => HttpCode::PreconditionRequired,
            429 => HttpCode::TooManyRequests,
            431 => HttpCode::RequestHeaderFieldsTooLarge,
            451 => HttpCode::UnavailableForLegalReasons,
            500 => HttpCode::InternalServerError,
            501 => HttpCode::NotImplemented,
            502 => HttpCode::BadGateway,
            503 => HttpCode::ServiceUnavailable,
            504 => HttpCode::GatewayTimeout,
            505 => HttpCode::HTTPVersionNotSupported,
            506 => HttpCode::VariantAlsoNegotiates,
            507 => HttpCode::InsufficientStorage,
            508 => HttpCode::LoopDetected,
            510 => HttpCode::NotExtended,
            511 => HttpCode::NetworkAuthenticationRequired,
            _ => HttpCode::UnDefined,
        }
    }

    pub fn code(&self) -> i32 {
        match self {
            // 1xx: 信息响应
            HttpCode::Continue => 100,
            HttpCode::SwitchingProtocols => 101,
            HttpCode::Processing => 102,
            HttpCode::EarlyHints => 103,

            // 2xx: 成功响应
            HttpCode::OK => 200,
            HttpCode::Created => 201,
            HttpCode::Accepted => 202,
            HttpCode::NonAuthoritativeInformation => 203,
            HttpCode::NoContent => 204,
            HttpCode::ResetContent => 205,
            HttpCode::PartialContent => 206,
            HttpCode::MultiStatus => 207,
            HttpCode::AlreadyReported => 208,
            HttpCode::IMUsed => 226,

            // 3xx: 重定向
            HttpCode::MultipleChoices => 300,
            HttpCode::MovedPermanently => 301,
            HttpCode::Found => 302,
            HttpCode::SeeOther => 303,
            HttpCode::NotModified => 304,
            HttpCode::UseProxy => 305,
            HttpCode::TemporaryRedirect => 307,
            HttpCode::PermanentRedirect => 308,

            // 4xx: 客户端错误
            HttpCode::BadRequest => 400,
            HttpCode::Unauthorized => 401,
            HttpCode::PaymentRequired => 402,
            HttpCode::Forbidden => 403,
            HttpCode::NotFound => 404,
            HttpCode::MethodNotAllowed => 405,
            HttpCode::NotAcceptable => 406,
            HttpCode::ProxyAuthenticationRequired => 407,
            HttpCode::RequestTimeout => 408,
            HttpCode::Conflict => 409,
            HttpCode::Gone => 410,
            HttpCode::LengthRequired => 411,
            HttpCode::PreconditionFailed => 412,
            HttpCode::PayloadTooLarge => 413,
            HttpCode::URITooLong => 414,
            HttpCode::UnsupportedMediaType => 415,
            HttpCode::RangeNotSatisfiable => 416,
            HttpCode::ExpectationFailed => 417,
            HttpCode::ImATeapot => 418,
            HttpCode::MisdirectedRequest => 421,
            HttpCode::UnprocessableEntity => 422,
            HttpCode::Locked => 423,
            HttpCode::FailedDependency => 424,
            HttpCode::TooEarly => 425,
            HttpCode::UpgradeRequired => 426,
            HttpCode::PreconditionRequired => 428,
            HttpCode::TooManyRequests => 429,
            HttpCode::RequestHeaderFieldsTooLarge => 431,
            HttpCode::UnavailableForLegalReasons => 451,

            // 5xx: 服务器错误
            HttpCode::InternalServerError => 500,
            HttpCode::NotImplemented => 501,
            HttpCode::BadGateway => 502,
            HttpCode::ServiceUnavailable => 503,
            HttpCode::GatewayTimeout => 504,
            HttpCode::HTTPVersionNotSupported => 505,
            HttpCode::VariantAlsoNegotiates => 506,
            HttpCode::InsufficientStorage => 507,
            HttpCode::LoopDetected => 508,
            HttpCode::NotExtended => 510,
            HttpCode::NetworkAuthenticationRequired => 511,

            HttpCode::UnDefined => 0,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            // 1xx: 信息响应
            HttpCode::Continue => "Continue",
            HttpCode::SwitchingProtocols => "Switching Protocols",
            HttpCode::Processing => "Processing",
            HttpCode::EarlyHints => "Early Hints",

            // 2xx: 成功响应
            HttpCode::OK => "OK",
            HttpCode::Created => "Created",
            HttpCode::Accepted => "Accepted",
            HttpCode::NonAuthoritativeInformation => "Non-Authoritative Information",
            HttpCode::NoContent => "No Content",
            HttpCode::ResetContent => "Reset Content",
            HttpCode::PartialContent => "Partial Content",
            HttpCode::MultiStatus => "Multi-Status",
            HttpCode::AlreadyReported => "Already Reported",
            HttpCode::IMUsed => "IM Used",

            // 3xx: 重定向
            HttpCode::MultipleChoices => "Multiple Choices",
            HttpCode::MovedPermanently => "Moved Permanently",
            HttpCode::Found => "Found",
            HttpCode::SeeOther => "See Other",
            HttpCode::NotModified => "Not Modified",
            HttpCode::UseProxy => "Use Proxy",
            HttpCode::TemporaryRedirect => "Temporary Redirect",
            HttpCode::PermanentRedirect => "Permanent Redirect",

            // 4xx: 客户端错误
            HttpCode::BadRequest => "Bad Request",
            HttpCode::Unauthorized => "Unauthorized",
            HttpCode::PaymentRequired => "Payment Required",
            HttpCode::Forbidden => "Forbidden",
            HttpCode::NotFound => "Not Found",
            HttpCode::MethodNotAllowed => "Method Not Allowed",
            HttpCode::NotAcceptable => "Not Acceptable",
            HttpCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
            HttpCode::RequestTimeout => "Request Timeout",
            HttpCode::Conflict => "Conflict",
            HttpCode::Gone => "Gone",
            HttpCode::LengthRequired => "Length Required",
            HttpCode::PreconditionFailed => "Precondition Failed",
            HttpCode::PayloadTooLarge => "Payload Too Large",
            HttpCode::URITooLong => "URI Too Long",
            HttpCode::UnsupportedMediaType => "Unsupported Media Type",
            HttpCode::RangeNotSatisfiable => "Range Not Satisfiable",
            HttpCode::ExpectationFailed => "Expectation Failed",
            HttpCode::ImATeapot => "I'm a teapot",
            HttpCode::MisdirectedRequest => "Misdirected Request",
            HttpCode::UnprocessableEntity => "Unprocessable Entity",
            HttpCode::Locked => "Locked",
            HttpCode::FailedDependency => "Failed Dependency",
            HttpCode::TooEarly => "Too Early",
            HttpCode::UpgradeRequired => "Upgrade Required",
            HttpCode::PreconditionRequired => "Precondition Required",
            HttpCode::TooManyRequests => "Too Many Requests",
            HttpCode::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            HttpCode::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

            // 5xx: 服务器错误
            HttpCode::InternalServerError => "Internal Server Error",
            HttpCode::NotImplemented => "Not Implemented",
            HttpCode::BadGateway => "Bad Gateway",
            HttpCode::ServiceUnavailable => "Service Unavailable",
            HttpCode::GatewayTimeout => "Gateway Timeout",
            HttpCode::HTTPVersionNotSupported => "HTTP Version Isn't Supported",
            HttpCode::VariantAlsoNegotiates => "Variant Also Negotiates",
            HttpCode::InsufficientStorage => "Insufficient Storage",
            HttpCode::LoopDetected => "Loop Detected",
            HttpCode::NotExtended => "Not Extended",
            HttpCode::NetworkAuthenticationRequired => "Network Authentication Required",

            HttpCode::UnDefined => "Undefined HttpCode",
        }
    }
}
