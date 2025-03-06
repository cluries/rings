
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

struct MakeMigratorArgs {
    name: LitStr,
    structs: Punctuated<Ident, Token![,]>,
}

impl Parse for MakeMigratorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let structs = Punctuated::parse_terminated(input)?;
        Ok(MakeMigratorArgs { name, structs })
    }
}


pub fn using_macros(_input: TokenStream) -> TokenStream {
    let expanded = quote! {

        #[allow(unused_imports)]
        use rings::{migrate_create_tables, migrate_drop_tables};

    };

    TokenStream::from(expanded)
}

 pub fn make_migrator(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as MakeMigratorArgs);

    let names = args.name.value();
    let migrate_ident = Ident::new(&format!("migrate_{}", names), proc_macro2::Span::call_site());

    let structs: Vec<_> = args.structs.into_iter().collect();

    let expanded = quote! {

        #[allow(unused_imports)]
        use sea_orm_migration::{  MigratorTrait };

        #[allow(unused_imports)]
        use log::{error, info, warn};

        struct Migrator;

        #[async_trait::async_trait]
        impl MigratorTrait for Migrator {
            fn migrations() -> Vec<Box<dyn MigrationTrait>> {
                vec![ #( Box::new(#structs), )* ]
            }
        }

        pub async fn #migrate_ident(connect_string:String) {
            const name:&str = #names;

            info!("Migrate Task : {}", name );
            info!("using dbms {:?}", connect_string);

            let db = Database::connect(connect_string).await.unwrap();

            warn!("\t{} starting up...", name);

            if Migrator::up(&db, None).await.is_ok() {
                info!("Migrate Task {} is finished", name);
                return;
            }

            error!("\t{} up  is failed!!!!!!!!!!!", name);

            info!("\t{} is starting task down", name);

            if Migrator::down(&db, None).await.is_err() {
                error!("\t{} down is failed!!!!!!!!!!", name);
            } else {
                info!("\t{} down is finished", name);
            }

            info!("Migrate Task {} is finished", name);
        }
    };

    TokenStream::from(expanded)
}