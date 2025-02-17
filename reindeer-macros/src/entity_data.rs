use std::str::FromStr;

use syn::{Attribute, Meta,Ident, Fields};
use crate::Errors;
use proc_macro2::{Span, TokenStream};
use crate::relations::Relations;

const ID_PARSE_ERROR : &'static str = "Could not parse id parameter. id must be a string containing either a field name.";



#[derive(Default,Clone)]
pub struct EntityData {
    pub crate_name : String,
    pub name : Option<String>,
    pub version : Option<u32>,
    pub id : Option<Ident>,
    pub id_type : Option<syn::Type>,
    pub children : Relations,
    pub siblings : Relations,
    pub fields : Vec<(syn::Visibility,syn::Ident,syn::Type)>,
}

impl EntityData {
    pub fn parse(span : &Span, attrs : &[Attribute], fields : &Fields, errors : &mut Errors) -> EntityData {
        let mut entity_data = EntityData::default();
        entity_data.crate_name = "reindeer".to_string();
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
            else if attr.path.is_ident("children") || attr.path.is_ident("siblings") {
                entity_data.parse_related_stores(attr,errors);
            }
        }
        entity_data.check(span,errors);
        entity_data
    }

    fn parse_entity_args(&mut self, meta : &Meta, errors : &mut Errors) {
        match meta {
            Meta::Path(p) => {
                errors.push(syn::Error::new_spanned(p, "Unrecognized argument. Accepted arguments are 'name', 'version' and 'id'"));
            },
            Meta::List(l) => {
                for token in &l.nested {
                    match token {
                        syn::NestedMeta::Meta(m) => {
                            self.parse_entity_args(m, errors);
                        },
                        syn::NestedMeta::Lit(l) => {
                            errors.push(syn::Error::new_spanned(l, "Unrecognized argument. Accepted arguments are 'name', 'version' and 'id'"));
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
                            errors.push(syn::Error::new_spanned(&nv.lit, "Store name must be a string litteral."))
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
                            errors.push(syn::Error::new_spanned(&nv.lit, "Store version must be a positive integer."))
                        }
                    }
                }
                else if nv.path.is_ident("id") {
                    match &nv.lit {
                        syn::Lit::Str(str) => {
                            self.parse_id_attr(&str.value(), &str.span(), errors);
                        },
                        _ => {
                            errors.push(syn::Error::new_spanned(&nv.lit, "Store ID must be the name of a field as a string litteral."))
                        }
                    }
                }
                else if nv.path.is_ident("crate") {
                    match &nv.lit {
                        syn::Lit::Str(str) => {
                            self.crate_name = str.value();
                        },
                        _ => {
                            errors.push(syn::Error::new_spanned(&nv.lit, "Crate name must be a string."))
                        }
                    }
                }
                else {
                    errors.push(syn::Error::new_spanned(&nv.path, "Unrecognized argument. Accepted arguments are 'name', 'version' and 'id'"))
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
                        self.id = Some(ident);
                    },
                    Err(_)=> {
                        errors.push(syn::Error::new(span.to_owned(), ID_PARSE_ERROR))
                    }
                }
            },
            Err(_) => {
                errors.push(syn::Error::new(span.to_owned(), ID_PARSE_ERROR))
            }
        }
        
    }

    fn parse_fields(&mut self, fields : &Fields, errors : &mut Errors) {
        match fields {
            Fields::Named(fields) => {
                for field in fields.named.iter() {
                    let field = field.clone();
                    self.fields.push((field.vis,field.ident.unwrap(),field.ty));
                }
            },
            _ => errors.push(syn::Error::new_spanned(fields, "Reindeer only supports deriving Entity on named structs. Please implement Entity manually.")),
        }
    }

    fn check(&mut self, span : &Span, errors : &mut Errors){
        match &self.id {
            None => {
                let id_field = self.fields.iter().find(|e| e.1.to_string() == "id");
                if let Some(id_field) = id_field {
                    self.id = Some(id_field.1.clone());
                    self.id_type = Some(id_field.2.clone());
                }
                else {
                    errors.push(syn::Error::new(span.to_owned(), r#"Missing ID specification. Use either a field called  `id`, or specify the 'id' argument of the `entity` helper attribute : `#[entity(id = "key")]`"#)); 
                }
            },
            Some(id) => {
                self.check_id(&id.clone(), errors);
            }
        }


    }
    fn check_id(&mut self, ident : &Ident, errors : &mut Errors) {
        match self.fields.iter().find(|e| e.1.to_string() == ident.to_string()) {
            Some(id) => {
                self.id_type = Some(id.2.clone());
            }
            None => {
                errors.push(syn::Error::new(ident.span(), format!("Cannot find referenced field '{}' in current type",ident)));
            }
            
        }
    }
    fn parse_related_stores(&mut self, attr : &Attribute, errors : &mut Errors){
        match attr.parse_args::<Relations>() {
            Ok(rel) => {
                if attr.path.is_ident("children"){
                    self.children = rel;
                }
                else {
                    self.siblings = rel;
                }
            },
            Err(e) => {
                errors.push(e);
            }
        }
    }

}