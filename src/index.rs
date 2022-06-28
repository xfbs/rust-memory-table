use crate::Identity;
use crate::IndexError;
use std::collections::*;

pub trait Index<T: Identity> {
    fn insert(&mut self, value: &T) -> Result<(), IndexError<T>>;
    fn remove(&mut self, value: &T) -> Result<(), IndexError<T>>;
}

#[derive(Default)]
pub struct UniqueBTreeIndex {}

#[derive(Default)]
pub struct BTreeIndex {}

#[derive(Default)]
pub struct HashIndex {}

#[derive(Default)]
pub struct UniqueHashIndex {}
