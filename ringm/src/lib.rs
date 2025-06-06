use proc_macro::TokenStream;

mod any;
mod db;
mod migrate;
mod service;
mod tools;
mod seaorm;

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
pub fn serviced(input: TokenStream) -> TokenStream {
    service::expand(input)
}

#[cfg(feature = "serivce_macro_use_func")]
#[proc_macro]
pub fn service_resolve(input: TokenStream) -> TokenStream {
    service::resolve(input)
}

#[proc_macro_attribute]
pub fn default_any(attr: TokenStream, item: TokenStream) -> TokenStream {
    any::default_any(attr, item)
}

#[proc_macro]
pub fn seaorm_mo(input: TokenStream) -> TokenStream {
    seaorm::define_normals(input)
}