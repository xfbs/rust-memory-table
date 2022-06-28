use crate::error::TableError;
use std::collections::btree_map::Entry;
use std::collections::*;
use std::error::Error;
use std::fmt::Debug;

pub trait Identity {
    type PrimaryKey: Eq + Ord + Clone + Debug;
    fn primary_key(&self) -> Self::PrimaryKey;
}

#[derive(Default)]
pub struct Table<T: Identity> {
    data: BTreeMap<T::PrimaryKey, T>,
    pre_insert_hooks: BTreeMap<String, Box<dyn Fn(&mut Self, &mut T)>>,
    constraints: BTreeMap<String, Box<dyn Fn(&T) -> Result<(), Box<dyn Error>>>>,
}

impl<T: Identity> Table<T> {
    pub fn new() -> Self {
        Table {
            data: Default::default(),
            pre_insert_hooks: Default::default(),
            constraints: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn insert(&mut self, mut element: T) -> Result<T::PrimaryKey, TableError<T>> {
        // make sure constraints do not complain
        for (name, constraint) in self.constraints.iter() {
            if let Err(error) = constraint(&element) {
                return Err(TableError::Constraint(name.clone(), error));
            }
        }

        // apply pre-insert hooks
        let mut pre_insert = std::mem::take(&mut self.pre_insert_hooks);
        for (_, hook) in pre_insert.iter() {
            hook(self, &mut element);
        }
        self.pre_insert_hooks = std::mem::take(&mut pre_insert);

        // insert into indices

        // insert into data
        let primary_key = element.primary_key();

        match self.data.entry(primary_key.clone()) {
            Entry::Vacant(entry) => entry.insert(element),
            Entry::Occupied(_) => return Err(TableError::Exists(primary_key)),
        };

        Ok(primary_key)
    }

    pub fn lookup(&self, key: &T::PrimaryKey) -> Option<&T> {
        self.data.get(key)
    }

    pub fn constraint_add(
        &mut self,
        name: &str,
        constraint: impl Fn(&T) -> Result<(), Box<dyn Error>> + 'static,
    ) {
        self.constraints
            .insert(name.to_string(), Box::new(constraint));
    }

    pub fn constraint_remove(&mut self, name: &str) {
        self.constraints.remove(name);
    }

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
