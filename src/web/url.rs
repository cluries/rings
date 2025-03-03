use std::collections::HashMap;

use tracing::error;




pub fn join(base:&str, other:&str)->String{

    let be = base.ends_with("/");
    let os = other.starts_with("/");
    if be && os{
        return format!("{}{}", base, other[1..].to_string());
    }

    if !be && !os{
        return format!("{}/{}", base, other);
    }

    format!("{}{}", base, other)
}

pub fn url_encode(val: &str) -> String {
    // percent_encoding::utf8_percent_encode(val, percent_encoding::NON_ALPHANUMERIC).collect()
    url::form_urlencoded::byte_serialize(val.as_bytes()).collect::<String>()
}

//
pub fn url_decode(val: &str) -> String {
    percent_encoding::percent_decode(val.as_bytes()).decode_utf8().unwrap_or_default().into()
}

pub fn parse_url_query(url: &str) -> HashMap<String, String> {
    match url::Url::parse(url) {
        Ok(url) => {
            url.query_pairs().into_owned().collect()
        }

        Err(err) => {
            error!("error {} parse:{}", err, url);
            HashMap::new()
        }
    }
}


// parse querystring
pub fn parse_query(query: &str) -> HashMap<String, String> {
    // let query = query.trim();
    // if query.is_empty() {
    //     return HashMap::new();
    // }

    url::form_urlencoded::parse(query.as_bytes()).into_owned().collect()
}




