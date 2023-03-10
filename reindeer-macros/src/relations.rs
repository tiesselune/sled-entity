use proc_macro2::{TokenTree, Ident};
use quote::ToTokens;
use syn::{parse::Parse, parenthesized, punctuated::Punctuated, Token, LitStr};

#[derive(Clone)]
pub struct Relation(syn::LitStr,syn::Ident);

impl Parse for Relation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let result = Punctuated::<TokenTree,Token![,]>::parse_separated_nonempty(&content)?;
        if result.len() != 2 {
            return Err(syn::Error::new(input.span(), format!(r#"A relation must respect the syntax ("store_name",Cascade) {}"#,result.len())))
        }
        let res1 = result[0].clone().into_token_stream().into();
        let res2 = result[1].clone().into_token_stream().into();
        match (syn::parse::<LitStr>(res1),syn::parse::<Ident>(res2)) {
            (Ok(name),Ok(deletion)) => {
                match &*deletion.to_string() {
                    "Cascade" | "BreakLink" | "Error" => Ok(Self(name,deletion)),
                    _ => Err(syn::Error::new_spanned(deletion, r#"The second part of the relation must be either Cascade, BreakLink or Error"#)),
                }
            },
            (Err(e),Ok(_)) => Err(syn::Error::new(e.span(), r#"The first part of the relation must be the store name as a string."#)),
            (Ok(_),Err(e)) => Err(syn::Error::new(e.span(), r#"The second part of the relation must be either Cascade, BreakLink or Error"#)),
            _ => {
                Err(syn::Error::new(input.span(), r#"A relation must respect the syntax ("store_name",Cascade)"#))
            }
        }
    }
}

#[derive(Default,Clone)]
pub struct Relations(Vec<Relation>);

impl Parse for Relations {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let result = Punctuated::<Relation,Token!(,)>::parse_separated_nonempty(input)?;
        Ok(Relations(result.into_iter().collect()))
    }
}