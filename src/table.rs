use crate::error::TableError;
use std::collections::btree_map::Entry;
use std::collections::*;
use std::error::Error;
use std::fmt::Debug;

pub trait Identity {
    type PrimaryKey: Eq + Ord + Clone + Debug;
    fn primary_key(&self) -> Self::PrimaryKey;
}

pub struct Table<T: Identity> {
    data: BTreeMap<T::PrimaryKey, T>,
    pre_insert_hooks: BTreeMap<String, Box<dyn Fn(&mut Self, &mut T)>>,
    post_insert_hooks: BTreeMap<String, Box<dyn Fn(&mut Self, &T::PrimaryKey)>>,
    constraints: BTreeMap<String, Box<dyn Fn(&T) -> Result<(), Box<dyn Error>>>>,
}

impl<T: Identity> Default for Table<T> {
    fn default() -> Self {
        Table {
            data: Default::default(),
            pre_insert_hooks: Default::default(),
            post_insert_hooks: Default::default(),
            constraints: Default::default(),
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

    /// Try inserting an element
    pub fn insert(&mut self, mut element: T) -> Result<T::PrimaryKey, TableError<T>> {
        // make sure constraints do not complain
        self.constraints_check(&element)?;

        // apply pre-insert hooks
        self.pre_insert_hooks_apply(&mut element);

        // insert into indices

        // insert into data
        let primary_key = element.primary_key();

        match self.data.entry(primary_key.clone()) {
            Entry::Vacant(entry) => entry.insert(element),
            Entry::Occupied(_) => return Err(TableError::Exists(primary_key)),
        };

        self.post_insert_hooks_apply(&primary_key);

        Ok(primary_key)
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
    ) {
        self.constraints
            .insert(name.to_string(), Box::new(constraint));
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

pub trait Index<T> {
    fn insert(&mut self, value: T) -> bool;
}

#[derive(Default)]
pub struct CustomIndex {}
