//! This crate provides the macro [`mongo_db`] to model a mongoDB database.

//To make [`mongo_db`] work reliably a couple of re-exports are needed, these are not relevant for using the macro.
#[doc(hidden)]
pub use {mongodb, mongodb_ext_derive, paste, serde};

/// Defines the default type for the `_id` field.
pub type DefaultId = String;

/// Expands one collection.
///
/// Needed internally, but has no big use on its own.
/// Thus hidden from documentation.
#[macro_export]
#[doc(hidden)]
macro_rules! expand_collection {
    // invoked with `_id: none`, thus assume `_id` is added already and finally expand to collection
    (
        $(#[$additional_coll_attr:meta])*
        $coll_name:ident<_id: none> {
            $(
                $(#[$additional_field_attr:meta])*
                $field:ident: $field_type:ty
            ),*$(,)?
        }
    ) => {
        $crate::paste::paste! {
            #[doc = "Represents the [`" $coll_name "`] collection in mongodb."]
            #[derive($crate::serde::Deserialize, $crate::serde::Serialize, $crate::mongodb_ext_derive::ConstName)]
            #[serde(rename_all = "camelCase")]
            #[const_name_value(Camel)]
            $(#[$additional_coll_attr])*
            pub struct $coll_name {
                $(
                    $(#[$additional_field_attr])*
                    pub [<$field:snake:lower>]: $field_type
                ),*
            }
        }
    };
    // specific type for `_id` given, add it and invoke again with `_id: none` to avoid adding the `_id` field again
    (
        $(#[$additional_coll_attr:meta])*
        $coll_name:ident<_id: $explicit_id_type:ty> {
            $(
                $(#[$additional_field_attr:meta])*
                $field:ident: $field_type:ty
            ),*$(,)?
        }
    ) => {
        $crate::expand_collection! {
            $(#[$additional_coll_attr])*
            $coll_name<_id: none> {
                #[serde(skip_serializing_if = "std::option::Option::is_none")]
                #[serde(rename = "_id")]
                _id: std::option::Option<$explicit_id_type>,
                $(
                    $(#[$additional_field_attr])*
                    $field: $field_type
                ),*
            }
        }
    };
    // no specific type for `_id` given, add `DefaultId` and invoke again
    (
        $(#[$additional_coll_attr:meta])*
        $coll_name:ident {
            $(
                $(#[$additional_field_attr:meta])*
                $field:ident: $field_type:ty
            ),*$(,)?
        }
    ) => {
        $crate::expand_collection! {
            $(#[$additional_coll_attr])*
            $coll_name<_id: $crate::DefaultId> {
                $(
                    $(#[$additional_field_attr])*
                    $field: $field_type
                ),*
            }
        }
    };
}

/// Expands the main database client.
///
/// Needed internally, but has no big use on its own.
/// Thus hidden from documentation.
#[macro_export]
#[doc(hidden)]
macro_rules! expand_main_client {
    (
        $(#[$additional_db_attr:meta])*
        $db_name:ident {
            $(
                $(#[$additional_coll_attr:meta])*
                $coll_name:ident<_id: none> {
                    $(
                        $(#[$additional_field_attr:meta])*
                        $field:ident: $field_type:ty
                    ),*$(,)?
                }
            ),+
        }
    ) => {
        $crate::paste::paste! {
            #[doc = "Client to interact with the [`" $db_name "`] database."]
            #[derive($crate::mongodb_ext_derive::ConstName)]
            #[const_name_value(Camel)]
            #[const_name_key("DB_NAME")]
            $(#[$additional_db_attr])*
            pub struct $db_name {
                pub client: $crate::mongodb::Client,
                pub database: $crate::mongodb::Database,
                $(pub [<$coll_name:snake:lower _coll>]: $crate::mongodb::Collection<schema::$coll_name>),+
            }

            impl $db_name {
                #[doc = "Initializer funtion of the database."]
                pub async fn new(connection_str: &str) -> std::result::Result<Self, std::string::String> {
                    let client = match $crate::mongodb::Client::with_uri_str(connection_str).await {
                        $crate::mongodb::error::Result::Ok(client) => client,
                        $crate::mongodb::error::Result::Err(e) => return Err(format!("Could not initialize mongodb client: {}", e)),
                    };
                    let database = client.database(DB_NAME);
                    $(let [<$coll_name:snake:lower _coll>] = database.collection(schema::[<$coll_name:snake:upper>]);)+
                    std::result::Result::Ok(Self {
                        client,
                        database,
                        $([<$coll_name:snake:lower _coll>]),+
                    })
                }

                #[doc = "Method that sends a ping command to the database."]
                #[allow(dead_code)]
                pub async fn ping(&self) -> $crate::mongodb::error::Result<$crate::mongodb::bson::document::Document> {
                    self.database.run_command($crate::mongodb::bson::doc!{"ping": 1}, std::option::Option::None).await
                }
            }
        }
    };
}

/// Model a mongodb database.
///
/// This macro creates structs / functions / constants / modules that represent a mongoDB database.
/// Being a macro (which is expanded at compile time) there is no run time performance penalty when using this macro.
///
/// # Structure
///
/// This macro wraps everything in a module called `mongo`.
///
/// The main database handler has the following attributes:
/// - Its name represents the database's name (eg. a database named `MyDatabase` has a struct `mongo::MyDatabase`).
/// - It has an initializer function:
///     `pub async fn new(connection_str: &str) -> Result<Self, String>`
/// - It has a ping function that sends a ping message to the database:
///     `pub async fn ping(&self) -> mongodb::error::Result<mongodb::bson::document::Document>`
/// - It contains handles to all given collections inside the database.
///     These handles have the format `{collection_name}_coll` where `{collection_name}` represents the collection's name in snake_case.
/// - It also contains a [`client`](mongodb::Client) and a [`database`](mongodb::Database) field for you to use.
///
/// All collections are wrapped in an additional public module named `schema`.
///
/// Each collection also has its own struct which stores all specified fields.
/// All collections' structs implement [`Serialize`](serde::Serialize) and [`Deserialize`](serde::Deserialize).
///
/// By default a field `_id` gets added to each collection automatically:
///     `pub _id: Option<DefaultId>` ([`DefaultId`]).
/// This field needs to exist for you to be able to obtain an `_id` field from the database.
/// When serializing, `_id` gets skipped if it is [`None`].
/// All fields except `_id` get renamed to `camelCase` when serializing (converting `_id` to `camelCase` results in `id`).
///
/// Additionally the following constants are specified:
/// - `mongo::DB_NAME` is set to the database's name in `camelCase`.
/// - `mongo::schema::{COLLECTION_NAME}` where `{COLLECTION_NAME}` represents each collection's name in screaming snake case. Set to the collection's name in `camelCase`.
///
/// # Hygiene
///
/// All structs / constants / functions are wrapped in a public module called `mongo`.
/// All structs / constants that refer to a collection are wrapped in an additional public module called `schema`.
/// This is done to maintain more hygiene by exposing less items.
/// A better hygiene creates less interference of the macro and its surrounding items.
///
/// In addition to this measure, all paths referred by the code in the macro are full paths, thus there should be no type interference.
///
/// # Examples
///
/// ## Manipulating / Removing `_id`
///
/// You can specify any type (that implements [`Serialize`](serde::Serialize) and [`Deserialize`](serde::Deserialize)) to be used inside the `_id` [`Option`] by specifying it in `<` / `>` after the collection name:
///
/// ```rust
/// use mongodb_ext::mongo_db;
///
/// mongo_db! {
///     SomeDatabase {
///         SomeCollection<_id: u128> {
///             first_name: String,
///         }
///     }
/// }
///
/// // _id is now u128
/// let some_document = mongo::schema::SomeCollection {
///     _id: Some(255),
///     first_name: String::from("Bob")
/// };
/// ```
///
/// It is also possible to disable the generation of an `_id` field all together by using `<_id: none>`.
///
/// ```rust
/// use mongodb_ext::mongo_db;
///
/// mongo_db! {
///     SomeDatabase {
///         SomeCollection<_id: none> {
///             email_address: String,
///             first_name: String,
///         }
///     }
/// }
///
/// // no _id exists, this example assumes that users are addressed via their email address
/// let some_document = mongo::schema::SomeCollection {
///     email_address: String::from("bob@example.com"),
///     first_name: String::from("Bob")
/// };
/// ```
///
/// These features are unique for each collection:
///
/// ```rust
/// use mongodb_ext::mongo_db;
///
/// mongo_db! {
///     SomeDatabase {
///         SomeCollection<_id: u128> {
///             first_name: String,
///         },
///         Another {
///             some_field: u32,
///         },
///         AndYetAnother<_id: none> {
///             email: String,
///             name: String,
///         }
///     }
/// }
///
/// // `_id` type changed to `u128`
/// let some_document = mongo::schema::SomeCollection {
///     _id: Some(255),
///     first_name: String::from("Bob")
/// };
/// // `_id` type default, eg. `DefaultId`
/// let another_document = mongo::schema::Another {
///     _id: Some(String::from("my_id")),
///     some_field: 1,
/// };
/// // `_id` field omitted
/// let and_yet_another_document = mongo::schema::AndYetAnother {
///     name: String::from("Bob"),
///     email: String::from("bob@example.com")
/// };
/// ```
///
/// ## Serializing from [`json!`](serde_json::json) and [`doc!`](mongodb::bson::doc)
///
/// ```rust
/// use mongodb_ext::mongo_db;
/// use serde_json::{json, Value};
/// use mongodb::{bson::{doc, Document}, bson};
///
/// mongo_db! {
///     #[derive(Debug, Clone)]
///     DatabaseOfItems {
///         #[derive(Debug, Clone, PartialEq)]
///         Items {
///             counter: u16,
///             name: String
///         },
///     }
/// }
///
/// // Note that `_id` is not specified here
/// let my_item: Value = json! ({
///     "counter": 0,
///     "name": "my_special_item"
/// });
///
/// let my_collection_entry: mongo::schema::Items =
///     serde_json::from_value(my_item)
///     .expect("Could not convert json Value to collection document");
///
/// assert_eq!(
///     my_collection_entry,
///     mongo::schema::Items {
///         _id: None,
///         counter: 0,
///         name: String::from("my_special_item")
///     }
/// );
///
/// // Note that `_id` is not specified here
/// let my_item: Document = doc! {
///     "counter": 0,
///     "name": "my_special_item"
/// };
///
/// let my_collection_entry: mongo::schema::Items = bson::de::from_document(my_item)
///     .expect("Could not convert mongodb bson Document to collection document");
///
/// assert_eq!(
///     my_collection_entry,
///     mongo::schema::Items {
///         _id: None,
///         counter: 0,
///         name: String::from("my_special_item")
///     }
/// );
/// ```
///
/// ## General Examples
///
/// ```rust
/// use mongodb_ext::mongo_db;
/// use serde_json::ser;
///
/// mongo_db! {
///     SomeDatabase {
///         #[derive(Debug, Clone)]
///         SomeCollection {
///             first_name: String,
///         }
///     }
/// }
///
/// let mut some_document = mongo::schema::SomeCollection {
///     _id: None,
///     first_name: String::from("alice")
/// };
///
/// // When serializing, `_id` is skipped only if `None`.
/// // Note the key conversion to `camelCase`.
/// assert_eq!(
///     ser::to_string(&some_document).unwrap(),
///     String::from("{\"firstName\":\"alice\"}")
/// );
///
/// // update `_id` field to include in serialization.
/// some_document._id = Some(String::from("my-custom-ID"));
/// assert_eq!(
///     ser::to_string(&some_document).unwrap(),
///     String::from("{\"_id\":\"my-custom-ID\",\"firstName\":\"alice\"}")
/// );
///
/// assert_eq!("someCollection", mongo::schema::SOME_COLLECTION);
/// assert_eq!("someDatabase", mongo::DB_NAME);
/// ```
///
/// ```rust
/// use mongodb_ext::mongo_db;
///
/// mongo_db! {
///     #[derive(Debug, Clone)]
///     MyDatabase {
///         #[derive(Debug, Clone)]
///         MyFirstCollection {
///             first_name: String,
///             last_name: String,
///             age: u8,
///         },
///         #[derive(Debug)]
///         AnotherCollection {
///             some_field: String
///         }
///     }
/// }
///
/// // all constants that were defined
/// assert_eq!("myDatabase", mongo::DB_NAME);
/// assert_eq!("myFirstCollection", mongo::schema::MY_FIRST_COLLECTION);
/// assert_eq!("anotherCollection", mongo::schema::ANOTHER_COLLECTION);
///
/// // initializer function and general usage
/// // note that `tokio_test::block_on` is just a test function to run `async` code
///
/// let mongo = tokio_test::block_on(mongo::MyDatabase::new("mongodb://example.com"))
///     .expect("Could not create mongoDB client");
///
/// let bob = mongo::schema::MyFirstCollection {
///     _id: None,
///     first_name: String::from("Bob"),
///     last_name: String::from("Bob's last name"),
///     age: 255,
/// };
///
/// // This should fail beause there is no actual mongoDB service running at the specified connection.
/// assert!(tokio_test::block_on(
///     mongo.my_first_collection_coll.insert_one(bob, None)
/// ).is_err());
/// ```
#[macro_export]
macro_rules! mongo_db {
    // only one match, the real magic happens in `expand_collection` and `expand_main_client`
    (
        $(#[$additional_db_attr:meta])*
        $db_name:ident {
            $(
                $(#[$additional_coll_attr:meta])*
                $coll_name:ident$(<_id: $id_spec:ident>)? {
                    $(
                        $(#[$additional_field_attr:meta])*
                        $field:ident: $field_type:ty
                    ),*$(,)?
                }
            ),+$(,)?
        }
    ) => {
        pub mod mongo {
            pub mod schema {
                $(
                    $crate::expand_collection! {
                        $(#[$additional_coll_attr])*
                        $coll_name$(<_id: $id_spec>)? {
                            $(
                                $(#[$additional_field_attr])*
                                $field: $field_type
                            ),*
                        }
                    }
                )+
            }

            $crate::expand_main_client ! {
                $(#[$additional_db_attr])*
                $db_name {
                    $(
                        $(#[$additional_coll_attr])*
                        $coll_name<_id: none> {
                            $(
                                $(#[$additional_field_attr])*
                                $field: $field_type
                            ),*
                        }
                    ),+
                }
            }
        }

    };
}

#[cfg(test)]
mod test {
    use super::mongo_db;

    mongo_db! {
        #[derive(Debug, Clone)]
        DatabaseOfDoom {
            #[derive(Debug, Clone, PartialEq)]
            Items {
                counter: u16,
                name: String
            },
            #[derive(Debug)]
            QueuedItems {
                something: Option<bool>,
            }
        }
    }

    #[test]
    pub fn check_json_serialization() {
        use serde_json::{from_value, json, Value};

        let my_item: Value = json! ({
            "counter": 0,
            "name": "my_special_item"
        });

        let my_collection_entry: mongo::schema::Items =
            from_value(my_item).expect("Could not convert json Value to collection document");

        assert_eq!(
            my_collection_entry,
            mongo::schema::Items {
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

        let my_collection_entry: mongo::schema::Items = from_document(my_item)
            .expect("Could not convert mongodb bson Document to collection document");

        assert_eq!(
            my_collection_entry,
            mongo::schema::Items {
                _id: None,
                counter: 0,
                name: String::from("my_special_item")
            }
        );
    }

    #[test]
    pub fn check_json_serialization_with_id() {
        use serde_json::{from_value, json, Value};

        let my_item: Value = json! ({
            "_id": "my_fancy_id",
            "counter": 0,
            "name": "my_special_item"
        });

        let my_collection_entry: mongo::schema::Items =
            from_value(my_item).expect("Could not convert json Value to collection document");

        assert_eq!(
            my_collection_entry,
            mongo::schema::Items {
                _id: Some(String::from("my_fancy_id")),
                counter: 0,
                name: String::from("my_special_item")
            }
        );
    }

    #[test]
    pub fn check_doc_serialization_with_id() {
        use mongodb::bson::{de::from_document, doc, Document};

        let my_item: Document = doc! {
            "_id": "my_fancy_id",
            "counter": 0,
            "name": "my_special_item"
        };

        let my_collection_entry: mongo::schema::Items = from_document(my_item)
            .expect("Could not convert mongodb bson Document to collection document");

        assert_eq!(
            my_collection_entry,
            mongo::schema::Items {
                _id: Some(String::from("my_fancy_id")),
                counter: 0,
                name: String::from("my_special_item")
            }
        );
    }

    #[test]
    pub fn check_constants() {
        assert_eq!("databaseOfDoom", mongo::DB_NAME);
        assert_eq!("items", mongo::schema::ITEMS);
        assert_eq!("queuedItems", mongo::schema::QUEUED_ITEMS);
    }

    /// This test is rather useless, but it's currently the best way to test the [`DatabaseOfDoom::new`] function.
    #[test]
    pub fn check_initializer() {
        // try to initialize with an invalid connection string
        if let Err(e) =
            tokio_test::block_on(mongo::DatabaseOfDoom::new("invalid connection string"))
        {
            // make sure the correct error message is produced
            assert_eq!(
                &e,
                "Could not initialize mongodb client: An invalid argument was provided: connection string contains no scheme"
            );
        } else {
            // this should really not happen
            panic!("Somehow constructed a database client without a proper connection string")
        }

        // initialize with valid connection string
        match tokio_test::block_on(mongo::DatabaseOfDoom::new("mongodb://localhost:27017")) {
            Ok(client) => {
                // check the collections' names
                assert_eq!(client.items_coll.name(), "items");
                assert_eq!(client.queued_items_coll.name(), "queuedItems");
            }
            Err(e) => {
                panic!(
                    "Could not construct mongodb client with proper connection string: {}",
                    e
                )
            }
        }
    }
}
