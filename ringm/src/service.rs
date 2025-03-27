use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

static SERVICE_MACRO_MARKS: std::sync::RwLock<Vec<(String, String)>> = std::sync::RwLock::new(Vec::new());

#[allow(unused)]
static SERVICE_MACRO_RESOLVES: std::sync::RwLock<Vec<(String, String)>> = std::sync::RwLock::new(Vec::new());

///
pub(crate) fn mark(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = crate::tools::parse_input_string_vec(attr);
    let args_json = rings::tools::json::Enc::en(&args).unwrap();

    let itemc = item.clone();
    let struct_obj = parse_macro_input!(itemc as DeriveInput);

    let struct_ident = struct_obj.ident.clone();
    let struct_name = struct_obj.ident.to_string();
    let func_name = format!("ringm_generated_rings_service_register_{}", struct_name.to_lowercase());
    let func_ident = Ident::new(&func_name, proc_macro2::Span::call_site());

    SERVICE_MACRO_MARKS.write().unwrap().push((func_name.clone(), args_json));

    let input_module = if args.len() > 0 { args[0].clone() } else { "".to_string() };


    let defaults = quote! {
        #[derive(Default)]
    };

    let function = quote! {
        pub async fn #func_ident() {
            // let _in_module = module_path!();

            ringm::service_resolve!(#func_name, #input_module);
            rings::service::registe_to_shared::<#struct_ident>();
        }
    };

    let mut merged = TokenStream::new();
    merged.extend(TokenStream::from(defaults));
    merged.extend(TokenStream::from(item));
    merged.extend(TokenStream::from(function));
    merged
}

/// input args:
/// root crate ?
pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let args = crate::tools::parse_input_string_vec(input);

    // let resolved = SERVICE_MACRO_RESOLVES.read().unwrap();
    let prefix_module = if args.is_empty() { "".to_string() } else { format!("{}::", args.join("::")) };

    let marks = SERVICE_MACRO_MARKS.read().unwrap();
    let generated_functions = marks.iter().map(|(function_name, mode_path)| {
        let func_ident = Ident::new(&function_name, proc_macro2::Span::call_site());
        let module = format!("{}{}", &prefix_module, &mode_path);
        // let module_ident = Ident::new(&module, proc_macro2::Span::call_site());

        quote! {
            {
                // use #module_ident;
                let _expand_mod_path = #module;
                let _expand = #func_ident();
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
pub(crate) fn resolve(input: TokenStream) -> TokenStream {
    let args = crate::tools::parse_input_string_vec(input);
    let resolveed = (args[0].clone(), args[1].clone());
    SERVICE_MACRO_RESOLVES.write().unwrap().push(resolveed);

    let _mod_path = module_path!();
    let vars = format!("Fn/Module {}::{}", args[1], _mod_path.to_string());

    let expanded = quote! {
        let _resolve_debug = #vars;
        let _resolve_mod_path = module_path!();
    };

    expanded.into()
}
