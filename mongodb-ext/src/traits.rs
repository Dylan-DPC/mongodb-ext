//! This module conains all traits of this crate.

use crate::{
    async_trait::async_trait,
    mongodb::{
        bson::document::Document, error::Result as MongoResult, Client as DbClient, Database,
    },
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
    ///
    /// Creates a database [`DbClient`] and calls [`new_with_client`](MongoClient::new_with_client) then.
    async fn new(connection_str: &str) -> MongoResult<Self>;
    /// Initializer function that uses the given client.
    ///
    /// Useful when interacting with multiple databases.
    fn new_with_client(client: DbClient) -> MongoResult<Self>;
    /// Method that sends a ping command to the database.
    async fn ping(&self) -> MongoResult<Document>;

    /// Returns a reference to the database object.
    fn database(&self) -> &Database;
    /// Returns a reference to the mongodb client object.
    fn client(&self) -> &DbClient;
}

#[cfg(feature = "mongodb-gridfs")]
pub use gridfs::GridFSDb;

/// Optional module that is enabled using the _"mongodb-gridfs"_ feature.
///
/// Provides automatic implementation of the [`GridFSDb`](gridfs::GridFSDb) trait on all types that implement [`MongoClient`].
#[cfg(feature = "mongodb-gridfs")]
pub mod gridfs {
    use {super::MongoClient, mongodb_gridfs::GridFSBucket};

    /// Trait that is implemented automatically on all Database handlers.
    ///
    /// Feature flag _"mongodb-gridfs"_ is needed to use this trait.
    ///
    /// ```rust
    /// use mongodb_ext::{mongo_db, GridFSDb, MongoClient};
    /// use mongodb_gridfs::GridFSBucket;
    /// use tokio_test::block_on;
    ///
    /// mongo_db! {
    ///     SomeDatabase {
    ///         SomeCollection {
    ///             some_field: String
    ///         }
    ///     }
    /// }
    ///
    /// use mongo::SomeDatabase;
    ///
    /// // using `tokio::block_on` to run async code in tests
    /// let db: SomeDatabase = block_on(SomeDatabase::new("mongodb://example.com")).unwrap();
    /// let bucket: GridFSBucket = db.create_bucket();
    /// ```
    pub trait GridFSDb: MongoClient {
        /// Creates a mongodb GridFS bucket.
        fn create_bucket(&self) -> GridFSBucket {
            GridFSBucket::new(self.database().clone(), None)
        }
    }

    impl<T> GridFSDb for T where T: MongoClient {}
}
