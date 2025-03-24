use proc_macro::TokenStream;

mod db;
mod migrate;

#[proc_macro]
pub fn migrate_using_macros(input: TokenStream) -> TokenStream {
    migrate::using_macros(input)
}

#[proc_macro]
pub fn migrate_make_migrator(input: TokenStream) -> TokenStream {
    migrate::make_migrator(input)
}
