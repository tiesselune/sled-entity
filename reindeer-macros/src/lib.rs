//! Derive macro for `reindeer`'s 🦌 `Entity` trait.
//!
//! To automatically derive Entity on a `struct`, you simply have to derive `Entity` (as Well as `serde`'s `Serialize` and `Deserialize` traits) like so:
//!
//! ```rust
//! #[derive(Serialize,Deserialize,Entity)]
//! struct User {
//!     id : (u32,u32),
//!     email : String,
//!     username : String,
//!     last_login : i64,
//!     password_hash : String,
//! }
//! ```
//!
//! ☝😉 This will generate an `Entity` implementation with store name `User`, version 0, and key being the `key` field.
//! To specify other values, use the helper attribute `entity` like so :
//!
//! ```rust
//! #[derive(Serialize,Deserialize,Entity)]
//! #[entity(name = "user", version = 1,key = "email")]
//! struct User {
//!     email : String,
//!     username : String,
//!     last_login : i64,
//!     password_hash : String,
//! }
//! ```
//!
//! To specify sibling entities and child entities, use the `sibling` and `child` helper attributes
//! respectively:
//!
//! ```rust
//! #[derive(Serialize,Deserialize,Entity)]
//! #[entity(name = "user", version = 1,key = "email")]
//! #[sibling(("user_data", Cascade))]
//! #[children(("doc",Cascade),("shared_doc",BreakLink))]
//! struct User {
//!     email : String,
//!     username : String,
//!     last_login : i64,
//!     password_hash : String,
//! }
//!
//! //! #[derive(Serialize,Deserialize,Entity)]
//! #[entity(name = "user_data", version = 1,key = "email")]
//! #[sibling(("user", Error))]
//! struct UserData {
//!     email : String,
//!     username : String,
//!     last_login : i64,
//!     password_hash : String,
//! }
//! ```
//!
//! The second part of each relation is a `reindeer::DeletionBehaviour` enum value : `BreakLink`,`Cascade`, or `Error`.
//!
mod entity_data;
mod relations;
mod schema;

use entity_data::EntityData;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use schema::save_schema;
use syn::Ident;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Visibility};

type Errors = Vec<syn::Error>;

///
/// Derive macro for `reindeer`'s 🦌 `Entity` trait.
///
/// To automatically derive Entity on a `struct`, you simply have to derive `Entity` (as Well as `serde`'s `Serialize` and `Deserialize` traits) like so:
///
/// ```rust
/// #[derive(Serialize,Deserialize,Entity)]
/// struct User {
///     key : (u32,u32),
///     email : String,
///     username : String,
///     last_login : i64,
///     password_hash : String,
/// }
/// ```
///
/// ☝😉 This will generate an `Entity` implementation with store name `User`, version 0, and key being the `key` field.
/// To specify other values, use the helper attribute `entity` like so :
///
/// ```rust
/// #[derive(Serialize,Deserialize,Entity)]
/// #[entity(name = "user", version = 1,key = "email")]
/// struct User {
///     email : String,
///     username : String,
///     last_login : i64,
///     password_hash : String,
/// }
/// ```
///
/// To specify sibling entities and child entities, use the `sibling` and `child` helper attributes
/// respectively:
///
/// ```rust
/// #[derive(Serialize,Deserialize,Entity)]
/// #[entity(name = "user", version = 1,key = "email")]
/// #[siblings(("user_data", Cascade))]
/// #[children(("doc",Cascade),("shared_doc",BreakLink))]
/// struct User {
///     email : String,
///     username : String,
///     last_login : i64,
///     password_hash : String,
/// }
///
/// #[derive(Serialize,Deserialize,Entity)]
/// #[entity(name = "user_data", version = 1,key = "email")]
/// #[siblings(("user", Error))]
/// struct UserData {
///     email : String,
///     username : String,
///     last_login : i64,
///     password_hash : String,
/// }
/// ```
///
/// The second part of each relation is a `reindeer::DeletionBehaviour` enum value : `BreakLink`,`Cascade`, or `Error`.
///
#[proc_macro_derive(Entity, attributes(entity, children, siblings))]
pub fn derive_entity(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let mut errors = Vec::new();
    let mut result = construct_token_stream(&ast, &mut errors);
    if !errors.is_empty() {
        result.extend::<TokenStream>(
            errors
                .iter()
                .map(|e| Into::<TokenStream>::into(e.to_compile_error()))
                .collect(),
        );
    }
    result
}

fn construct_token_stream(input: &DeriveInput, errors: &mut Errors) -> TokenStream {
    let mut result = TokenStream::new();

    match &input.data {
        syn::Data::Struct(s) => {
            let entity_data = EntityData::parse(&input.span(), &input.attrs, &s.fields, errors);
            let attr_copy = entity_data.clone();
            result.extend([
                generate_alias(
                    &input.ident,
                    entity_data.version.unwrap_or(0),
                    &input.vis,
                    &input.generics,
                ),
                generate_impl(&input.ident, &attr_copy, &input.generics),
            ]);
            save_schema(&entity_data,errors);
        }
        syn::Data::Enum(_) => errors.push(syn::Error::new_spanned(
            input,
            "Cannot derive Entity on an enum. Please implement Entity manually.",
        )),
        syn::Data::Union(_) => errors.push(syn::Error::new_spanned(
            input,
            "Cannot derive Entity on a union. Please implement Entity manually.",
        )),
    }
    result
}

fn generate_alias(
    name: &Ident,
    version: u32,
    vis: &Visibility,
    generics: &syn::Generics,
) -> TokenStream {
    let (_, ty_generics, _) = generics.split_for_impl();
    let versionned_ident = Ident::new(
        &format!("{}_v{}", name, version),
        Span::call_site(),
    );
    quote! {
        #vis type #versionned_ident #ty_generics = #name #ty_generics;
    }
    .into()
}

fn generate_impl(
    struct_name: &Ident,
    entity_data: &EntityData,
    generics: &syn::Generics,
) -> TokenStream {
    if let (Some(store_name), Some(id_field), Some(key_type), crate_name) = (
        &entity_data.name,
        &entity_data.key,
        &entity_data.key_type,
        &entity_data.crate_name,
    ) {
        let crate_name = Ident::new(crate_name, Span::call_site());
        let children: Vec<proc_macro2::TokenStream> = entity_data
            .children
            .0
            .iter()
            .map(|e| {
                let (name, deletion) = (e.0.clone(), e.1.clone());
                quote! {(#name,#crate_name::DeletionBehaviour::#deletion)}
            })
            .collect();
        let siblings: Vec<proc_macro2::TokenStream> = entity_data
            .siblings
            .0
            .iter()
            .map(|e| {
                let (name, deletion) = (e.0.clone(), e.1.clone());
                quote! {(#name,#crate_name::DeletionBehaviour::#deletion)}
            })
            .collect();
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        quote! {
            impl #impl_generics #crate_name::Entity for #struct_name #ty_generics #where_clause {
                type Key = #key_type;
                fn store_name() -> &'static str {
                    #store_name
                }
                fn get_key(&self) -> &Self::Key {
                    &self.#id_field
                }
                fn set_key(&mut self, key : &Self::Key) {
                    self.#id_field = key.clone();
                }
                fn get_child_stores() -> Vec<(&'static str, #crate_name::DeletionBehaviour)> {
                    vec![#(#children,)*]
                }
                fn get_sibling_stores() -> Vec<(&'static str, #crate_name::DeletionBehaviour)> {
                    vec![#(#siblings,)*]
                }
            }
        }
        .into()
    } else {
        TokenStream::new()
    }
}
