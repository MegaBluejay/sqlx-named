use std::collections::HashSet;

use indexmap::IndexMap;
use proc_macro2::{Ident, Span};
use quote::quote;
use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};
use syn::parse::Parser as _;

use crate::parse::{ArgType, QueryInput, QueryVariant};

mod parse;
mod util;

fn expand(input: QueryInput, out_ident: Ident) -> syn::Result<proc_macro2::TokenStream> {
    let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, &input.sql)
        .tokenize()
        .expect("failed to tokenize sql");

    let unnamed = tokens
        .iter()
        .filter_map(|token| match token {
            sqlparser::tokenizer::Token::Placeholder(placeholder) => Some(placeholder),
            _ => None,
        })
        .all(|placeholder| placeholder.chars().skip(1).all(|c| c.is_ascii_digit()));

    let (sql, args) = if unnamed {
        let args = input
            .args
            .into_iter()
            .map(|arg| match arg.typ {
                ArgType::Named(_) => Err(syn::Error::new_spanned(arg.val, "named arg")),
                ArgType::Unnamed(_) => Ok(arg.val),
            })
            .collect::<Result<Vec<_>, _>>()?;

        (input.sql, args)
    } else {
        let named_args = input
            .args
            .into_iter()
            .map(|arg| {
                let name = match arg.typ {
                    ArgType::Named(name) => Ok(name),
                    ArgType::Unnamed(name) => {
                        name.ok_or_else(|| syn::Error::new_spanned(&arg.val, "unnamed arg"))
                    }
                }?;
                Ok::<_, syn::Error>((name, arg.val))
            })
            .collect::<Result<IndexMap<_, _>, _>>()?;

        let mut used = HashSet::new();

        for token in &mut tokens {
            if let sqlparser::tokenizer::Token::Placeholder(placeholder) = token {
                let arg = &placeholder[1..];
                let Some(index) = named_args.get_index_of(arg) else {
                    panic!("arg not given: {}", arg);
                };
                used.insert(arg.to_owned());
                *placeholder = format!("${}", index + 1);
            }
        }

        let unused = named_args
            .keys()
            .filter(|arg| !used.contains(*arg))
            .collect::<Vec<_>>();

        if !unused.is_empty() {
            panic!("unused args: {:?}", unused.as_slice());
        }

        let sql = tokens
            .into_iter()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
            .concat();

        (sql, named_args.into_values().collect::<Vec<_>>())
    };

    let as_type = input.as_type.map(|as_type| quote! { #as_type, });

    Ok(quote! {
        ::sqlx::#out_ident!(#as_type #sql, #(#args),*)
    })
}

fn query_generic(
    variant: QueryVariant,
    out_ident: Ident,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match variant
        .parse_query()
        .parse(input)
        .and_then(|input| expand(input, out_ident))
    {
        Ok(out) => out.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

macro_rules! def_variant {
    ($ident:ident) => {
        #[proc_macro]
        pub fn $ident(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            const IDENT_STR: &str = stringify!($ident);
            let variant = QueryVariant {
                file: const_str::contains!(IDENT_STR, "file"),
                as_type: const_str::contains!(IDENT_STR, "as"),
            };
            let out_ident = Ident::new(
                const_str::replace!(IDENT_STR, "_file", ""),
                Span::call_site(),
            );
            query_generic(variant, out_ident, input)
        }
    };
}

def_variant!(query);
def_variant!(query_as);
def_variant!(query_as_unchecked);
def_variant!(query_file);
def_variant!(query_file_as);
def_variant!(query_file_as_unchecked);
def_variant!(query_file_scalar);
def_variant!(query_file_scalar_unchecked);
def_variant!(query_file_unchecked);
def_variant!(query_scalar);
def_variant!(query_scalar_unchecked);
def_variant!(query_unchecked);
