use proc_macro::Ident;
use syn::Attribute;
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
        let mut attributes_data = EntityAttributeData::default();
        for attr in attrs {
            if attr.path.is_ident("entity") {
                match attr.parse_meta(){
                    Ok(meta) => {
                        
                    },
                    Err(e) => errors.push(e),
                }
            }
            else if attr.path.is_ident("children") {

            }
            else if attr.path.is_ident("siblings") {
                
            }
        }
        attributes_data
    }
}