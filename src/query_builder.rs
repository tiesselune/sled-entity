
use sled::Db;

use crate::{relation::Relation, Entity, Result, AsBytes};

/// `QueryBuilder` is a convenient way to build query to target several conditions without the need to immediately
/// serialize/deserialize data from disk.
/// For simplicity's sake, ids are expressed as `&[u8]`. To get the key of an entity using this type,
/// just use `&entity.get_key().as_bytes()`.
///
/// Execute the query with `get` or `get_single`, providing the Db instance to run it on.
pub struct QueryBuilder<'a> {
    ids: Vec<Vec<u8>>,
    parent: Option<Vec<u8>>,
    related_to: Vec<(&'a str, Vec<u8>, Option<&'a str>)>,
}

impl<'a> QueryBuilder<'a> {
    /// Creates a new Query Builder.
    pub fn new() -> QueryBuilder<'a> {
        QueryBuilder {
            ids: Vec::new(),
            parent: None,
            related_to: Vec::new(),
        }
    }

    /// Specifies an array of ids to consider in this query. This can be used multiple times.
    pub fn with_ids(&mut self, ids: &mut Vec<impl AsBytes>) -> &mut QueryBuilder<'a> {
        self.ids.append(&mut ids.iter().map(|id| id.as_bytes()).collect());
        self
    }

    /// Specifies an single id to consider in this query. This can be used multiple times to specify several ids.
    pub fn with_id(&mut self, id: impl AsBytes) -> &mut QueryBuilder<'a> {
        self.ids.push(id.as_bytes());
        self
    }
    /// Specifies that this entity is the child of a given parent.
    /// This implies that the queried store is marked as a child of another entity type.
    pub fn with_parent(&mut self, id: impl AsBytes) -> &mut QueryBuilder<'a> {
        self.parent = Some(id.as_bytes());
        self
    }

    /// Specifies that a named relation to another entity has to exist.
    /// This can be used multiple times to specify several conditions.
    pub fn with_named_relation_to<OT: Entity>(
        &mut self,
        id: impl AsBytes,
        name: &'a str,
    ) -> &mut QueryBuilder<'a> {
        self.related_to.push((OT::store_name(), id.as_bytes(), Some(name)));
        self
    }

    /// Specifies that an unnamed relation to another entity has to exist.
    /// This can be used multiple times to specify several conditions.
    pub fn with_relation_to<OT: Entity>(&mut self, id: impl AsBytes) -> &mut QueryBuilder<'a> {
        self.related_to.push((OT::store_name(), id.as_bytes(), None));
        self
    }

    /// Executes the query and returns the result as a Vec of the chosen entity.
    pub fn get<T : Entity>(&self, db : &Db) -> Result<Vec<T>> {
        Ok(T::get_each_u8(&self.get_ids::<T>(db)?, db))
    }

    /// Executes the query and returns the first entity matching the query.
    pub fn get_single<T : Entity>(&self, db : &Db) -> Result<Option<T>> {
        T::get_from_u8_array(&self.get_ids::<T>(db)?[0],db)
    }

    fn get_ids<T : Entity>(&self, db: &Db) -> Result<Vec<Vec<u8>>> {
        let target_ids = match (self.ids.len(), self.related_to.len(), &self.parent) {
            (0, 0, None) => {
                return Ok(Vec::new());
            }
            (0, 0, Some(p)) => {
                let mut result: Vec<Vec<u8>> = Vec::new();
                T::get_tree(db)?.scan_prefix(p).for_each(|elem| {
                    if let Ok((key, _)) = elem {
                        result.push(key.to_vec());
                    }
                });
                return Ok(result);
            }
            (0, _, _) => {
                self.get_related_ids::<T>(db)?
            }
            (_, _, _) => {
                let related_ids = self.get_related_ids::<T>(db)?;
                let mut target_ids = Vec::new();
                for related_id in related_ids {
                    if self.ids.contains(&related_id.as_ref()) {
                        target_ids.push(related_id.clone());
                    }
                }
                target_ids
            }
        };

        if let Some(parent) = &self.parent {
            Ok(target_ids.iter().filter(|id| id.starts_with(parent)).cloned().collect())
        }
        else {
            Ok(target_ids)
        }
    }

    fn get_related_ids<T : Entity>(&self, db: &Db) -> Result<Vec<Vec<u8>>> {
        let mut target_ids = Vec::new();
        for (tree_name, id, relation_name) in &self.related_to {
            let descriptor = Relation::get_descriptor_with_key_and_tree_name(tree_name, id, db)?;
            if let Some(list) = descriptor.related_entities.get(T::store_name()) {
                for rel in list {
                    if rel.name.as_deref() == relation_name.as_deref() {
                        target_ids.push(rel.key.clone());
                    }
                }
            }
        }
        Ok(target_ids)
    }
}
