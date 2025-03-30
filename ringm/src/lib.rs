use proc_macro::TokenStream;

mod any;
mod db;
mod migrate;
mod service;
mod tools;

#[proc_macro]
pub fn migrate_using_macros(input: TokenStream) -> TokenStream {
    migrate::using_macros(input)
}

#[proc_macro]
pub fn migrate_make_migrator(input: TokenStream) -> TokenStream {
    migrate::make_migrator(input)
}

// #[proc_macro_attribute]
// pub fn ringm_service(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mod_path = module_path!();
//     let output = quote! {
//         #[ringm::service("abc")]
//     };
//
//     let mut r = TokenStream::new();
//     r.extend(TokenStream::from(output));
//     r.extend(item);
//
//     r.into()
// }

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    service::mark(attr, item)
}

#[proc_macro]
pub fn serviced(input: TokenStream) -> TokenStream {
    service::expand(input)
}

#[cfg(feature = "use_func_register")]
#[proc_macro]
pub fn service_resolve(input: TokenStream) -> TokenStream {
    service::resolve(input)
}

#[proc_macro_attribute]
pub fn default_any(attr: TokenStream, item: TokenStream) -> TokenStream {
    any::default_any(attr, item)
}
