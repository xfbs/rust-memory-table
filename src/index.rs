use crate::Identity;
use crate::IndexError;
use std::any::Any;
use std::collections::btree_map::Entry;
use std::collections::*;

mod btree;
mod btree_unique;
mod hash;

pub use btree::BTreeIndex;
pub use btree_unique::UniqueBTreeIndex;

pub trait Index<T: Identity> {
    /// Remove all elements from the index.
    fn clear(&mut self);

    /// Insert an element into the index.
    fn insert(&mut self, value: &T) -> Result<(), IndexError<T>>;

    /// Insert multiple elements into the index.
    fn insert_bulk(
        &mut self,
        mut values: Box<dyn Iterator<Item = &T>>,
    ) -> Result<(), IndexError<T>> {
        unimplemented!()
    }

    /// Remove an element from the index.
    fn remove(&mut self, value: &T) -> Result<(), IndexError<T>>;

    /// Lookup a key in this index.
    fn lookup(
        &mut self,
        key: &dyn Any,
    ) -> Result<Box<dyn Iterator<Item = T::PrimaryKey>>, IndexError<T>>;
}
