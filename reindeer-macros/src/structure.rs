#[derive(PartialEq, Eq)]
enum IdStructure {
    Simple(String),
    Tuple(Box<(IdStructure,IdStructure)>)
}

#[derive(PartialEq, Eq)]
pub struct EntityStructureData {
    name : Option<String>,
    version : Option<u32>,
    id : IdStructure,
    children : Vec<(String,String)>,
    siblings : Vec<(String,String)>,
    fields : Vec<(String,String)>,
}