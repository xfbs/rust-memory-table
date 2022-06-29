use crate::index::Index;
use crate::Identity;
use crate::IndexError;
use std::any::Any;
use std::collections::btree_map::Entry;
use std::collections::*;

#[derive(Default)]
pub struct BTreeIndex<T: Identity, K: Ord + 'static, F: Fn(&T) -> K> {
    map: F,
    data: BTreeMap<K, BTreeSet<T::PrimaryKey>>,
}

impl<T: Identity, K: Ord + 'static, F: Fn(&T) -> K> BTreeIndex<T, K, F> {
    pub fn new(map: F) -> Self {
        BTreeIndex {
            map,
            data: Default::default(),
        }
    }

    pub fn insert(&mut self, element: &T) -> Result<(), IndexError<T>> {
        let key = (self.map)(&element);
        match self.data.entry(key) {
            Entry::Vacant(entry) => {
                let mut set = BTreeSet::new();
                set.insert(element.primary_key());
                entry.insert(set);
                Ok(())
            }
            Entry::Occupied(mut value) => {
                let mut set = value.get_mut();
                set.insert(element.primary_key());
                Ok(())
            }
        }
    }

    pub fn remove(&mut self, element: &T) -> Result<(), IndexError<T>> {
        let key = (self.map)(&element);
        match self.data.entry(key) {
            Entry::Occupied(mut value) => {
                let mut set = value.get_mut();
                set.remove(&element.primary_key());

                // remove the entry altogether if the set is empty
                if set.len() == 0 {
                    drop(set);
                    value.remove();
                }

                Ok(())
            }
            Entry::Vacant(entry) => {
                // FIXME error?
                //unimplemented!()
                Ok(())
            }
        }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn lookup(&self, key: &K) -> impl Iterator<Item = T::PrimaryKey> {
        self.data
            .get(key)
            .cloned()
            .map(|value| value.into_iter())
            .into_iter()
            .flatten()
    }
}

impl<T: Identity, K: Ord + 'static, F: Fn(&T) -> K> Index<T> for BTreeIndex<T, K, F> {
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
        if let Some(key) = &key.downcast_ref::<K>() {
            Ok(Box::new(
                self.data
                    .get(key)
                    .cloned()
                    .map(|value| value.into_iter())
                    .into_iter()
                    .flatten(),
            ))
        } else {
            Err(IndexError::KeyType)
        }
    }
}
