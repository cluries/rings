
use proc_macro::{TokenStream, TokenTree};




/// parse input args
#[allow(unused)]
pub(crate) fn parse_input_string(attr: TokenStream) -> String {
    parse_input_string_vec(attr).first().unwrap().to_string()
}


#[allow(unused)]
pub(crate) fn parse_input_string_vec(attr: TokenStream) -> Vec<String> {
    attr.into_iter()
        .filter(|x| match x {
            TokenTree::Ident(_) => true,
            TokenTree::Literal(_) => true,
            _ => false,
        })
        .map(|x| match x {
            TokenTree::Literal(lit) => lit.to_string().trim_matches('"').to_string(),
            TokenTree::Ident(ident) => ident.to_string(),
            TokenTree::Punct(punct) => punct.as_char().to_string(),
            TokenTree::Group(group) => group.stream().to_string(),
        })
        .collect()
}

#[allow(unused)]
pub(crate) fn parse_input_string_vec_directly(attr: TokenStream) -> Vec<String> {
    fn item(x: &str) -> String {
        x.trim().trim_matches('"').to_string()
    }

    let attr = attr.to_string();
    let parts: Vec<String> = attr.split(',').map(item).collect();
    parts
}
