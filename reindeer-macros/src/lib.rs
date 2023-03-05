use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{ItemStruct};
use quote::{quote,ToTokens};
use syn::Ident;

#[proc_macro_attribute]
pub fn entity(args : TokenStream, item : TokenStream) -> TokenStream {
    let ast: Result<ItemStruct,syn::Error> = syn::parse(item);
    if let Err(e) = ast {
        panic!("The entity attribute must be applied on a struct.")
    }
    let ast = ast.unwrap();
    let struct_name = ast.ident.clone();
    let versionned_name = Ident::new(&format!("{}_v{}",struct_name,1), Span::call_site());
    quote! {
        #ast
        pub type #versionned_name = #struct_name;
    }.into_token_stream().into()
}