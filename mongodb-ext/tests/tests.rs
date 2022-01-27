use mongodb_ext::{mongo_db, DefaultId, MongoClient, MongoCollection};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct MyLocalType;

mongo_db! {
    #[derive(Debug, Clone)]
    Database {
        {
            use std::collections::HashMap;
            use super::super::MyLocalType;
            use mongodb_ext::MongoCollection;
        }

        #[derive(Debug, Clone)]
        Collection1<version: 2, _id: none> {
            map: HashMap<String, u32>,
            local: MyLocalType
        }-{
            pub fn collection_code() -> bool { true }
        };
        #[derive(Debug, Clone, PartialEq)]
        Collection2<version: 3> {
            counter: u16,
            name: String
        };
        #[derive(Debug)]
        Collection3 {
            something: Option<bool>,
        };
        #[derive(Debug)]
        Collection4<_id: none, version: 29> {
            #[serde(rename = "thisFieldsNewName")]
            renamed_field: String,
            #[serde(skip_serializing)]
            ignored_field: u64
        };
        #[derive(Debug, PartialEq)]
        Collection5 {
            #[builder(default = <Collection5 as MongoCollection>::SCHEMA_VERSION)]
            schema_version: i32,
        }
    }-{
        pub fn mongo_code() -> bool { true }
    }
}

#[test]
pub fn check_schema_versions() {
    use crate::MongoCollection;

    assert_eq!(mongo::schema::Collection1::SCHEMA_VERSION, 2);
    assert_eq!(mongo::schema::Collection2::SCHEMA_VERSION, 3);
    assert_eq!(mongo::schema::Collection3::SCHEMA_VERSION, 1);
    assert_eq!(mongo::schema::Collection4::SCHEMA_VERSION, 29);
}

#[test]
pub fn check_additional_code() {
    assert!(mongo::Database::mongo_code());
    assert!(mongo::schema::Collection1::collection_code());
}

#[test]
pub fn check_field_attributes() {
    use serde_json::ser;

    let another_one = mongo::schema::Collection4 {
        renamed_field: String::from("something"),
        ignored_field: 1,
    };

    assert_eq!(
        ser::to_string(&another_one).expect("Could not serialize AnotherOne"),
        String::from("{\"thisFieldsNewName\":\"something\"}")
    );
}

#[test]
pub fn check_json_serialization() {
    use serde_json::{from_value, json, Value};

    let my_item: Value = json! ({
        "counter": 0,
        "name": "my_special_item"
    });

    let my_collection_entry: mongo::schema::Collection2 =
        from_value(my_item).expect("Could not convert json Value to collection document");

    assert_eq!(
        my_collection_entry,
        mongo::schema::Collection2 {
            _id: None,
            counter: 0,
            name: String::from("my_special_item")
        }
    );
}

#[test]
pub fn check_doc_serialization() {
    use mongodb::bson::{de::from_document, doc, Document};

    let my_item: Document = doc! {
        "counter": 0,
        "name": "my_special_item"
    };

    let my_collection_entry: mongo::schema::Collection2 = from_document(my_item)
        .expect("Could not convert mongodb bson Document to collection document");

    assert_eq!(
        my_collection_entry,
        mongo::schema::Collection2 {
            _id: None,
            counter: 0,
            name: String::from("my_special_item")
        }
    );
}

#[test]
pub fn check_json_serialization_with_id() {
    use {
        mongodb::bson::oid::ObjectId,
        serde_json::{from_value, json, Value},
    };

    let my_item: Value = json! ({
        "_id": "0123456789ABCDEF01234567",
        "counter": 0,
        "name": "my_special_item"
    });

    let my_collection_entry: mongo::schema::Collection2 =
        from_value(my_item).expect("Could not convert json Value to collection document");

    assert_eq!(
        my_collection_entry,
        mongo::schema::Collection2 {
            _id: Some(ObjectId::parse_str("0123456789ABCDEF01234567").unwrap()),
            counter: 0,
            name: String::from("my_special_item")
        }
    );
}

#[test]
pub fn check_doc_serialization_with_id() {
    use mongodb::bson::{de::from_document, doc, oid::ObjectId, Document};

    let my_item: Document = doc! {
        "_id": "0123456789ABCDEF01234567",
        "counter": 0,
        "name": "my_special_item"
    };

    let my_collection_entry: mongo::schema::Collection2 = from_document(my_item)
        .expect("Could not convert mongodb bson Document to collection document");

    assert_eq!(
        my_collection_entry,
        mongo::schema::Collection2 {
            _id: Some(ObjectId::parse_str("0123456789ABCDEF01234567").unwrap()),
            counter: 0,
            name: String::from("my_special_item")
        }
    );
}

#[test]
pub fn check_constant_names() {
    assert_eq!("database", mongo::Database::NAME);
    assert_eq!("collection1", mongo::schema::Collection1::NAME);
    assert_eq!("collection2", mongo::schema::Collection2::NAME);
    assert_eq!("collection3", mongo::schema::Collection3::NAME);
    assert_eq!("collection4", mongo::schema::Collection4::NAME);
}

/// This test is rather useless, but it's currently the best way to test the [`DatabaseOfDoom::new`] function.
#[test]
pub fn check_initializer() {
    // try to initialize with an invalid connection string
    if let Err(e) = tokio_test::block_on(mongo::Database::new("invalid connection string")) {
        // make sure the correct error message is produced
        assert_eq!(
            format!("{}", e),
            String::from("An invalid argument was provided: connection string contains no scheme")
        );
    } else {
        // this should really not happen
        panic!("Somehow constructed a database client without a proper connection string")
    }

    // initialize with valid connection string
    match tokio_test::block_on(mongo::Database::new("mongodb://localhost:27017")) {
        Ok(client) => {
            // check the collections' names
            assert_eq!(client.collection1_coll.name(), "collection1");
            assert_eq!(client.collection2_coll.name(), "collection2");
            assert_eq!(client.collection3_coll.name(), "collection3");
            assert_eq!(client.collection4_coll.name(), "collection4");
        }
        Err(e) => {
            panic!(
                "Could not construct mongodb client with proper connection string: {}",
                e
            )
        }
    }
}

#[test]
pub fn test_typed_builder() {
    assert_eq!(
        mongo::schema::Collection2::builder()
            .name("Alice".to_string())
            .counter(255)
            .build(),
        mongo::schema::Collection2 {
            _id: None,
            name: "Alice".to_string(),
            counter: 255
        }
    );
    let oid = DefaultId::new();
    assert_eq!(
        mongo::schema::Collection2::builder()
            .name("Alice".to_string())
            .counter(255)
            ._id(Some(oid.clone()))
            .build(),
        mongo::schema::Collection2 {
            _id: Some(oid),
            name: "Alice".to_string(),
            counter: 255
        }
    );
}

#[test]
pub fn test_schema_version_default() {
    use mongo::schema::Collection5;

    assert_eq!(
        mongo::schema::Collection2::builder()
            .name("Alice".to_string())
            .counter(255)
            .build(),
        mongo::schema::Collection2 {
            _id: None,
            name: "Alice".to_string(),
            counter: 255
        }
    );
    assert_eq!(
        mongo::schema::Collection5::builder().build(),
        mongo::schema::Collection5 {
            _id: None,
            schema_version: <Collection5 as MongoCollection>::SCHEMA_VERSION
        }
    );
}
