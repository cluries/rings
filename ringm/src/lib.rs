use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

mod db;
mod migrate;
mod service;

#[proc_macro]
pub fn migrate_using_macros(input: TokenStream) -> TokenStream {
    migrate::using_macros(input)
}

#[proc_macro]
pub fn migrate_make_migrator(input: TokenStream) -> TokenStream {
    migrate::make_migrator(input)
}

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    service::mark(attr, item)
}

#[proc_macro]
pub fn service_load(input: TokenStream) -> TokenStream {
    service::expand(input)
}
