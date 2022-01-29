#![cfg(feature = "mongodb-gridfs")]

use mongodb_ext::mongo_db;

mongo_db! {
    #[derive(Debug, Clone)]
    Database {
        {
            use std::collections::HashMap;
        }

        #[derive(Debug, Clone)]
        Collection1<version: 2, _id: none> {
            map: HashMap<String, u32>,
        };
    }
}

#[test]
pub fn get_bucket_from_db() {
    use mongodb_ext::{GridFSDb, MongoClient};
    use tokio_test::block_on;

    let mongo = block_on(mongo::Database::new("mongodb://example.com")).unwrap();
    let _bucket = mongo.create_bucket();
}
