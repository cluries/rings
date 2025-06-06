use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, Token};

struct DefineMoArgs {
    name: Ident,
    predications: Ident,
}

impl Parse for DefineMoArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let predications = input.parse()?;
        Ok(DefineMoArgs { name, predications })
    }
}

pub(crate) fn define_normals(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DefineMoArgs);

    let name = input.name;
    let predications = input.predications;

    let retrieve = format_ident!("{}Retriever", name);
    let persist = format_ident!("{}Repository", name);

    let m_alias = format_ident!("{}Mde", name);
    let e_alias = format_ident!("{}Ent", name);
    let c_alias = format_ident!("{}Col", name);
    let a_alias = format_ident!("{}Amd", name);

    let expanded = quote! {
        pub struct #retrieve;
        pub struct #persist;

        pub type #m_alias = crate::entity::#predications::Model;
        pub type #e_alias = crate::entity::#predications::Entity;
        pub type #c_alias = crate::entity::#predications::Column;
        pub type #a_alias = crate::entity::#predications::ActiveModel;

    };

    expanded.into()
}
