use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::ItemStruct;
use quote::{quote,ToTokens};
use syn::Ident;
use std::fs::{write,create_dir_all};
use std::path::PathBuf;

#[proc_macro_attribute]
pub fn entity(args : TokenStream, item : TokenStream) -> TokenStream {
    let ast: Result<ItemStruct,syn::Error> = syn::parse(item);
    if let Err(_) = ast {
        let content : TokenStream = quote! {
            compile_error!("'entity' can only be applied on a valid struct.");
        }.to_token_stream().into();
        //content.extend([item]);
        return content;
    }
    let ast = ast.unwrap();
    let struct_name = ast.ident.clone();
    let versionned_name = Ident::new(&format!("{}_v{}",struct_name,1), Span::call_site());
    let dir = env!("CARGO_MANIFEST_DIR");
    let mut migrations_dir = PathBuf::new();
    migrations_dir.push(dir);
    migrations_dir.push("schema");
    create_dir_all(&migrations_dir).unwrap();
    migrations_dir.push(&format!("{}_v{}.json",struct_name,1));
    write(&migrations_dir,"Hello, world!").unwrap();
    quote! {
        #ast
        type #versionned_name = #struct_name;
    }.into_token_stream().into()
}