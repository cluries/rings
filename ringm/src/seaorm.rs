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

    // Retriever
    // Repository

    let retrieve = format_ident!("{}Finder", name);
    let persist = format_ident!("{}Mutator", name);

    let model_alias = format_ident!("{}Mod", name);
    let entity_alias = format_ident!("{}Ent", name);
    let column_alias = format_ident!("{}Col", name);
    let active_model_alias = format_ident!("{}Act", name);

    let expanded = quote! {

        #[doc = "Finder struct for handling data queries."]
        pub struct #retrieve;

        #[doc = "Mutator struct for handling data persistence and CRUD operations."]
        pub struct #persist;

        pub type #model_alias = crate::entity::#predications::Model;
        pub type #entity_alias = crate::entity::#predications::Entity;
        pub type #column_alias = crate::entity::#predications::Column;
        pub type #active_model_alias = crate::entity::#predications::ActiveModel;

    };

    expanded.into()
}
