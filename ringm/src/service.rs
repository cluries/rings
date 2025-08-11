use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

static SERVICE_MACRO_MARKS: std::sync::RwLock<Vec<(String, Vec<String>)>> = std::sync::RwLock::new(Vec::new());

#[allow(unused)]
static SERVICE_MACRO_RESOLVES: std::sync::RwLock<Vec<(String, String)>> = std::sync::RwLock::new(Vec::new());

fn join_crate(parts: &Vec<String>) -> String {
    let parts: Vec<String> = parts.iter().filter(|s| s.trim().len() > 0).map(|x| x.clone()).collect();
    if parts.len() > 0 {
        parts.join("::").to_string()
    } else {
        String::new()
    }
}

#[cfg(feature = "serivce_macro_use_func")]
pub(crate) fn mark(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = crate::tools::parse_input_string_vec(attr);

    let itemc = item.clone();
    let struct_obj = parse_macro_input!(itemc as DeriveInput);

    let struct_ident = struct_obj.ident.clone();

    let struct_name = struct_obj.ident.to_string();

    let func_name = format!("ringm_generated_rings_service_register_{}", struct_name.to_lowercase());
    let func_ident = Ident::new(&func_name, proc_macro2::Span::call_site());

    SERVICE_MACRO_MARKS.write().unwrap().push((func_name.clone(), args.clone()));

    let defaults = quote! {
        #[derive(Default)]
    };

    let mut merged = TokenStream::new();
    merged.extend(TokenStream::from(defaults));
    merged.extend(TokenStream::from(item));

    let input_module = join_crate(&args);

    let function = quote! {
        pub async fn #func_ident() {
            let _in_module = module_path!();

            ringm::service_resolve!(#func_name, #input_module);
            rings::service::registe_to_shared::<#struct_ident>().await;

            // rings::prelude::tracing::info!("---{} {}, {:?}",#func_name, #struct_ident, _in_module);
            println!("Service registered with function: {}", #func_name);
            rings::prelude::tracing::info!("Service registered with function: {}", #func_name);
        }
    };

    merged.extend(TokenStream::from(function));

    merged
}

#[cfg(not(feature = "serivce_macro_use_func"))]
pub(crate) fn mark(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = crate::tools::parse_input_string_vec(attr);

    let itemc = item.clone();
    let struct_obj = parse_macro_input!(itemc as DeriveInput);

    let struct_name = struct_obj.ident.to_string();
    SERVICE_MACRO_MARKS.write().unwrap().push((struct_name, args.clone()));

    let defaults = quote! {
        #[derive(Default)]
    };

    let mut merged = TokenStream::new();
    merged.extend(TokenStream::from(defaults));
    merged.extend(TokenStream::from(item));

    merged
}

/// input args:
/// root crate ?

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let args = crate::tools::parse_input_string_vec(input);

    // let resolved = SERVICE_MACRO_RESOLVES.read().unwrap();
    let prefix_module = if args.is_empty() { "crate".to_string() } else { args.join("::") };

    let marks = SERVICE_MACRO_MARKS.read().unwrap();
    let generated_functions = marks.iter().map(|(ident_name, module_args)| {
        let mode_path = join_crate(module_args);
        let module = join_crate(&vec![prefix_module.clone(), mode_path.clone(), ident_name.clone()]);

        let using = syn::parse_str::<syn::Path>(&module).unwrap();
        let using_quote = if !mode_path.is_empty() {
            quote! {
                use #using;
            }
        } else {
            quote! {}
        };

        #[cfg(feature = "serivce_macro_use_func")]
        {
            let func_ident = Ident::new(&ident_name, proc_macro2::Span::call_site());
            quote! {
                {
                    #using_quote
                    let _expand = #func_ident().await;
                }
            }
        }

        #[cfg(not(feature = "serivce_macro_use_func"))]
        {
            let struct_ident = Ident::new(&ident_name, proc_macro2::Span::call_site());
            quote! {
                {
                    #using_quote;
                    rings::service::registe_to_shared::<#struct_ident>().await;
                    println!("Service registered with directy: {}", #ident_name);
                    rings::prelude::tracing::info!("Service registered with directy: {}", #ident_name);
                }
            }
        }
    });

    let expanded = quote! {
        {
            #(#generated_functions)*
        }
    };

    expanded.into()
}

/// input args:
/// function_name, module_path
///
#[cfg(feature = "serivce_macro_use_func")]
pub(crate) fn resolve(input: TokenStream) -> TokenStream {
    let args = crate::tools::parse_input_string_vec(input);
    let resolveed = (args[0].clone(), args[1].clone());
    SERVICE_MACRO_RESOLVES.write().unwrap().push(resolveed);

    let _mod_path = module_path!();
    // let vars = format!("Fn/Module {}::{}", args[1], _mod_path.to_string());

    let expanded = quote! {
        // let _resolve_debug = #vars;
        let _resolve_mod_path = module_path!();
    };

    expanded.into()
}
