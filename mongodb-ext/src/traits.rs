//! This module conains all traits of this crate.

use crate::{
    async_trait::async_trait,
    mongodb::{bson::document::Document, error::Result as MongoResult},
};

/// Trait that is implemented automatically on each collection struct by [`mongo_db`].
pub trait MongoCollection {
    /// The collection's name.
    const NAME: &'static str;
}

/// Trait that is implemented automatically on the database handler struct by [`mongo_db`].
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
