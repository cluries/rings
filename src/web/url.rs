use std::collections::HashMap;
use tracing::error;

/// Join two url path
pub fn join(base: &str, other: &str) -> String {
    let be = base.ends_with("/");
    let os = other.starts_with("/");
    if be && os {
        return format!("{}{}", base, other[1..].to_string());
    }

    if !be && !os {
        return format!("{}/{}", base, other);
    }

    format!("{}{}", base, other)
}

/// Url encode
pub fn url_encode(val: &str) -> String {
    // percent_encoding::utf8_percent_encode(val, percent_encoding::NON_ALPHANUMERIC).collect()
    url::form_urlencoded::byte_serialize(val.as_bytes()).collect::<String>()
}

/// Url decode
pub fn url_decode(val: &str) -> String {
    percent_encoding::percent_decode(val.as_bytes()).decode_utf8().unwrap_or_default().into()
}

/// Parse url query string, return a hashmap
/// all value will be decoded to string
/// url must be a valid url
/// then get the query string and parse it
pub fn parse_url_query(url: &str) -> HashMap<String, String> {
    match url::Url::parse(url) {
        Ok(url) => url.query_pairs().into_owned().collect(),

        Err(err) => {
            error!("error {} parse:{}", err, url);
            HashMap::new()
        },
    }
}

/// Parse query string, return a hashmap
/// it's different from parse_url_query, because it's not url
///
pub fn parse_query(query: &str) -> HashMap<String, String> {
    // let query = query.trim();
    // if query.is_empty() {
    //     return HashMap::new();
    // }

    url::form_urlencoded::parse(query.as_bytes()).into_owned().collect()
}


pub fn get_query_value(query:&str, name:&str) -> Option<String> {
    let query = parse_query(query);
    query.get(name).cloned()
}

