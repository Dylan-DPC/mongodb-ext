//! Derive a constant value holding the struct's name as [`&str`].
//!
//! All case modifications from the [`Case`] enum can be used to modify the value of the constant.
//!
//! The constant's key can be renamed.

extern crate convert_case;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
extern crate quote;

use {
    crate::{
        convert_case::{Case, Casing},
        proc_macro::TokenStream,
        proc_macro2::{Literal, TokenStream as TokenStream2},
        quote::{quote, ToTokens},
        syn::{DeriveInput, Ident, LitStr, TypePath},
    },
    std::convert::From,
};

/// Derivable macro that provides you with a constant variable that holds the name of the item derived on.
///
/// Key and value of the constant can be manipulated to some degree.
///
/// This macro uses the [`convert_case`] crate internally.
/// Its [`Case`] enum does not implement much conversion, so this macro relies on its [`Debug`] implementation.
///
/// # Examples
///
/// The default implementation names the key after the struct in SCREAMING_SNAKE_CASE and the value after the struct's name as-is.
///
/// ```rust
/// use mongodb_ext_derive::ConstName;
///
/// #[derive(ConstName)]
/// struct MyStruct {
///     some_field: u16,
/// }
///
/// assert_eq!("MyStruct", MY_STRUCT);
/// ```
///
/// ## Customizing the [`ConstName`] derive
///
/// Change the constant's value's case by using any type from [`Case`] inside of `#[const_name_value()]`.
///
/// ```rust
/// use mongodb_ext_derive::ConstName;
/// use convert_case::Case;
///
/// #[derive(ConstName)]
/// #[const_name_value(Case::Snake)]
/// struct MyStruct {
///     some_field: u16,
/// }
///
/// assert_eq!("my_struct", MY_STRUCT);
/// ```
///
/// ```rust
/// use mongodb_ext_derive::ConstName;
///
/// #[derive(ConstName)]
/// #[const_name_value(convert_case::Case::Camel)]
/// struct MyStruct {
///     some_field: u16,
/// }
///
/// assert_eq!("myStruct", MY_STRUCT);
/// ```
///
/// You do not actually need to specify a valid path to [`Case`], but all [`Case`]'s types are dynamically supported:
///
/// ```rust
/// use mongodb_ext_derive::ConstName;
///
/// #[derive(ConstName)]
/// // Specifying "Camel" without having Case in scope, still works.
/// #[const_name_value(Camel)]
/// struct MyStruct {
///     some_field: u16,
/// }
///
/// assert_eq!("myStruct", MY_STRUCT);
/// ```
///
/// Change the constant's key by specifying a new key with `#[const_name_key()]`.
///
/// ```rust
/// use mongodb_ext_derive::ConstName;
///
/// #[derive(ConstName)]
/// #[const_name_key("STRUCT_NAME")]
/// struct MyStruct {
///     some_field: u16,
/// }
///
/// assert_eq!("MyStruct", STRUCT_NAME);
/// ```
///
/// Invalid attributes end up in a panic whilst compiling.
///
/// ```compile_fail
/// use mongodb_ext_derive::ConstName;
///
/// #[derive(ConstName)]
/// #[const_name_value(NotACaseValue)]
/// struct MyStruct {
///     some_field: u16,
/// }
/// ```
///
/// ```compile_fail
/// use mongodb_ext_derive::ConstName;
///
/// #[derive(ConstName)]
/// #[const_name_key(THIS_IS_NOT_A_STRING_LITERAL)]
/// struct MyStruct {
///     some_field: u16,
/// }
/// ```
#[proc_macro_derive(ConstName, attributes(const_name_value, const_name_key))]
pub fn const_name_derive(items: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(items as DeriveInput);

    // save span for later
    let key_value_span = ast.ident.span();
    // create default constant variable value
    let mut value = ast.ident.to_string();
    // create default constant variable name
    let mut key = value.to_case(Case::UpperSnake);

    // loop through all attributes on the struct to find relevant ones
    'outer: for attr in ast.attrs {
        if attr.path.is_ident("const_name_key") {
            key = attr
                // parse key as literal
                .parse_args::<Literal>()
                .expect("Could not parse `const_name` derive attribute's arguments as literal")
                // convert to string
                .to_string()
                // strip quotes
                .replace("\"", "");
        } else if attr.path.is_ident("const_name_value") {
            let given_case = attr
                // parse value case as path
                .parse_args::<TypePath>()
                .expect("Could not parse `name_case` derive attribute's arguments as path");
            // get last part of parsed path and convert it to string
            let given_case: String = given_case
                .path
                .segments
                .last()
                .expect("Could not get identifier of given path in `name_case` derive attribute")
                .ident
                .to_string();
            // search for a matching case in Case enum
            for case in Case::all_cases() {
                if format!("{:?}", case).eq(&given_case) {
                    // case found, set it and continue the outer loop
                    value = value.to_case(Case::from(case));
                    continue 'outer;
                }
            }
            // no matching cases found
            panic!(
                "Supplied attribute to `name_case` is invalid ({})",
                given_case
            );
        }
    }

    let mut ts: TokenStream2 = quote!(pub const);
    Ident::new(&key, key_value_span).to_tokens(&mut ts);
    ts.extend(quote!(: &str =));
    LitStr::new(&value, key_value_span).to_tokens(&mut ts);
    ts.extend(quote!(;));
    ts.into()
}
