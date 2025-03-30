use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn default_any(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // let args = crate::tools::parse_input_string_vec(attr);

    let itemc = item.clone();
    let struct_obj = parse_macro_input!(itemc as DeriveInput);

    let struct_ident = struct_obj.ident.clone();
    // let struct_name = struct_obj.ident.to_string();

    let any_impl = quote! {

        // gend
        impl rings::any::AnyTrait for #struct_ident {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };

    let mut merged = TokenStream::new();
    merged.extend(TokenStream::from(item));
    merged.extend(TokenStream::from(any_impl));
    merged
}
