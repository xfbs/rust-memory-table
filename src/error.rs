use crate::table::Identity;
use std::error::Error;

/// Errors that can occur when dealing with tables.
#[derive(thiserror::Error, Debug)]
pub enum TableError<T: Identity> {
    #[error("Constraint {0:} failed: {1:}")]
    Constraint(String, Box<dyn Error>),
    #[error("Value with primary key {0:?} already exists")]
    Exists(T::PrimaryKey),
    #[error("Duplicate entry in index {0:}, already has {1:?}")]
    Duplicate(String, T::PrimaryKey),
}

/// Errors that can occur when dealing with indices.
#[derive(thiserror::Error, Debug)]
pub enum IndexError<T: Identity> {
    #[error("Duplicate entry, already has {0:?}")]
    Duplicate(T::PrimaryKey),
}
