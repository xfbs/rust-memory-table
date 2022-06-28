mod error;
mod index;
pub mod table;
#[cfg(test)]
mod tests;

pub use crate::index::{BTreeIndex, Index, UniqueBTreeIndex};
pub use crate::table::Identity;
pub use crate::table::Table;
pub use error::{IndexError, TableError};
