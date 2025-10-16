use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_str, Data, DeriveInput, Expr, Fields, Lit, Meta, MetaList, MetaNameValue};

/// Method
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
