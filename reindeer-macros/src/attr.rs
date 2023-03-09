use std::str::FromStr;

use syn::{Attribute, Meta,TypeTuple, Ident};
use crate::Errors;
use proc_macro2::{Span, TokenStream};

const ID_PARSE_ERROR : &'static str = "Could not parse id parameter. id must be a string containing either a field name, or a tuple of field names.";

#[derive(Clone)]
pub enum IdStructure {
    Simple(syn::Ident),
    Tuple(Vec<IdStructure>)
}

#[derive(Default,Clone)]
pub struct EntityAttributeData {
    pub name : Option<String>,
    pub version : Option<u32>,
    pub id : Option<IdStructure>,
    pub children : Vec<(syn::Ident,syn::Ident)>,
    pub siblings : Vec<(syn::Ident,syn::Ident)>,
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
            else if attr.path.is_ident("entity_id") {
                match attr.parse_args::<TypeTuple>() {
                    Ok(v) => {
                        attribute_data.id = Some(Self::parse_id_tuple(&v, errors));
                    },
                    Err(e) => {
                        match attr.parse_args::<Ident>() {
                            Ok(i) => {
                                attribute_data.id = Some(IdStructure::Simple(i));
                            },
                            Err(_) => {
                                errors.push(syn::Error::new_spanned(attr, ID_PARSE_ERROR));
                            }
                        }
                        
                    },
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
                errors.push(syn::Error::new_spanned(p, "Unrecognized argument 1"));
            },
            Meta::List(l) => {
                for token in &l.nested {
                    match token {
                        syn::NestedMeta::Meta(m) => {
                            Self::parse_entity_args(m, attribute_data, errors);
                        },
                        syn::NestedMeta::Lit(l) => {
                            errors.push(syn::Error::new_spanned(l, "Unrecognized argument 2"));
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
                    match &nv.lit {
                        syn::Lit::Str(str) => {
                            Self::parse_id_attr(&str.value(), &str.span(), attribute_data, errors);
                        },
                        _ => {
                            errors.push(syn::Error::new_spanned(&nv.lit, "Store name must be a string."))
                        }
                    }
                }
                else {
                    errors.push(syn::Error::new_spanned(&nv.path, "Unknown parameter"))
                }
            },
        }
    }

    fn parse_id_attr(str : &str, span : &Span, attribute_data : &mut EntityAttributeData, errors : &mut Errors){
        let tokens = TokenStream::from_str(str);
        match tokens {
            Ok(tokens) => {
                let ident = syn::parse::<Ident>(tokens.clone().into());
                match ident {
                    Ok(ident) => {
                        attribute_data.id = Some(IdStructure::Simple(ident.into()));
                    },
                    Err(_)=> {
                        match syn::parse::<syn::TypeTuple>(tokens.into()) {
                            Ok(tuple) => {
                                attribute_data.id = Some(Self::parse_id_tuple(&tuple, errors));
                            },
                            Err(_) => {
                                errors.push(syn::Error::new(span.to_owned(), ID_PARSE_ERROR))
                            }
                        }
                    }
                }
            },
            Err(_) => {
                errors.push(syn::Error::new(span.to_owned(), ID_PARSE_ERROR))
            }
        }
        
    }
    
    fn parse_id_tuple(tuple : &TypeTuple, errors : &mut Errors) -> IdStructure {
        let mut result = Vec::new();
        for elem in &tuple.elems {
            match elem {
                syn::Type::Path(p) => {
                    if p.path.segments.len() == 1 {
                        result.push(IdStructure::Simple(p.path.segments[0].ident.clone()));
                    }
                    else {
                        errors.push(syn::Error::new_spanned(p, "Elements must be field names, or tuples of field names."))
                    }
                },
                syn::Type::Tuple(t) => {
                    result.push(Self::parse_id_tuple(t,errors));
                },
                _ => errors.push(syn::Error::new_spanned(elem, "Elements must be field names, or tuples of field names.")),
            }
        }
        if result.len() == 0 {
            errors.push(syn::Error::new_spanned(tuple, "Could not parse id parameter. id must be a string containing either a field name, or a tuple of field names."))
        }
        IdStructure::Tuple(result)
    }
}