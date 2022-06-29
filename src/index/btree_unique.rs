use crate::index::Index;
use crate::Identity;
use crate::IndexError;
use std::any::Any;
use std::collections::btree_map::Entry;
use std::collections::*;

#[derive(Default)]
pub struct UniqueBTreeIndex<T: Identity, K: Ord + 'static, F: Fn(&T) -> K> {
    map: F,
    data: BTreeMap<K, T::PrimaryKey>,
}

impl<T: Identity, K: Ord + 'static, F: Fn(&T) -> K> UniqueBTreeIndex<T, K, F> {
    pub fn new(map: F) -> Self {
        UniqueBTreeIndex {
            map,
            data: Default::default(),
        }
    }

    pub fn insert(&mut self, element: &T) -> Result<(), IndexError<T>> {
        let key = (self.map)(&element);
        match self.data.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(element.primary_key());
                Ok(())
            }
            Entry::Occupied(value) => Err(IndexError::Duplicate(value.get().clone())),
        }
    }

    pub fn remove(&mut self, element: &T) -> Result<(), IndexError<T>> {
        let key = (self.map)(&element);
        match self.data.entry(key) {
            Entry::Occupied(value) if value.get() == &element.primary_key() => {
                value.remove();
                Ok(())
            }
            Entry::Vacant(entry) => {
                // FIXME error?
                unimplemented!()
            }
            Entry::Occupied(value) => {
                // FIXME error?
                Ok(())
            }
        }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn lookup(&self, key: &K) -> impl Iterator<Item = T::PrimaryKey> {
        self.data.get(key).cloned().into_iter()
    }
}

impl<T: Identity, K: Ord + 'static, F: Fn(&T) -> K> Index<T> for UniqueBTreeIndex<T, K, F> {
    fn clear(&mut self) {
        self.clear()
    }

    fn insert(&mut self, value: &T) -> Result<(), IndexError<T>> {
        self.insert(value)
    }

    fn remove(&mut self, value: &T) -> Result<(), IndexError<T>> {
        self.remove(value)
    }

    fn lookup(
        &mut self,
        key: &dyn Any,
    ) -> Result<Box<dyn Iterator<Item = T::PrimaryKey>>, IndexError<T>> {
        if let Some(key) = key.downcast_ref::<K>() {
            Ok(Box::new(self.data.get(key).cloned().into_iter()))
        } else {
            Err(IndexError::KeyType)
        }
    }
}
