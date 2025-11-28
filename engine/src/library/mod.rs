// Placeholder module for library database
// TODO: Implement SQLite database and queries

pub mod database;
pub mod queries;
pub mod models;

pub use database::LibraryDatabase;
pub use queries::LibraryQueries;
pub use models::*;
