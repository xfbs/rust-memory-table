mod error;
pub mod table;
#[cfg(test)]
mod tests;

pub use crate::table::Identity;
pub use crate::table::Table;
pub use error::TableError;
