use proc_macro::{TokenStream, TokenTree};

/// parse input args
pub(crate) fn parse_input_string(attr: TokenStream) -> String {
    parse_input_string_vec(attr).first().unwrap().to_string()
}

pub(crate) fn parse_input_string_vec(attr: TokenStream) -> Vec<String> {
    attr.into_iter()
        .map(|x| match x {
            TokenTree::Literal(lit) => lit.to_string().trim_matches('"').to_string(),
            _ => {
                panic!("Expected a string literal")
            },
        })
        .collect()
}
