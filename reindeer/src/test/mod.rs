mod test_entities;

use crate::{
    error::Result, relation::FamilyDescriptor, test::test_entities::GrandChildEntity,
    AutoIncrementEntity, DeletionBehaviour, Entity, QueryBuilder,
};
use test_entities::{
    set_up, set_up_content, tear_down, ChildEntity1, ChildEntity2, Entity1, Entity2, Entity3,
};
use uuid::Uuid;

fn get_random_name() -> String {
    format!("reindeer-test-{}", Uuid::new_v4())
}
#[test]
fn create_and_register() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    assert!(FamilyDescriptor::exists(&String::from("entity_1"), &db)?);
    assert!(FamilyDescriptor::exists(&String::from("entity_2"), &db)?);
    assert!(FamilyDescriptor::exists(
        &String::from("child_entity_1"),
        &db
    )?);
    let fam_desc = FamilyDescriptor::get(&String::from("entity_1"), &db)?;
    assert!(fam_desc.is_some());
    assert_eq!(fam_desc.unwrap().sibling_trees.len(), 1);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_save_save_next_and_get() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e1_0 = Entity1::get(&0, &db)?;
    let e1_1 = Entity1::get(&1, &db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?;
    let e2_2 = Entity2::get(&String::from("id2"), &db)?;
    assert!(e1_0.is_some());
    assert!(e1_1.is_some());
    assert!(e2_1.is_some());
    assert!(e2_2.is_some());
    let e1_0 = e1_0.unwrap();
    let e1_1 = e1_1.unwrap();
    let e2_1 = e2_1.unwrap();
    let e2_2 = e2_2.unwrap();
    assert_eq!(e1_0.id, 0);
    assert_eq!(e1_0.prop1, "Hello, World!");
    assert_eq!(e1_1.id, 1);
    assert_eq!(e1_1.prop1, "Hello, Nancy!");
    assert!(Entity1::get(&8, &db)?.is_none());
    assert_eq!(e2_1.prop2, 3);
    assert_eq!(e2_2.prop2, 5);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_save_and_get_children() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let child_1 = ChildEntity1::get(&(String::from("id3"), 0), &db)?;
    assert!(child_1.is_some());
    let e2_3 = Entity2::get(&String::from("id3"), &db)?.unwrap();
    let children: Vec<ChildEntity1> = e2_3.get_children(&db)?;
    assert_eq!(children.len(), 3);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_cascade_children() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e2_3 = Entity2::get(&String::from("id3"), &db)?.unwrap();
    let mut children: Vec<ChildEntity1> = e2_3.get_children(&db)?;
    assert_eq!(children.len(), 3);
    Entity2::remove(&String::from("id3"), &db)?;
    assert!(Entity2::get(&String::from("id3"), &db)?.is_none());
    children = e2_3.get_children(&db)?;
    assert_eq!(children.len(), 0);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_delete_children_error() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e3_2 = Entity3::get(&2, &db)?.unwrap();
    let mut children: Vec<ChildEntity2> = e3_2.get_children(&db)?;
    assert_eq!(children.len(), 3);
    assert!(Entity3::remove(&2, &db).is_err());
    let e3_2 = Entity3::get(&2, &db)?;
    assert!(e3_2.is_some());
    children = e3_2.unwrap().get_children(&db)?;
    assert_eq!(children.len(), 3);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_add_sibling() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("First Sibling"),
    };
    e1.save_next(&db)?;
    let mut e3 = Entity3 {
        id: 0,
        some_bool: false,
    };
    e1.save_sibling(&mut e3, &db)?;
    assert_eq!(e3.id, e1.id);
    assert_eq!(e3.id, 3);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_delete_sibling_cascade() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("First Sibling"),
    };
    e1.save_next(&db)?;
    let mut e3 = Entity3 {
        id: 0,
        some_bool: false,
    };
    e1.save_sibling(&mut e3, &db)?;
    assert!(Entity1::remove(e1.get_key(), &db).is_ok());
    assert!(Entity1::get(e1.get_key(), &db)?.is_none());
    assert!(Entity3::get(e3.get_key(), &db)?.is_none());
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_delete_sibling_error() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("First Sibling"),
    };
    e1.save_next(&db)?;
    let mut e3 = Entity3 {
        id: 0,
        some_bool: false,
    };
    e1.save_sibling(&mut e3, &db)?;
    assert!(Entity3::remove(e1.get_key(), &db).is_err());
    assert!(Entity1::get(e1.get_key(), &db)?.is_some());
    assert!(Entity3::get(e3.get_key(), &db)?.is_some());
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_free_relation() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e1 = Entity1::get(&2, &db)?.unwrap();
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_2 = Entity2::get(&String::from("id2"), &db)?.unwrap();
    assert!(e1
        .create_relation(
            &e2_1,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    assert!(e1
        .create_relation(
            &e2_2,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    let related = e1.get_related::<Entity2>(&db)?;
    assert_eq!(related.len(), 2);
    assert_eq!(related[0].get_key(), "id1");
    assert_eq!(related[1].get_key(), "id2");
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_free_relation_cascade() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("First Sibling"),
    };
    e1.save_next(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_2 = Entity2::get(&String::from("id2"), &db)?.unwrap();
    assert!(e1
        .create_relation(
            &e2_1,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    assert!(e1
        .create_relation(
            &e2_2,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    let related = e1.get_related::<Entity2>(&db)?;
    assert_eq!(related.len(), 2);
    assert!(Entity1::remove(e1.get_key(), &db).is_ok());
    assert_eq!(e1.get_related::<Entity2>(&db)?.len(), 0);
    assert!(Entity2::get(&String::from("id1"), &db)?.is_none());
    assert!(Entity2::get(&String::from("id2"), &db)?.is_none());
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_free_relation_error() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("First Sibling"),
    };
    e1.save_next(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_2 = Entity2::get(&String::from("id2"), &db)?.unwrap();
    assert!(e1
        .create_relation(
            &e2_1,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    assert!(e1
        .create_relation(
            &e2_2,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    let related = e1.get_related::<Entity2>(&db)?;
    assert_eq!(related.len(), 2);
    assert!(Entity2::remove(e2_1.get_key(), &db).is_err());
    assert_eq!(e1.get_related::<Entity2>(&db)?.len(), 2);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_recursive_cascade() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("First Sibling"),
    };
    e1.save_next(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_3 = Entity2::get(&String::from("id3"), &db)?.unwrap();
    assert!(e1
        .create_relation(
            &e2_1,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    assert!(e1
        .create_relation(
            &e2_3,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    let related = e1.get_related::<Entity2>(&db)?;
    assert_eq!(related.len(), 2);
    assert!(Entity1::remove(e1.get_key(), &db).is_ok());
    assert_eq!(e1.get_related::<Entity2>(&db)?.len(), 0);
    assert_eq!(ChildEntity1::get_count(&db)?, 1);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_recursive_error() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e1 = Entity1::get(&2, &db)?.unwrap();
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_3 = Entity2::get(&String::from("id3"), &db)?.unwrap();
    assert!(e1
        .create_relation(
            &e2_1,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    assert!(e1
        .create_relation(
            &e2_3,
            DeletionBehaviour::Cascade,
            DeletionBehaviour::Error,
            Some("relation1"),
            &db
        )
        .is_ok());
    let related = e1.get_related::<Entity2>(&db)?;
    assert_eq!(related.len(), 2);
    assert!(Entity1::remove(e1.get_key(), &db).is_err());
    assert_eq!(e1.get_related::<Entity2>(&db)?.len(), 2);
    assert_eq!(ChildEntity1::get_count(&db)?, 4);

    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_adopt_child() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_3 = Entity2::get(&String::from("id3"), &db)?.unwrap();
    let mut children: Vec<ChildEntity1> = e2_3.get_children(&db)?;
    assert_eq!(children.len(), 3);
    assert_eq!(children[1].get_key().1, 1);
    let children_2: Vec<ChildEntity1> = e2_1.get_children(&db)?;
    assert_eq!(children_2.len(), 0);
    e2_1.adopt_as_next_child(&mut children[1], &db)?;
    let children: Vec<ChildEntity1> = e2_3.get_children(&db)?;
    assert_eq!(children.len(), 2);
    let other_children: Vec<ChildEntity1> = e2_1.get_children(&db)?;
    assert_eq!(other_children.len(), 1);
    assert_eq!(other_children[0].get_key().1, 0);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_adopt_child_with_children() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_3 = Entity2::get(&String::from("id3"), &db)?.unwrap();
    let mut children: Vec<ChildEntity1> = e2_3.get_children(&db)?;
    let mut grand_children: Vec<GrandChildEntity> = children[2].get_children(&db)?;
    assert_eq!(grand_children.len(), 3);
    e2_1.adopt_as_next_child(&mut children[2], &db)?;
    let other_children: Vec<ChildEntity1> = e2_1.get_children(&db)?;
    assert_eq!(other_children.len(), 1);
    let child = &other_children[0];
    assert_eq!(child.get_key().1, 0);
    grand_children = child.get_children(&db)?;
    assert_eq!(grand_children.len(), 3);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_adopt_child_with_relations() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e2_3 = Entity2::get(&String::from("id3"), &db)?.unwrap();
    let e3 = Entity3::get(&0, &db)?.unwrap();
    let mut children: Vec<ChildEntity1> = e2_3.get_children(&db)?;
    children[2].create_relation(
        &e3,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        None,
        &db,
    )?;
    e2_1.adopt_as_next_child(&mut children[2], &db)?;
    let other_children: Vec<ChildEntity1> = e2_1.get_children(&db)?;
    assert_eq!(other_children.len(), 1);
    let child = &other_children[0];
    assert_eq!(child.get_key().1, 0);
    assert_eq!(child.get_related::<Entity3>(&db)?.len(), 1);
    assert_eq!(e3.get_related::<ChildEntity1>(&db)?.len(), 1);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_named_relations() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e3_1 = Entity3::get(&0, &db)?.unwrap();
    let e3_3 = Entity3::get(&2, &db)?.unwrap();
    e2_1.create_relation(
        &e3_1,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        Some("rel1"),
        &db,
    )?;
    e2_1.create_relation(
        &e3_3,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        Some("rel2"),
        &db,
    )?;
    assert!(e2_1.is_related_to(&e3_1, &db)?);
    assert!(e2_1.is_related_to(&e3_3, &db)?);
    assert!(e2_1.is_related_to_with_name(&e3_1, "rel1", &db)?);
    assert!(!e2_1.is_related_to_with_name(&e3_3, "rel1", &db)?);
    assert!(e2_1.is_related_to_with_any_name(&e3_3, &["rel1", "rel2"], &db)?);
    assert_eq!(e2_1.get_related::<Entity3>(&db)?.len(), 2);
    assert_eq!(e2_1.get_related_with_name::<Entity3>("rel2", &db)?.len(), 1);
    assert_eq!(
        e2_1.get_single_related_with_name::<Entity3>("rel2", &db)?
            .unwrap()
            .id,
        2
    );
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_query_builder() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let result = QueryBuilder::new()
        .with_parent(&String::from("id3"))
        .get::<ChildEntity1>(&db)?;
    assert_eq!(result.len(), 3);
    let result_2 = QueryBuilder::new()
        .with_parent(&String::from("id3"))
        .with_relation_to::<ChildEntity2>(&(2, 1))
        .get::<ChildEntity1>(&db)?;
    assert_eq!(result_2.len(), 1);
    let child_entity_1 = ChildEntity1::get(&(String::from("id3"), 1), &db)?.unwrap();
    let child_entity_2 = ChildEntity2::get(&(2, 1), &db)?.unwrap();
    child_entity_1.create_relation(
        &child_entity_2,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        None,
        &db,
    )?;
    let result_3 = QueryBuilder::new()
        .with_parent(&String::from("id3"))
        .with_relation_to::<ChildEntity2>(&(2, 1))
        .get::<ChildEntity1>(&db)?;
    assert_eq!(result_3.len(), 2);
    tear_down(&name)?;
    Ok(())
}

#[test]
fn test_filtered_query_builder() -> Result<()> {
    let name = get_random_name();
    let db = set_up(&name)?;
    set_up_content(&db)?;
    let e2_1 = Entity2::get(&String::from("id1"), &db)?.unwrap();
    let e3_1 = Entity3::get(&0, &db)?.unwrap();
    let e3_3 = Entity3::get(&2, &db)?.unwrap();
    e2_1.create_relation(
        &e3_1,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        Some("rel1"),
        &db,
    )?;
    e2_1.create_relation(
        &e3_3,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        Some("rel1"),
        &db,
    )?;
    e2_1.create_relation(
        &e3_3,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        Some("rel2"),
        &db,
    )?;
    let result = QueryBuilder::new()
        .with_named_relation_to::<Entity2>(&e2_1.id, "rel1")
        .get_with_filter(|e: &Entity3| e.some_bool, &db)?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 2);
    tear_down(&name)?;
    Ok(())
}
