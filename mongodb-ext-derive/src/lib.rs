//! This crate is the `proc_macro` library associated with the `mongodb_ext` crate.
//!
//! Since recent changes, this crate has an unfortunate name.
//! "derive" is not quite correct, because this crate's purpose is to provide macros, not **derive** macros explicitly.
//!
//! This crate currently provides one macro: [`case!`].

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
        proc_macro2::Span,
        quote::ToTokens,
        syn::{
            parse::{Error as SynError, Parse, ParseStream, Result as SynResult},
            spanned::Spanned,
            token::FatArrow,
            LitStr, Path,
        },
    },
    std::convert::From,
};

struct CaseInput(LitStr);
impl Parse for CaseInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // parse first path
        let first_path: Path = input.parse::<Path>()?;
        // get first path's span
        let first_span: Span = first_path.span();
        // convert first path to String
        let first_string: String = if let Some(last_of_first) = first_path.segments.last() {
            last_of_first.ident.to_string()
        } else {
            // throw error if there is no last element
            return Err(SynError::new(first_span, "Cannot get last element of path"));
        };

        // parse `=>`
        let _: FatArrow = input.parse::<FatArrow>()?;

        // parse last (second) case
        let last_path: Path = input.parse::<Path>()?;
        // get last path's span
        let last_span: Span = last_path.span();
        // convert last path to String
        let last_string: String = if let Some(last_of_last) = last_path.segments.last() {
            last_of_last.ident.to_string()
        } else {
            // throw error if there is no last element
            return Err(SynError::new(last_span, "Cannot get last element of path"));
        };

        // parse case as instance of `convert_case::Case`
        let mut case: Option<Case> = None;
        for c in Case::all_cases() {
            if format!("{:?}", c).eq(&last_string) {
                case = Some(c);
                break;
            }
        }
        if case.is_none() {
            return Err(SynError::new(
                last_span,
                "Cannot parse case parameter as `Case`",
            ));
        }
        let case = case.unwrap();

        // change first path's case and return
        let parsed_path: String = first_string.to_case(case);
        Ok(Self(LitStr::new(&parsed_path, first_span)))
    }
}

/// Small macro that converts the input path to a given case.
///
/// The general accepted format is: `case!(path::to::Type => Case)`
///
/// Hereby
/// - `path::to::Type` can be any path. It does not need to exist.
/// - `=>` is just a fat arrow that separates the two parameters.
/// - `Case` is any path that points to any value of the [`convert_case`] crate's [`Case`] enum.
///
/// This macro always expands to a [`&str`] literal ([`LitStr`](struct@syn::LitStr)).
///
/// # Examples
///
/// The identifier given does not need to be an existing type:
///
/// ```rust
/// use mongodb_ext_derive::case;
///
/// // `MyImaginaryType` is not imported, but works anyways
/// // `Case` is not imported, but works anyways
/// assert_eq!(
///     case!(MyImaginaryType => Case::Camel),
///     "myImaginaryType"
/// );
/// ```
///
/// If a path is given, only the last element will be parsed:
///
/// ```rust
/// use mongodb_ext_derive::case;
///
/// assert_eq!(
///     case!(std::collection::HashMap => Snake),
///     "hash_map"
/// );
/// ```
///
/// ```rust
/// use mongodb_ext_derive::case;
///
/// assert_eq!(
///     case!(std::this::r#type::does::not::exist::THIS_CONSTANT_DOES_NOT_EXIST => Pascal),
///     "ThisConstantDoesNotExist"
/// );
///
/// assert_eq!(
///     case!(std::this::r#type::does::not::exist::this_function_does_not_exist => Title),
///     "This Function Does Not Exist"
/// );
///
/// assert_eq!(
///     case!(std::this::r#type::does::not::exist::ThisTypeDoesNotExist => Camel),
///     "thisTypeDoesNotExist"
/// );
/// ```
#[proc_macro]
pub fn case(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as CaseInput)
        .0
        .to_token_stream()
        .into()
}
