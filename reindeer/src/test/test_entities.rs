use crate::error::Result;
use crate::AutoIncrementEntity;
use serde_derive::{Deserialize, Serialize};
use sled::Db;

use crate::DeletionBehaviour;
use crate::Entity;

#[derive(Serialize, Deserialize,Entity)]
#[entity(name = "entity_1",version = 1,crate = "crate")]
#[siblings(("entity_3",Cascade))]
pub struct Entity1 {
    pub id: u32,
    pub prop1: String,
}

#[derive(Serialize, Deserialize, Clone,Entity)]
#[entity(name = "entity_2",version = 1,crate = "crate")]
#[children(("child_entity_1",Cascade))]
pub struct Entity2 {
    pub id: String,
    pub prop2: u32,
}

#[derive(Serialize, Deserialize,Entity)]
#[entity(name = "entity_3",version = 1,crate = "crate")]
#[siblings(("entity_1",Error))]
#[children(("child_entity_2",Error))]
pub struct Entity3 {
    pub id: u32,
    pub some_bool : bool,
}

#[derive(Serialize, Deserialize, Clone,Entity)]
#[entity(name = "child_entity_1",version = 1,crate = "crate")]
#[children(("grand_child_entity",Cascade))]
pub struct ChildEntity1 {
    id: (String, u32),
}

#[derive(Serialize, Deserialize,Entity)]
#[entity(name = "child_entity_2",version = 1,crate = "crate")]
pub struct ChildEntity2 {
    id: (u32, u32),
}

#[derive(Serialize, Deserialize,Entity)]
#[entity(name = "grand_child_entity",version = 1,crate = "crate")]
pub struct GrandChildEntity {
    id: ((String, u32), u32),
}

pub fn set_up(name: &str) -> Result<Db> {
    let mut dir = std::env::temp_dir();
    dir.push(name);

    let db = crate::open(dir.to_str().unwrap())?;
    Entity1::register(&db)?;
    Entity2::register(&db)?;
    Entity3::register(&db)?;
    ChildEntity1::register(&db)?;
    ChildEntity2::register(&db)?;
    GrandChildEntity::register(&db)?;
    Ok(db)
}

pub fn set_up_content(db: &Db) -> Result<()> {
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("Hello, World!"),
    };
    e1.save_next(db)?;
    e1.prop1 = String::from("Hello, Nancy!");
    e1.save_next(db)?;
    e1.prop1 = String::from("Hello, Steeve!");
    e1.save_next(db)?;
    let mut e2 = Entity2 {
        id: String::from("id1"),
        prop2: 3,
    };
    e2.save(db)?;
    e2.set_key(&String::from("id2"));
    e2.prop2 = 5;
    e2.save(db)?;
    let e2_2 = e2.clone();
    e2.set_key(&String::from("id3"));
    e2.prop2 = 1000;
    e2.save(db)?;
    let mut e3 = Entity3 { id: 0, some_bool : false };
    e3.save_next(db)?;
    e3.save_next(db)?;
    let mut e4 = ChildEntity1 {
        id: (String::from("id0"), 0),
    };
    e3.some_bool = true;
    e3.save_next(db)?;

    e2.save_next_child(&mut e4, db)?;
    e2.save_next_child(&mut e4, db)?;
    e2.save_next_child(&mut e4, db)?;
    let mut e4_2 = e4.clone();
    e2_2.save_next_child(&mut e4_2, db)?;

    let mut e5 = ChildEntity2 { id: (0, 0) };
    e3.save_next_child(&mut e5, db)?;
    e3.save_next_child(&mut e5, db)?;
    e4.create_relation(
        &e5,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        None,
        db,
    )?;
    e3.save_next_child(&mut e5, db)?;
    let mut grand_child = GrandChildEntity {
        id: ((String::from("id0"), 1), 0),
    };
    e4.save_next_child(&mut grand_child, db)?;
    e4.save_next_child(&mut grand_child, db)?;
    e4.save_next_child(&mut grand_child, db)?;
    assert_eq!(grand_child.get_key().0 .0, "id3");
    assert_eq!(grand_child.get_key().0 .1, 2);
    assert_eq!(grand_child.get_key().1, 2);
    Ok(())
}

pub fn tear_down(name: &str) -> Result<()> {
    let mut dir = std::env::temp_dir();
    dir.push(name);
    std::fs::remove_dir_all(dir.to_str().unwrap())?;
    Ok(())
}
