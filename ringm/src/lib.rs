use proc_macro::TokenStream;
use syn::parse::Parse;


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


//
// #[proc_macro_attribute]
// pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
//
//     // 解析属性参数
//     let attr_args = parse_macro_input!(attr as AttributeArgs);
//     let mut custom_name = None;
//
//     for arg in attr_args {
//         if let NestedMeta::Meta(Meta::NameValue(nv)) = arg {
//             if nv.path.is_ident("name") {
//                 if let syn::Lit::Str(lit) = nv.lit {
//                     custom_name = Some(lit.value());
//                 }
//             }
//         }
//     }
//
//     // 解析结构体
//     let input = parse_macro_input!(item as ItemStruct);
//     let struct_name = &input.ident;
//
//     // 使用参数生成代码
//     let greeting = custom_name.unwrap_or_else(|| struct_name.to_string());
//     let expanded = quote! {
//         #input
//
//         impl #struct_name {
//             pub fn greet(&self) {
//                 println!("Custom greeting: {}", #greeting);
//             }
//         }
//     };
//
//     TokenStream::from(expanded)
//
// }