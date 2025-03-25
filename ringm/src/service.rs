use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

static SERVICE_MACRO_MARKS: std::sync::Mutex<Vec<(String, String)>> = std::sync::Mutex::new(Vec::new());
static SREVICE_RESOLVES: std::sync::Mutex<Vec<(String, String)>> = std::sync::Mutex::new(Vec::new());


pub fn mark(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = crate::tools::parse_input_string(attr);
    let itemc = item.clone();
    let struct_obj = parse_macro_input!(itemc as DeriveInput);

    let mut marks = SERVICE_MACRO_MARKS.lock().unwrap();

    let function_name_string = format!("ringm_generated_rings_service_register_{}", struct_obj.ident.to_string());
    let function_name = Ident::new(&function_name_string, proc_macro2::Span::call_site());

    // let call_site = proc_macro2::Span::();
    // let s = call_site.unwrap().source_text().unwrap();
    let s = "".to_string();

    marks.push((function_name_string.clone(),s));

    // let info = proc_macro2::Span::call_site();

    let function = quote! {

        //
        pub fn #function_name() {
            let mod_path = module_path!();
            ringm::service_resolve!(#function_name_string, mod_path);

            println!("This is function: {}, args: {} mod:{}", #function_name_string , #attrs, mod_path );
        }
    };

    let mut merged = TokenStream::from(item);
    merged.extend(TokenStream::from(function));
    merged
}

pub fn expand(input: TokenStream) -> TokenStream {
    let attrs = crate::tools::parse_input_string(input);
    let marks = SERVICE_MACRO_MARKS.lock().unwrap();

    let generated_functions = marks.iter().map(|(function_name, mod_path)| {
        // let function_ident = Ident::new(&function_name, proc_macro2::Span::call_site());
        // quote! {
        //     let _ = #function_ident();
        // };
        quote! {
            let _ = #mod_path;
        }
    });

    let expanded = quote! {
        println!("This is service expand: {}", #attrs);
        {
            #(#generated_functions)*
        }
    };

    expanded.into()
}


pub fn resolve(attr: TokenStream) -> TokenStream {
    let attrs = crate::tools::parse_input_string(attr);
    let mut resolves = SREVICE_RESOLVES.lock().unwrap();
    resolves.push((attrs[0], attrs[1]));

    let expanded = quote! {

    };

    expanded.into()
}