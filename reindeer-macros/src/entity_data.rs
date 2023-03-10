use std::str::FromStr;

use syn::{Attribute, Meta,TypeTuple, Ident, Fields};
use crate::Errors;
use proc_macro2::{Span, TokenStream};

const ID_PARSE_ERROR : &'static str = "Could not parse id parameter. id must be a string containing either a field name, or a tuple of field names.";

#[derive(Clone)]
pub enum IdStructure {
    Simple(syn::Ident),
    Tuple(Vec<IdStructure>)
}

#[derive(Default,Clone)]
pub struct EntityData {
    pub name : Option<String>,
    pub version : Option<u32>,
    pub id : Option<IdStructure>,
    pub children : Vec<(syn::Ident,syn::Ident)>,
    pub siblings : Vec<(syn::Ident,syn::Ident)>,
    pub fields : Vec<(syn::Visibility,syn::Ident,syn::Type)>,
}

impl EntityData {
    pub fn parse(span : &Span, attrs : &[Attribute], fields : &Fields, errors : &mut Errors) -> EntityData {
        let mut entity_data = EntityData::default();
        entity_data.parse_fields( fields, errors);
        for attr in attrs {
            if attr.path.is_ident("entity") {
                match attr.parse_meta(){
                    Ok(meta) => {
                        entity_data.parse_entity_args(&meta, errors);
                    },
                    Err(e) => errors.push(e),
                }
            }
            else if attr.path.is_ident("entity_id") {
                match attr.parse_args::<TypeTuple>() {
                    Ok(v) => {
                        entity_data.id = Some(Self::parse_id_tuple(&v, errors));
                    },
                    Err(_) => {
                        match attr.parse_args::<Ident>() {
                            Ok(i) => {
                                entity_data.id = Some(IdStructure::Simple(i));
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
        entity_data.check(span,errors);
        entity_data
    }

    fn parse_entity_args(&mut self, meta : &Meta, errors : &mut Errors) {
        match meta {
            Meta::Path(p) => {
                errors.push(syn::Error::new_spanned(p, "Unrecognized argument 1"));
            },
            Meta::List(l) => {
                for token in &l.nested {
                    match token {
                        syn::NestedMeta::Meta(m) => {
                            self.parse_entity_args(m, errors);
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
                            self.name = Some(str.value());
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
                                    self.version = Some(int);
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
                            self.parse_id_attr(&str.value(), &str.span(), errors);
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

    fn parse_id_attr(&mut self, str : &str, span : &Span, errors : &mut Errors){
        let tokens = TokenStream::from_str(str);
        match tokens {
            Ok(tokens) => {
                let ident = syn::parse::<Ident>(tokens.clone().into());
                match ident {
                    Ok(ident) => {
                        self.id = Some(IdStructure::Simple(ident.into()));
                    },
                    Err(_)=> {
                        match syn::parse::<syn::TypeTuple>(tokens.into()) {
                            Ok(tuple) => {
                                self.id = Some(Self::parse_id_tuple(&tuple, errors));
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

    fn parse_fields(&mut self, fields : &Fields, errors : &mut Errors) {
        match fields {
            Fields::Named(fields) => {
                for field in fields.named.iter() {
                    let field = field.clone();
                    self.fields.push((field.vis,field.ident.unwrap(),field.ty));
                }
            },
            _ => errors.push(syn::Error::new_spanned(fields, "Reindeer only supports deriving Entity on named structs.")),
        }
    }

    fn check(&mut self, span : &Span, errors : &mut Errors){
        match &self.id {
            None => {
                let id_field = self.fields.iter().find(|e| e.1.to_string() == "id");
                if let Some(id_field) = id_field {
                    self.id = Some(IdStructure::Simple(id_field.1.clone()));
                }
                else {
                    errors.push(syn::Error::new(span.to_owned(), "Missing ID specification. Use either a field called  `id`, the `id` macro meta, or the `entity_id` attribute."));
                }
            },
            Some(id) => {
                self.check_id(id, errors);
            }
        }


    }
    fn check_id(&self, id_struct : &IdStructure, errors : &mut Errors) {
        match id_struct {
            IdStructure::Simple(ident) => {
                if self.fields.iter().find(|e| e.1.to_string() == ident.to_string()).is_none() {
                    errors.push(syn::Error::new(ident.span(), format!("Cannot find referenced field '{}'",ident)));
                }
            },
            IdStructure::Tuple(sub_struct) => {
                for id_part in sub_struct {
                    self.check_id(id_part, errors);
                }
            }
        }
    }

}