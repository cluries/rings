use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::{parse_macro_input, ItemStruct};

static SERVICE_MACRO_MARKS: std::sync::Mutex<Vec<(String, String)>> = std::sync::Mutex::new(Vec::new());

pub fn mark(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_str = parse_attr_string(attr);

    let input = parse_macro_input!(item as ItemStruct);
    // let struct_name = input.ident.to_string();

    let mut marks = SERVICE_MACRO_MARKS.lock().unwrap();
    marks.push((attr_str, input.ident.to_string().clone()));

    input.into_token_stream().into()
}

pub fn expand(input: TokenStream) -> TokenStream {
    // Parse the input string
    let input_str = parse_attr_string(input);

    // Look up the struct name associated with this string
    let marks = SERVICE_MACRO_MARKS.lock().unwrap();

    let mut ident = String::new();
    for (attr_str, ist) in marks.iter() {
        ident.push_str(&format!("{} = {} ", attr_str, ist));
    }

    let expanded = quote! {
        use rings;
        let c = rings::service::ServiceManager::shared().await;
        // c.register::<>();

        println!("===={}", c.name());
    };

    expanded.into()
}

fn parse_attr_string(attr: TokenStream) -> String {
    let mut iter = attr.into_iter();
    if let Some(TokenTree::Literal(lit)) = iter.next() {
        let lit_str = lit.to_string();
        // Remove the quotes from the string literal
        lit_str.trim_matches('"').to_string()
    } else {
        panic!("Expected a string literal")
    }
}
