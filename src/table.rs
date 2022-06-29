use crate::error::{IndexError, TableError};
use crate::index::Index;
use std::collections::btree_map::Entry;
use std::collections::*;
use std::error::Error;
use std::fmt::Debug;

pub trait Identity {
    type PrimaryKey: Eq + Ord + Clone + Debug + 'static;
    fn primary_key(&self) -> Self::PrimaryKey;
}

pub struct Table<T: Identity> {
    data: BTreeMap<T::PrimaryKey, T>,
    pre_insert_hooks: BTreeMap<String, Box<dyn Fn(&mut Self, &mut T)>>,
    post_insert_hooks: BTreeMap<String, Box<dyn Fn(&mut Self, &T::PrimaryKey)>>,
    constraints: BTreeMap<String, Box<dyn Fn(&T) -> Result<(), Box<dyn Error>>>>,
    indices: BTreeMap<String, Box<dyn Index<T>>>,
}

impl<T: Identity> Default for Table<T> {
    fn default() -> Self {
        Table {
            data: Default::default(),
            pre_insert_hooks: Default::default(),
            post_insert_hooks: Default::default(),
            constraints: Default::default(),
            indices: Default::default(),
        }
    }
}

impl<T: Identity> Table<T> {
    /// Create new, empty table
    pub fn new() -> Self {
        Table::default()
    }

    /// Get count of elements in table
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Clear all data in this table.
    pub fn clear(&mut self) {
        self.data.clear();
        for (_, index) in &mut self.indices {
            index.clear();
        }
    }

    /// Try inserting an element
    pub fn insert(&mut self, mut element: T) -> Result<T::PrimaryKey, TableError<T>> {
        // apply pre-insert hooks, need to do this first because they might
        // modify the element.
        self.pre_insert_hooks_apply(&mut element);

        // make sure constraints do not complain.
        self.constraints_check(&element)?;

        // insert into indices
        self.indices_insert(&element)?;

        // insert into data
        let primary_key = element.primary_key();
        match self.data.entry(primary_key.clone()) {
            Entry::Vacant(entry) => entry.insert(element),
            Entry::Occupied(_) => {
                let _ = self.indices_remove(&element);
                return Err(TableError::Exists(primary_key));
            }
        };

        // apply post-insert hooks
        self.post_insert_hooks_apply(&primary_key);

        Ok(primary_key)
    }

    /// Insert an element into all indices.
    fn indices_insert(&mut self, element: &T) -> Result<(), TableError<T>> {
        for (name, index) in self.indices.iter_mut() {
            use IndexError::*;
            match index.insert(element) {
                Ok(()) => {}
                Err(Duplicate(key)) => {
                    let name = name.clone();
                    let _ = self.indices_remove(&element);
                    return Err(TableError::Duplicate(name, key));
                }
                Err(KeyType) => unreachable!(),
            }
        }
        Ok(())
    }

    /// Remove an element from all indices.
    fn indices_remove(&mut self, element: &T) -> Result<(), TableError<T>> {
        for (name, index) in self.indices.iter_mut() {
            use IndexError::*;
            match index.remove(element) {
                Ok(()) => {}
                Err(Duplicate(key)) => unreachable!(),
                Err(KeyType) => unreachable!(),
            }
        }
        Ok(())
    }

    /// Adds an index to the table
    pub fn index_add(
        &mut self,
        name: &str,
        mut index: impl Index<T> + 'static,
    ) -> Result<(), TableError<T>> {
        index.clear();

        // insert all current data into the index.
        for (_, data) in &self.data {
            match index.insert(&data) {
                Ok(()) => {}
                // TODO
                Err(e) => unimplemented!(),
            }
        }

        self.indices.insert(name.to_string(), Box::new(index));
        Ok(())
    }

    /// Removes an index from the table, if it exists.
    pub fn index_remove(&mut self, name: &str) -> Option<Box<dyn Index<T>>> {
        self.indices.remove(name)
    }

    /// Check constraints against this element
    pub fn constraints_check(&self, element: &T) -> Result<(), TableError<T>> {
        for (name, constraint) in self.constraints.iter() {
            if let Err(error) = constraint(&element) {
                return Err(TableError::Constraint(name.clone(), error));
            }
        }
        Ok(())
    }

    /// Apply pre-insert hooks
    fn pre_insert_hooks_apply(&mut self, element: &mut T) {
        let mut pre_insert = std::mem::take(&mut self.pre_insert_hooks);
        for (_, hook) in pre_insert.iter() {
            hook(self, element);
        }
        self.pre_insert_hooks = std::mem::take(&mut pre_insert);
    }

    /// Apply post-insert hooks
    fn post_insert_hooks_apply(&mut self, key: &T::PrimaryKey) {
        let mut hooks = std::mem::take(&mut self.post_insert_hooks);
        for (_, hook) in hooks.iter() {
            hook(self, key);
        }
        self.post_insert_hooks = std::mem::take(&mut hooks);
    }

    /// Try looking up an element by it's primary key
    pub fn lookup(&self, key: &T::PrimaryKey) -> Option<&T> {
        self.data.get(key)
    }

    /// Add a constraint to this table
    pub fn constraint_add(
        &mut self,
        name: &str,
        constraint: impl Fn(&T) -> Result<(), Box<dyn Error>> + 'static,
    ) -> Result<(), TableError<T>> {
        // make sure this constraint works with existing data
        for (_, value) in &self.data {
            if let Err(error) = constraint(value) {
                return Err(TableError::Constraint(name.to_string(), error));
            }
        }

        // add constraint
        self.constraints
            .insert(name.to_string(), Box::new(constraint));

        Ok(())
    }

    /// Remove a constraint from this table
    pub fn constraint_remove(&mut self, name: &str) {
        self.constraints.remove(name);
    }

    /// Add a pre-insert hook to the table
    pub fn pre_insert_hook_add(&mut self, name: &str, hook: impl Fn(&mut Self, &mut T) + 'static) {
        self.pre_insert_hooks
            .insert(name.to_string(), Box::new(hook));
    }
}
