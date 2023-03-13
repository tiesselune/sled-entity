use std::{fs::{read_to_string,write, self}, path::Path};

use proc_macro2::Span;
use serde::{Serialize, Deserialize};
use toml::from_str;

use crate::{entity_data::{EntityData, SerEntityData}, Errors};
use std::path::PathBuf;

#[derive(Serialize,Deserialize)]
struct SchemaConfig {
    pub path : String,
}

pub fn save_schema(entity_data : &EntityData,errors : &mut Errors){
    let mut path = PathBuf::new();
    path.push(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("reindeer.toml");
    match read_to_string(&path) {
        Ok(data) => {
            match from_str::<SchemaConfig>(&data) {
                Ok(config) => {
                    save_schema_with_config(entity_data, &config,errors);
                },
                Err(e) => {
                    errors.push(syn::Error::new(Span::call_site(), format!("Cannot read TOML file {} {}",path.to_str().unwrap(),e)));
                },
            }
        },
        Err(e) => {
            errors.push(syn::Error::new(Span::call_site(), format!("Cannot read path {} {}",path.to_str().unwrap(),e)));
        }
    }
}

fn save_schema_with_config(entity_data : &EntityData, config : &SchemaConfig, errors : &mut Errors) {
    let serializable_data : SerEntityData = entity_data.clone().into();
    let name = entity_data.name.clone();
    match toml::to_string_pretty(&serializable_data) {
        Ok(str_data) => {
            let mut schema_path = PathBuf::new();
            schema_path.push(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            schema_path.push(&config.path);
            if !Path::exists(&schema_path) {
                fs::create_dir_all(&schema_path);
            }
            schema_path.push(format!("{}_v{}.toml",&name.unwrap(),entity_data.version.unwrap_or(0)));
            match write(&schema_path, str_data) {
                Ok(_) => {},
                Err(e) => {
                    errors.push(syn::Error::new(Span::call_site(), format!("Cannot write file {} {}",schema_path.to_str().unwrap(),e)));
                },
            }
        },
        Err(e) => errors.push(syn::Error::new(Span::call_site(), format!("Cannot serialize file {}",e)))
    }
}