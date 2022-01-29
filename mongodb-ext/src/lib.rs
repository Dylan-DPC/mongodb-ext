//! This crate provides the macro [`mongo_db`] to model a mongoDB database.
//!
//! # Features
//!
//! Feature flags are documented here.
//!
//! ## `default`
//!
//! This feature enables the following feature(s):
//!
//! - `mongodb-gridfs`
//!
//! ## `mongodb-gridfs`
//!
//! Enabling this feature creates automatic implementations of the then-available trait `GridFSDb`.

/// To make [`mongo_db`] work reliably a couple of re-exports are needed, these are not relevant for using the macro.
#[doc(hidden)]
pub use {async_trait, mongodb, mongodb_ext_derive, paste, serde, typed_builder};

#[doc(hidden)]
pub mod traits;

#[doc(hidden)]
pub use crate::mongodb_ext_derive::case;

#[cfg(feature = "mongodb-gridfs")]
pub use crate::traits::GridFSDb;

pub use crate::traits::{MongoClient, MongoCollection};

/// Defines the default type inside an [`Option`] for the `_id` field.
///
/// Re-export from [`mongodb::bson::oid::ObjectId`].
///
pub use mongodb::bson::oid::ObjectId as DefaultId;

/// Defines the default value used as schema version in [`MongoCollection::SCHEMA_VERSION`] if not specified otherwise.
pub const DEFAULT_SCHEMA_VERSION: i32 = 1;

/// This macro parses the per-collection parameters in a more usable format.
#[macro_export]
#[doc(hidden)]
macro_rules! parse_collection_params {
    (
        version: $version:literal,
        _id: $id:ident
        $($rest:tt)*
    ) => {
        $crate::expand_collection_version! {
            version = $version;
            id = $id;
            $($rest)*
        }
    };
    (
        _id: $id:ident,
        version: $version:literal
        $($rest:tt)*
    ) => {
        $crate::expand_collection_version! {
            version = $version;
            id = $id;
            $($rest)*
        }
    };
    (
        version: $version:literal
        $($rest:tt)*
    ) => {
        $crate::expand_collection_version! {
            version = $version;
            id = ;
            $($rest)*
        }
    };
    (
        _id: $id:ident
        $($rest:tt)*
    ) => {
        $crate::expand_collection_version! {
            version = ;
            id = $id;
            $($rest)*
        }
    };
    (
        $($rest:tt)*
    ) => {
        $crate::expand_collection_version! {
            version = ;
            id = ;
            $($rest)*
        }
    };
}

/// Expands schema version that is given in `<` / `>` behind each collection.
#[macro_export]
#[doc(hidden)]
macro_rules! expand_collection_version {
    (
        version = ;
        $($rest:tt)*
    ) => {
        $crate::expand_collection_id!{
            version = $crate::DEFAULT_SCHEMA_VERSION;
            $($rest)*
        }
    };
    (
        version = $version:literal;
        $($rest:tt)*
    ) => {
        $crate::expand_collection_id!{
            version = $version;
            $($rest)*
        }
    };
}

/// Expands collection _id that is given in `<` / `>` behind each collection.
#[macro_export]
#[doc(hidden)]
macro_rules! expand_collection_id {
    (
        version = $version:expr;
        id = ;
        $($rest:tt)*
    ) => {
        $crate::expand_collection!{
            @add_id
            version = $version;
            id = $crate::DefaultId;
            $($rest)*
        }
    };
    (
        version = $version:expr;
        id = none;
        $($rest:tt)*
    ) => {
        $crate::expand_collection!{
            @final
            version = $version;
            id = none;
            $($rest)*
        }
    };
    (
        version = $version:expr;
        id = $id:ty;
        $($rest:tt)*
    ) => {
        $crate::expand_collection!{
            @add_id
            version = $version;
            id = $id;
            $($rest)*
        }
    };
}

/// Expands one collection.
///
/// Needed internally, but has no big use on its own.
/// Thus hidden from documentation.
#[macro_export]
#[doc(hidden)]
macro_rules! expand_collection {
    // invoked with `_id: none`, thus assume `_id` is added already and finally expand to collection
    (
        @final
        version = $schema_version:expr;
        id = none;
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
        $crate::paste::paste! {
            #[doc = "Represents the [`" $coll_name "`] collection in mongodb."]
            #[derive($crate::serde::Deserialize, $crate::serde::Serialize, $crate::typed_builder::TypedBuilder)]
            #[serde(rename_all = "camelCase")]
            $(#[$additional_coll_attr])*
            pub struct $coll_name {
                $(
                    $(#[$additional_field_attr])*
                    pub $field: $field_type
                ),*
            }

            impl $crate::MongoCollection for $coll_name {
                const NAME: &'static str = $crate::case!($coll_name => Camel);
                const SCHEMA_VERSION: i32 = $schema_version;
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
        @add_id
        version = $schema_version:expr;
        id = $explicit_id_type:ty;
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
            @final
            version = $schema_version;
            id = none;
            $(#[$additional_coll_attr])*
            $coll_name {
                #[serde(skip_serializing_if = "std::option::Option::is_none")]
                #[serde(rename = "_id")]
                #[builder(default)]
                _id: std::option::Option<$explicit_id_type>,
                $(
                    $(#[$additional_field_attr])*
                    $field: $field_type
                ),*
            }-{
                #[doc = "Returns a reference to the `_id` field."]
                #[allow(dead_code)]
                pub fn id(&self) -> &Option<$explicit_id_type> {
                    &self._id
                }
                $($($inner_tokens2)+)?
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
        $(-{
            $($impl:tt)+
        })?
    ) => {
        $crate::paste::paste! {
            #[doc = "Client to interact with the `" $db_name "` database."]
            $(#[$additional_db_attr])*
            pub struct $db_name {
                pub client: $crate::mongodb::Client,
                pub database: $crate::mongodb::Database,
                $(
                    #[doc = "Handle to the `" $coll_name "` collection"]
                    pub [<$coll_name:snake:lower _coll>]: $crate::mongodb::Collection<schema::$coll_name>
                ),+
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

                fn database(&self) -> &$crate::mongodb::Database {
                    &self.database
                }
                fn client(&self) -> &$crate::mongodb::Client {
                    &self.client
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
/// For a detailled syntax demonstration see [Examples](#examples).
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
/// use mongodb_ext::{mongo_db, MongoClient, MongoCollection, DefaultId};
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
/// let oid = DefaultId::parse_str("0123456789ABCDEF01234567").unwrap();
/// some_document._id = Some(oid);
/// assert_eq!(
///     ser::to_string(&some_document).unwrap(),
///     String::from("{\"_id\":{\"$oid\":\"0123456789abcdef01234567\"},\"firstName\":\"alice\"}")
/// );
///
/// // constants store the collection / database names in `camelCase` + collection version
/// assert_eq!("someCollection", mongo::schema::SomeCollection::NAME);
/// assert_eq!(1, mongo::schema::SomeCollection::SCHEMA_VERSION);
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
/// assert_eq!(1, mongo::schema::MyFirstCollection::SCHEMA_VERSION);
/// assert_eq!("anotherCollection", mongo::schema::AnotherCollection::NAME);
/// assert_eq!(1, mongo::schema::AnotherCollection::SCHEMA_VERSION);
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
/// // This should fail beause there is no actual mongoDB service running at the specified
/// // connection.
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
/// // _id is now `u128` instead of `DefaultId`
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
/// use mongodb_ext::{mongo_db, DefaultId};
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
/// let oid = DefaultId::parse_str("0123456789ABCDEF01234567").unwrap();
/// let another_document = mongo::schema::Another {
///     _id: Some(oid),
///     some_field: 1,
/// };
/// // `_id` field disabled
/// let and_yet_another_document = mongo::schema::AndYetAnother {
///     name: String::from("Bob"),
///     email: String::from("bob@example.com")
/// };
/// ```
///
/// Each collection that does not have a parameter of `id: none` implements a function `id(&self)` that returns a reference to its ID:
///
/// ```rust
/// use mongodb_ext::{mongo_db, DefaultId};
///
/// mongo_db! {
///     SomeDatabase {
///         SomeCollection<_id: u128> {};
///         Another {};
///     }
/// }
///
/// // `id` returns `&Option<u128>`
/// let some_collection = mongo::schema::SomeCollection {
///     _id: Some(255),
/// };
/// assert_eq!(
///     *some_collection.id(),
///     Some(255)
/// );
///
/// // `id` returns `&Option<DefaultId>`
/// let oid = DefaultId::parse_str("0123456789ABCDEF01234567").unwrap();
/// let another = mongo::schema::Another {
///     _id: Some(oid.clone()),
/// };
/// assert_eq!(
///     *another.id(),
///     Some(oid)
/// );
/// ```
///
/// ## Versioning of your schema
///
/// Your database schema version is managed via [`MongoCollection::SCHEMA_VERSION`].
///
/// This can be modified like so:
///
/// ```rust
/// use mongodb_ext::{mongo_db, MongoCollection};
/// use serde_json::ser;
///
/// mongo_db! {
///     SomeDatabase {
///         // no schema version defaults to const `DEFAULT_SCHEMA_VERSION`
///         Items {
///             name: String,
///         };
///         // schema version of 200
///         Queue<version: 200> {
///             item: i32,
///         };
///         // schema version of 4
///         SomeCollection<version: 4, _id: none> {
///             first_name: String,
///         };
///         // schema version of 5
///         FourthCollection<_id: String, version: 5> {};
///     }
/// }
///
/// // default schema version is 1
/// assert_eq!(1, mongodb_ext::DEFAULT_SCHEMA_VERSION);
///
/// assert_eq!(mongo::schema::Items::SCHEMA_VERSION, 1);
/// assert_eq!(mongo::schema::Queue::SCHEMA_VERSION, 200);
/// assert_eq!(mongo::schema::SomeCollection::SCHEMA_VERSION, 4);
/// assert_eq!(mongo::schema::FourthCollection::SCHEMA_VERSION, 5);
/// ```
///
/// ## Serializing from [`json!`](serde_json::json) and [`doc!`](mongodb::bson::doc) macros
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
/// use mongodb_ext::{mongo_db, DefaultId};
///
/// mongo_db! {
///     // specify globally needed code in `mongo` here:
///     {
///         use std::collections::HashMap;
///     }
///     SomeDatabase {
///         // specify globally needed code in `schema` here:
///         {
///             use {
///                 std::collections::HashMap,
///                 mongodb::bson::oid::ObjectId
///             };
///         }
///
///         // specify collection-dependent code in an additional block below the
///         // collection connected with a `-`:
///         SomeCollection {
///             dict: HashMap<String, u32>,
///         }-{
///             pub fn some_collection_function() -> bool { true }
///         };
///         #[derive(Debug, PartialEq)]
///         AnotherCollection {}-{
///             pub fn from_id(id: ObjectId) -> Self { Self { _id: Some(id) } }
///         }
///     }-{
///         // specify implementations on the database handler here:
///         pub fn give_bool() -> bool { true }
///     }
/// }
///
/// assert!(mongo::SomeDatabase::give_bool());
/// assert!(mongo::schema::SomeCollection::some_collection_function());
///
/// let oid = DefaultId::parse_str("0123456789ABCDEF01234567").unwrap();
/// assert_eq!(
///     mongo::schema::AnotherCollection::from_id(oid.clone()),
///     mongo::schema::AnotherCollection {
///         _id: Some(oid),
///     },
/// );
/// ```
///
/// ## [`TypedBuilder`](typed_builder::TypedBuilder)
///
/// Each schema implements [`TypedBuilder`](typed_builder::TypedBuilder) which lets you create a collection more easily.
///
/// If `_id` is not set to `none`, the `_id` field will have a `builder` attribute set to `default`.
/// This enables you to skip specifying `_id` as [`None`].
///
/// ```rust
/// use mongodb_ext::{mongo_db, MongoClient, MongoCollection};
///
/// mongo_db! {
///     MyDatabase {
///         #[derive(Debug, PartialEq)]
///         MyCollection<version: 2, _id: u128> {
///             name: String,
///             counter: u32,
///             schema_version: i32
///         }
///     }
/// }
///
/// use mongo::schema::MyCollection;
///
/// assert_eq!(
///     // constructing using the builder
///     // note that no field `_id` is specified, thus `None` is used
///     MyCollection::builder()
///         .name("Alice".to_string())
///         .counter(1)
///         .schema_version(MyCollection::SCHEMA_VERSION)
///         .build(),
///     // constructing normally
///     MyCollection {
///         _id: None,
///         name: "Alice".to_string(),
///         counter: 1,
///         schema_version: MyCollection::SCHEMA_VERSION
///     }
/// );
/// ```
///
/// Combining the schema version with the typed builder can be very useful:
///
/// ```rust
/// use mongodb_ext::{mongo_db, MongoClient, MongoCollection};
///
/// mongo_db! {
///     MyDatabase {
///         {
///             use mongodb_ext::MongoCollection;
///         }
///         #[derive(Debug, PartialEq)]
///         MyCollection<version: 2, _id: u128> {
///             name: String,
///             counter: u32,
///             #[builder(default = <MyCollection as MongoCollection>::SCHEMA_VERSION)]
///             schema_version: i32
///         }
///     }
/// }
///
/// use mongo::schema::MyCollection;
///
/// assert_eq!(
///     // specifying no version takes version constant by default
///     MyCollection::builder()
///         .name("Alice".to_string())
///         .counter(255)
///         .build(),
///     MyCollection {
///         _id: None,
///         name: "Alice".to_string(),
///         counter: 255,
///         schema_version: 2
///     }
/// );
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
                $coll_name:ident$(<$($collection_param_name:ident: $collection_param_value:tt),+>)? {
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
                    $crate::parse_collection_params! {
                        $(
                            $($collection_param_name: $collection_param_value),+
                        )?

                        $(#[$additional_coll_attr])*

                        $coll_name {
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
