use crate::Identity;
use crate::TableError;
use std::collections::*;

pub trait Index<T: Identity> {
    fn insert(&mut self, value: &T) -> Result<(), TableError<T>>;
    fn remove(&mut self, value: &T) -> Result<(), TableError<T>>;
}

#[derive(Default)]
pub struct UniqueBTreeIndex {}

#[derive(Default)]
pub struct BTreeIndex {}

#[derive(Default)]
pub struct HashIndex {}

#[derive(Default)]
pub struct UniqueHashIndex {}
