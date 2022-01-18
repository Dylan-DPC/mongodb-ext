//! This crate provides the macro [`mongo_db`] to model a mongoDB database.

//To make [`mongo_db`] work reliably a couple of re-exports are needed, these are not relevant for using the macro.
#[doc(hidden)]
pub use {async_trait, mongodb, mongodb_ext_derive, paste, serde};

#[doc(hidden)]
pub mod traits;

#[doc(hidden)]
pub use crate::mongodb_ext_derive::case;

pub use crate::traits::{MongoClient, MongoCollection};

/// Defines the default type inside an [`Option`] for the `_id` field.
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
        $(-{
            $($inner_tokens2:tt)+
        })?
    ) => {
        $crate::paste::paste! {
            #[doc = "Represents the [`" $coll_name "`] collection in mongodb."]
            #[derive($crate::serde::Deserialize, $crate::serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            $(#[$additional_coll_attr])*
            pub struct $coll_name {
                $(
                    $(#[$additional_field_attr])*
                    pub [<$field:snake:lower>]: $field_type
                ),*
            }

            impl $crate::MongoCollection for $coll_name {
                const NAME: &'static str = $crate::case!($coll_name => Camel);
            }

            $(
                impl $coll_name {
                    $($inner_tokens2)+
                }
            )?
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
        $(-{
            $($inner_tokens2:tt)+
        })?
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
            $(-{
                $($inner_tokens2)+
            })?
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
        $(-{
            $($inner_tokens2:tt)+
        })?
    ) => {
        $crate::expand_collection! {
            $(#[$additional_coll_attr])*
            $coll_name<_id: $crate::DefaultId> {
                $(
                    $(#[$additional_field_attr])*
                    $field: $field_type
                ),*
            }
            $(-{
                $($inner_tokens2)+
            })?
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
        $(-{
            $($impl:tt)+
        })?
    ) => {
        $crate::paste::paste! {
            #[doc = "Client to interact with the [`" $db_name "`] database."]
            $(#[$additional_db_attr])*
            pub struct $db_name {
                pub client: $crate::mongodb::Client,
                pub database: $crate::mongodb::Database,
                $(pub [<$coll_name:snake:lower _coll>]: $crate::mongodb::Collection<schema::$coll_name>),+
            }

            #[$crate::async_trait::async_trait]
            impl $crate::MongoClient for $db_name {
                const NAME: &'static str = $crate::case!($db_name => Camel);

                async fn new(connection_str: &str) -> $crate::mongodb::error::Result<Self> {
                    let client = match $crate::mongodb::Client::with_uri_str(connection_str).await {
                        $crate::mongodb::error::Result::Ok(client) => client,
                        $crate::mongodb::error::Result::Err(e) => return $crate::mongodb::error::Result::Err(e),
                    };
                    let database = client.database(Self::NAME);
                    // create a scope here to hygienically `use` the trait.
                    {
                        use $crate::MongoCollection;
                        $(
                            let [<$coll_name:snake:lower _coll>] = database.collection(schema::$coll_name::NAME);
                        )+
                        $crate::mongodb::error::Result::Ok(Self {
                            client,
                            database,
                            $([<$coll_name:snake:lower _coll>]),+
                        })
                    }
                }

                async fn ping(&self) -> $crate::mongodb::error::Result<$crate::mongodb::bson::document::Document> {
                    self.database.run_command($crate::mongodb::bson::doc!{"ping": 1}, std::option::Option::None).await
                }
            }
            $(
                impl $db_name {
                    $($impl)+
                }
            )?
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
/// - It implements the [`MongoClient`] trait.
/// - It contains handles to all given collections inside the database.
///     These handles have the format `{collection_name}_coll` where `{collection_name}` represents the collection's name in `snake_case`.
/// - It also contains a [`client`](mongodb::Client) and a [`database`](mongodb::Database) field for you to use.
///
/// All collections are wrapped in an additional public module named `schema`.
///
/// Each collection has its own struct which stores all specified fields.
/// All collection structs implement [`Serialize`](serde::Serialize), [`Deserialize`](serde::Deserialize) and [`MongoCollection`].
///
/// By default a field `_id` gets added to each collection automatically:
///     `pub _id: Option<DefaultId>` (see [`DefaultId`] for more info).
/// This field needs to exist for you to be able to obtain an `_id` field from the database.
/// When serializing, `_id` gets skipped if it is [`None`].
/// All fields except `_id` get renamed to `camelCase` when serializing (converting `_id` to `camelCase` results in `id`).
///
/// _Note_: All structs' names in `camelCase` can be accessed via the [`MongoClient`] / [`MongoCollection`] trait.
///
/// # Examples
///
/// ## General Examples
///
/// ```rust
/// use mongodb_ext::{mongo_db, MongoClient, MongoCollection};
/// use serde_json::ser;
///
/// mongo_db! {
///     // database name
///     SomeDatabase {
///         // additional attributes for the collection
///         #[derive(Debug, Clone)]
///         // collection name
///         SomeCollection {
///             // collection fields
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
/// // When serializing, `_id` is skipped only if it is `None`.
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
/// // constants store the collection / database names in `camelCase`
/// assert_eq!("someCollection", mongo::schema::SomeCollection::NAME);
/// assert_eq!("someDatabase", mongo::SomeDatabase::NAME);
/// ```
///
/// Multiple collections need to be separated by `;`, a trailing `;` is optional:
///
/// ```rust
/// use mongodb_ext::{mongo_db, MongoCollection, MongoClient};
///
/// mongo_db! {
///     #[derive(Debug, Clone)]
///     MyDatabase {
///         #[derive(Debug, Clone)]
///         MyFirstCollection {
///             first_name: String,
///             last_name: String,
///             age: u8,
///         };
///         #[derive(Debug)]
///         AnotherCollection {
///             some_field: String
///         };
///     }
/// }
///
/// // all constants that were defined
/// assert_eq!("myDatabase", mongo::MyDatabase::NAME);
/// assert_eq!("myFirstCollection", mongo::schema::MyFirstCollection::NAME);
/// assert_eq!("anotherCollection", mongo::schema::AnotherCollection::NAME);
///
/// // initializer function and general usage
/// // note that `tokio_test::block_on` is just a test function to run `async` code in doc tests
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
/// // _id is now u128 instead of `DefaultId`
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
///             #[serde(skip_serializing_if = "Option::is_none")]
///             email_address: Option<String>,
///             first_name: String,
///         }
///     }
/// }
///
/// // no `_id` exists, this example assumes that users are addressed via their email address
/// let some_document = mongo::schema::SomeCollection {
///     email_address: Some(String::from("bob@example.com")),
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
///         };
///         Another {
///             some_field: u32,
///         };
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
/// // `_id` field disabled
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
///         };
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
/// ## Adding your own code
///
/// Additional code for the `mongo` and `schema` modules can be specified in curly braces (`{` / `}`).
///
/// ```rust
/// use mongodb_ext::mongo_db;
///
/// mongo_db! {
///     // specify code to be in `mongo` here:
///     {
///         pub fn this_is_a_function_in_mongo() -> bool { true }
///     }
///     SomeDatabase {
///         // specify code to be in `schema` here:
///         {
///             pub fn this_is_a_function_in_schema() -> bool { true }
///             use std::collections::HashMap;
///         }
///         SomeCollection {
///             dict: HashMap<String, u32>,
///         }
///     }
/// }
///
/// assert!(mongo::this_is_a_function_in_mongo());
/// assert!(mongo::schema::this_is_a_function_in_schema());
/// ```
///
/// ### Code positioning
///
/// `Impl`ementations can be easily added by using the preset feature:
///
/// ```rust
/// use mongodb_ext::mongo_db;
///
/// mongo_db! {
///     // specify globally needed code in `mongo` here:
///     {
///         use std::collections::HashMap;
///     }
///     SomeDatabase {
///         // specify globally needed code in `schema` here:
///         {
///             use std::collections::HashMap;
///         }
///
///         // specify collection-dependent code in an additional block below the collection connected with a `-`:
///         SomeCollection {
///             dict: HashMap<String, u32>,
///         }-{
///             pub fn some_collection_function() -> bool { true }
///         };
///         AnotherCollection {}-{
///             pub fn id(&self) -> &Option<String> { &self._id }
///         }
///     }-{
///         /// specify `mongo` implementations here:
///         pub fn give_bool() -> bool { true }
///     }
/// }
///
/// assert!(mongo::SomeDatabase::give_bool());
/// assert!(mongo::schema::SomeCollection::some_collection_function());
///
/// let another_collection = mongo::schema::AnotherCollection {
///     _id: Some(String::from("id")),
/// };
/// assert_eq!(*another_collection.id(), Some(String::from("id")));
/// ```
#[macro_export]
macro_rules! mongo_db {
    // only one match, the real magic happens in `expand_collection` and `expand_main_client`
    (
        $({
            $($outer_tokens:tt)+
        })?

        $(#[$additional_db_attr:meta])*
        $db_name:ident {

            $({
                $($inner_tokens:tt)+
            })?

            $(
                $(#[$additional_coll_attr:meta])*
                $coll_name:ident$(<_id: $id_spec:ident>)? {
                    $(
                        $(#[$additional_field_attr:meta])*
                        $field:ident: $field_type:ty
                    ),*$(,)?
                }
                $(-{
                    $($inner_impl:tt)+
                })?
            );+$(;)?
        }
        $(-{
            $($outer_impl:tt)+
        })?
    ) => {
        pub mod mongo {
            $($($outer_tokens)*)?

            pub mod schema {
                $($($inner_tokens)*)?

                $(
                    $crate::expand_collection! {
                        $(#[$additional_coll_attr])*
                        $coll_name$(<_id: $id_spec>)? {
                            $(
                                $(#[$additional_field_attr])*
                                $field: $field_type
                            ),*
                        }
                        $(-{
                            $($inner_impl)+
                        })?
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
                $(-{
                    $($outer_impl)+
                })?
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::mongo_db;

    #[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
    pub struct MyLocalType;

    mongo_db! {
        #[derive(Debug, Clone)]
        DatabaseOfDoom {
            {
                use std::collections::HashMap;
                use super::super::MyLocalType;
            }

            #[derive(Debug, Clone)]
            TypeCheckCollection {
                map: HashMap<String, u32>,
                local: MyLocalType
            }-{
                pub fn collection_code() -> bool { true }
            };
            #[derive(Debug, Clone, PartialEq)]
            Items {
                counter: u16,
                name: String
            };
            #[derive(Debug)]
            QueuedItems {
                something: Option<bool>,
            };
            #[derive(Debug)]
            AnotherOne<_id: none> {
                #[serde(rename = "thisFieldsNewName")]
                renamed_field: String,
                #[serde(skip_serializing)]
                ignored_field: u64
            };
        }-{
            pub fn mongo_code() -> bool { true }
        }
    }

    #[test]
    pub fn check_additional_code() {
        assert!(mongo::DatabaseOfDoom::mongo_code());
        assert!(mongo::schema::TypeCheckCollection::collection_code());
    }

    #[test]
    pub fn check_field_attributes() {
        use serde_json::ser;

        let another_one = mongo::schema::AnotherOne {
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
        use super::traits::*;
        assert_eq!("databaseOfDoom", mongo::DatabaseOfDoom::NAME);
        assert_eq!("items", mongo::schema::Items::NAME);
        assert_eq!("queuedItems", mongo::schema::QueuedItems::NAME);
    }

    /// This test is rather useless, but it's currently the best way to test the [`DatabaseOfDoom::new`] function.
    #[test]
    pub fn check_initializer() {
        use super::traits::MongoClient;
        // try to initialize with an invalid connection string
        if let Err(e) =
            tokio_test::block_on(mongo::DatabaseOfDoom::new("invalid connection string"))
        {
            // make sure the correct error message is produced
            assert_eq!(
                format!("{}", e),
                String::from(
                    "An invalid argument was provided: connection string contains no scheme"
                )
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
