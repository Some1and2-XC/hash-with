//! # HashWith
//!
//! `HashWith` is a procedural macro for deriving [`Hash`] on structs that contain
//! fields which do not implement [`Hash`] by default (like [`f64`]).
//! It supports custom hash functions per field.
//!
//! ## Inline Example
//!
//! The `#[hash_with(expr)]` notation takes in some expression and so long as the result is a type that
//! implements [`Hash`]. For example if you wanted to serialize a [`f64`] in your struct you can use
//! the following snippet.
//!
//! ```rust
//! use hash_with::HashWith;
//!
//! /// Some struct which needs to implement [`Hash`]
//! #[derive(HashWith)]
//! struct Brightness {
//!     /// The inner value with a hash function override.
//!     /// The `f64::to_bits()` method returns a `u64` which is why it can be used here.
//!     #[hash_with(self.inner.to_bits())]
//!     inner: f64,
//! }
//!
//! # use std::hash::{Hash, Hasher, DefaultHasher};
//! # impl Brightness {
//! #     fn get_hash(&self) -> u64 {
//! #         let mut hasher = DefaultHasher::new();
//! #         self.hash(&mut hasher);
//! #         return hasher.finish();
//! #     }
//! # }
//! #
//! // Sets values
//! let b1 = Brightness { inner: 1.1 };
//! let b2 = Brightness { inner: 2.2 };
//!
//! // Not equal in terms of their hash
//! assert_ne!(b1.get_hash(), b2.get_hash());
//! ```
//!
//! ## Function Call Example
//! With [`HashWith`] you can also call functions by name for the [`Hash`] implementation. This can
//! be useful for repeatedly creating a [`Hash`] implementation for multiple of the same datatype
//! in a struct.
//!
//! The function must however have the signature `Fn<T, H: std::hash::Hasher>(T, &mut H) -> ()`.
//! Basically what this means is the function must look something like the following example.
//!
//! ```rust
//! # use hash_with::HashWith;
//! # use std::hash::Hash;
//! #
//! /// A custom hash function for f64
//! fn hash_f64_bits<H: std::hash::Hasher>(val: &f64, state: &mut H) {
//!     val.to_bits().hash(state);
//! }
//!
//! /// An example struct.
//! #[derive(HashWith)]
//! struct Config {
//!     name: String,
//!     /// A brightness value which is hashed with [`hash_f64_bits`].
//!     #[hash_with = "hash_f64_bits"]
//!     brightness: f64,
//! }
//! ```
//!
//! # Ignoring Fields in Hash Calculation Example
//!
//! The `#[hash_without]` attribute can be applied to struct fields to exclude them
//! from the generated hash. This is useful for fields that should not affect equality
//! in hashed collections or when you want to ignore volatile or irrelevant data.
//!
//! ```rust
//! # use hash_with::HashWith;
//! #
//! #[derive(HashWith)]
//! struct User {
//!     id: u32,
//!     /// This field will be ignored in the hash calculation.
//!     #[hash_without]
//!     session_token: String,
//! }
//!
//! # use std::hash::{Hash, Hasher, DefaultHasher};
//! # impl User {
//! #     fn get_hash(&self) -> u64 {
//! #         let mut hasher = DefaultHasher::new();
//! #         self.hash(&mut hasher);
//! #         return hasher.finish();
//! #     }
//! # }
//! #
//! let user_1 = User { id: 1, session_token: "abc".into() };
//! let user_2 = User { id: 1, session_token: "xyz".into() };
//!
//! // The hash ignores `session_token`, so these are equal in terms of hash.
//! assert_eq!(user_1.get_hash(), user_2.get_hash());
//! ```

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_str, Data, DeriveInput, Expr, Fields, Lit, Meta, MetaList, MetaNameValue};

/// Derives [`Hash`] implementations for the attributes of struct, using custom per-field hash functions when needed.
///
/// This function is intended to be used on structs where some fields do not implement
/// [`std::hash::Hash`] by default (for example, [`f64`]). This implements the `#[hash_with(...)]`
/// macro on struct fields, allowing either an inline expression or a named function
/// to define how the field is hashed.
///
/// # Inline Expression Example
///
/// ```rust
/// use hash_with::HashWith;
///
/// #[derive(HashWith)]
/// struct Brightness {
///     /// Use a custom hash expression for this field.
///     #[hash_with(self.inner.to_bits())]
///     inner: f64,
/// }
///
/// # use std::hash::{Hash, Hasher, DefaultHasher};
/// # impl Brightness {
/// #     fn get_hash(&self) -> u64 {
/// #         let mut hasher = DefaultHasher::new();
/// #         self.hash(&mut hasher);
/// #         return hasher.finish();
/// #     }
/// # }
/// #
/// let b1 = Brightness { inner: 1.1 };
/// let b2 = Brightness { inner: 2.2 };
///
/// assert_ne!(b1.get_hash(), b2.get_hash());
/// ```
///
/// # Function Call Example
///
/// ```rust
/// use hash_with::HashWith;
/// # use std::hash::Hash;
///
/// /// A custom hash function for f64
/// fn hash_f64_bits<H: std::hash::Hasher>(val: &f64, state: &mut H) {
///     val.to_bits().hash(state);
/// }
///
/// #[derive(HashWith)]
/// struct Config {
///     name: String,
///     /// This field is hashed using the custom function [`hash_f64_bits`].
///     #[hash_with = "hash_f64_bits"]
///     brightness: f64,
/// }
/// ```
///
/// # Ignoring Fields in Hash Calculation Example
///
/// The `#[hash_without]` attribute can be applied to struct fields to exclude them
/// from the generated hash. This is useful for fields that should not affect equality
/// in hashed collections or when you want to ignore volatile or irrelevant data.
///
/// ```rust
/// use hash_with::HashWith;
///
/// #[derive(HashWith)]
/// struct User {
///     id: u32,
///     /// This field will be ignored in the hash calculation.
///     #[hash_without]
///     session_token: String,
/// }
///
/// # use std::hash::{Hash, Hasher, DefaultHasher};
/// # impl User {
/// #     fn get_hash(&self) -> u64 {
/// #         let mut hasher = DefaultHasher::new();
/// #         self.hash(&mut hasher);
/// #         return hasher.finish();
/// #     }
/// # }
/// #
/// let user_1 = User { id: 1, session_token: "abc".into() };
/// let user_2 = User { id: 1, session_token: "xyz".into() };
///
/// // The hash ignores `session_token`, so these are equal in terms of hash.
/// assert_eq!(user_1.get_hash(), user_2.get_hash());
/// ```
#[proc_macro_derive(HashWith, attributes(hash_with, hash_without))]
pub fn derive_hash_with(input: TokenStream) -> TokenStream {

    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut hash_stmts = Vec::new();

    // Gets the data
    if let Data::Struct(data_struct) = &input.data {
        // Gets the named fields
        if let Fields::Named(fields) = &data_struct.fields {

            // Goes through all the fields
            for field in &fields.named {

                // Gets the name of each field
                let field_name = field.ident.as_ref().unwrap();
                let mut custom_hash_fn = None;

                for attr in &field.attrs {
                    if attr.path().is_ident("hash_with") {

                        let function_name = match &attr.meta {
                            Meta::NameValue(
                                MetaNameValue {
                                    value: Expr::Lit(syn::ExprLit {
                                        lit: Lit::Str(
                                             function_name_str_with_quotes
                                        ),
                                        ..
                                    }),
                                .. }
                            ) => {
                                let func = function_name_str_with_quotes.parse_with(syn::Path::parse_mod_style).expect("Failed to parse string!");
                                // let func = parse_str::<Expr>(&function_name_str_with_quotes.value()).unwrap_or_else(|_| panic!("Failed to parse string!"));
                                quote! {
                                    #func(&self.#field_name, state);
                                }
                            },
                            // Handles the list implementation (i.e. `#[hash_with( ... )]`)
                            Meta::List(
                                MetaList {
                                    tokens,
                                .. }
                            ) => {
                                let expr = parse_str::<Expr>(&tokens.to_string()).expect("Failed to parse tokens").to_token_stream();
                                quote! {
                                    #expr.hash(state);
                                }
                            },
                            _ => panic!("Failed to parse `{}` for `hash_with` macro.", attr.to_token_stream().to_string()),
                        };

                        custom_hash_fn = Some(function_name);

                    }

                    if attr.path().is_ident("hash_without") {
                        custom_hash_fn = Some(proc_macro2::TokenStream::new());
                    }

                }

                let hash_function = match custom_hash_fn {
                    Some(tokens) => tokens,
                    None => quote! { self.#field_name.hash(state); }
                };

                hash_stmts.push(hash_function);

            }

        }
        else {
            panic!("HashWith only supports structs with named fields!");
        }

    }

    let expanded = quote! {
        impl std::hash::Hash for #name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                #(#hash_stmts)*
            }
        }
    };

    return TokenStream::from(expanded);

}
