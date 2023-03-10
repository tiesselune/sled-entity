mod attr;
mod structure;

use attr::EntityData;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput, DataStruct, Visibility, spanned::Spanned};
use quote::quote;
use syn::Ident;

type Errors = Vec<syn::Error>;

#[proc_macro_derive(Entity, attributes(entity,entity_id,siblings,children))]
pub fn derive_entity(item : TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let mut errors = Vec::new();
    let mut result = construct_token_stream(&ast, &mut errors);
    if errors.len() > 0 {
        result.extend::<TokenStream>(errors.iter().map(|e| Into::<TokenStream>::into(e.to_compile_error())).collect());
    }
    result
}

fn construct_token_stream(input : &DeriveInput, errors : &mut Errors) -> TokenStream {
    let mut result = TokenStream::new();
    
    match &input.data {
        syn::Data::Struct(s) => {
            let entity_data = EntityData::parse(&input.span(),&input.attrs,&s.fields, errors);
            let attr_copy = entity_data.clone();
            result.extend([
                generate_alias(&input.ident, entity_data.version.unwrap_or(0), &input.vis),
                generate_impl(s, &attr_copy, errors),
            ])
        },
        syn::Data::Enum(_) => errors.push(syn::Error::new_spanned(input, "Cannot derive Entity on an enum. Please implement Entity manually.")),
        syn::Data::Union(_) => errors.push(syn::Error::new_spanned(input, "Cannot derive Entity on a union. Please implement Entity manually.")),
    }
    result
}

fn generate_alias(name : &Ident,version : u32, vis : &Visibility) -> TokenStream {
    let versionned_ident = Ident::new(&format!("{}_v{}",name.to_string(),version), Span::call_site());
    quote ! {
        #vis type #versionned_ident = #name;
    }.into()
}

fn generate_impl(s : &DataStruct,entity_data : &EntityData, errors : &mut Errors) -> TokenStream {
    TokenStream::new()
}