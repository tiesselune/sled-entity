use proc_macro::Ident;
use syn::{Attribute, Meta};
use crate::Errors;

#[derive(Clone)]
pub enum IdStructure {
    Simple(Ident),
    Tuple(Box<(IdStructure,IdStructure)>)
}

#[derive(Default,Clone)]
pub struct EntityAttributeData {
    pub name : Option<String>,
    pub version : Option<u32>,
    pub id : Option<IdStructure>,
    pub children : Vec<Ident>,
    pub siblings : Vec<Ident>,
}

impl EntityAttributeData {
    pub fn parse(attrs : &[Attribute], errors : &mut Errors) -> EntityAttributeData {
        let mut attribute_data = EntityAttributeData::default();
        for attr in attrs {
            if attr.path.is_ident("entity") {
                match attr.parse_meta(){
                    Ok(meta) => {
                        Self::parse_entity_args(&meta, &mut attribute_data, errors);
                    },
                    Err(e) => errors.push(e),
                }
            }
            else if attr.path.is_ident("children") {

            }
            else if attr.path.is_ident("siblings") {
                
            }
        }
        attribute_data
    }

    fn parse_entity_args(meta : &Meta, attribute_data : &mut EntityAttributeData, errors : &mut Errors) {
        match meta {
            Meta::Path(p) => {
                errors.push(syn::Error::new_spanned(p, "Unrecognized argument"));
            },
            Meta::List(l) => {
                for token in &l.nested {
                    match token {
                        syn::NestedMeta::Meta(m) => {
                            Self::parse_entity_args(m, attribute_data, errors);
                        },
                        syn::NestedMeta::Lit(l) => {
                            errors.push(syn::Error::new_spanned(l, "Unrecognized argument"));
                        },
                    }
                }
            },
            Meta::NameValue(nv) => {
                if nv.path.is_ident("name") {
                    match &nv.lit {
                        syn::Lit::Str(str) => {
                            attribute_data.name = Some(str.value());
                        },
                        _ => {
                            errors.push(syn::Error::new_spanned(&nv.lit, "Store name must be a string."))
                        }
                    }
                }
                else if nv.path.is_ident("version") {
                    match &nv.lit {
                        syn::Lit::Int(int) => {
                            match int.base10_parse::<u32>() {
                                Ok(int) => {
                                    attribute_data.version = Some(int);
                                },
                                Err(_) => {
                                    errors.push(syn::Error::new_spanned(&int, "Store version must be a positive integer."))
                                },
                            }
                        },
                        _ => {
                            errors.push(syn::Error::new_spanned(&nv.lit, "Store name must be a string."))
                        }
                    }
                }
                else if nv.path.is_ident("id") {
                    
                }
            },
        }
    }
}