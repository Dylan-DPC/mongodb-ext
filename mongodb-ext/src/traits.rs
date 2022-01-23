//! This module conains all traits of this crate.

use crate::{
    async_trait::async_trait,
    mongodb::{bson::document::Document, error::Result as MongoResult},
};

/// Trait that is implemented automatically on each collection struct by [`mongo_db`].
pub trait MongoCollection {
    /// The collection's name.
    const NAME: &'static str;
    /// The collection's schema version.
    ///
    /// Change that in your [`mongo_db!`](crate::mongo_db) invocation every time you change your schema.
    ///
    /// You do not actually need to use this in your schema, but it is implemented for your convinience.
    const SCHEMA_VERSION: i32;
}

/// Async trait that is implemented automatically on the database handler struct by [`mongo_db`].
#[async_trait]
pub trait MongoClient
where
    Self: Sized,
{
    /// The database's name.
    const NAME: &'static str;
    /// Initializer funtion of the database.
    async fn new(connection_str: &str) -> MongoResult<Self>;
    /// Method that sends a ping command to the database.
    async fn ping(&self) -> MongoResult<Document>;
}
