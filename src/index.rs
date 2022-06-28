use crate::Identity;
use crate::IndexError;
use std::collections::btree_map::Entry;
use std::collections::*;

pub trait Index<T: Identity> {
    /// Remove all elements from the index.
    fn clear(&mut self);

    /// Insert an element into the index.
    fn insert(&mut self, value: &T) -> Result<(), IndexError<T>>;

    /// Remove an element from the index.
    fn remove(&mut self, value: &T) -> Result<(), IndexError<T>>;
}

#[derive(Default)]
pub struct UniqueBTreeIndex<T: Identity, K: Ord, F: Fn(&T) -> K> {
    map: F,
    data: BTreeMap<K, T::PrimaryKey>,
}

impl<T: Identity, K: Ord, F: Fn(&T) -> K> UniqueBTreeIndex<T, K, F> {
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
                unimplemented!()
            }
        }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }
}

impl<T: Identity, K: Ord, F: Fn(&T) -> K> Index<T> for UniqueBTreeIndex<T, K, F> {
    fn clear(&mut self) {
        self.clear()
    }

    fn insert(&mut self, value: &T) -> Result<(), IndexError<T>> {
        self.insert(value)
    }

    fn remove(&mut self, value: &T) -> Result<(), IndexError<T>> {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct BTreeIndex {}

#[derive(Default)]
pub struct HashIndex {}

#[derive(Default)]
pub struct UniqueHashIndex {}
