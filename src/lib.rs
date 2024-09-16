use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use proc_macro2::{Ident, Span};
use quote::quote;
use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};
use syn::{
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    Expr, LitStr, Token, Type,
};

fn get_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(path) => Some(path.path.get_ident()?.to_string()),
        _ => None,
    }
}

fn parse_arg(input: ParseStream) -> syn::Result<Arg> {
    let expr = input.parse::<Expr>()?;

    Ok(match expr {
        Expr::Path(path) => Arg {
            typ: ArgType::Unnamed(path.path.get_ident().map(|ident| ident.to_string())),
            expr: Expr::Path(path),
        },
        Expr::Cast(cast) => Arg {
            typ: ArgType::Unnamed(get_name(&cast.expr)),
            expr: Expr::Cast(cast),
        },
        Expr::Assign(ass) => Arg {
            typ: ArgType::Named(
                get_name(&ass.left)
                    .ok_or_else(|| syn::Error::new_spanned(ass.left, "invalid arg name"))?,
            ),
            expr: *ass.right,
        },
        expr => Arg {
            typ: ArgType::Unnamed(None),
            expr,
        },
    })
}

// from sqlx-macros-core
fn read_file_src(source: &str, source_span: Span) -> syn::Result<String> {
    let file_path = resolve_path(source, source_span)?;

    std::fs::read_to_string(&file_path).map_err(|e| {
        syn::Error::new(
            source_span,
            format!(
                "failed to read query file at {}: {}",
                file_path.display(),
                e
            ),
        )
    })
}

// from sqlx-macros-core
fn resolve_path(path: impl AsRef<Path>, err_span: Span) -> syn::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        return Err(syn::Error::new(
            err_span,
            "absolute paths will only work on the current machine",
        ));
    }

    // requires `proc_macro::SourceFile::path()` to be stable
    // https://github.com/rust-lang/rust/issues/54725
    if path.is_relative()
        && !path
            .parent()
            .map_or(false, |parent| !parent.as_os_str().is_empty())
    {
        return Err(syn::Error::new(
            err_span,
            "paths relative to the current file's directory are not currently supported",
        ));
    }

    let base_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(
            err_span,
            "CARGO_MANIFEST_DIR is not set; please use Cargo to build",
        )
    })?;
    let base_dir_path = Path::new(&base_dir);

    Ok(base_dir_path.join(path))
}

enum ArgType {
    Unnamed(Option<String>),
    Named(String),
}

struct Arg {
    typ: ArgType,
    expr: Expr,
}

struct QueryInput {
    as_type: Option<Type>,
    sql: String,
    args: Punctuated<Arg, Token![,]>,
}

struct QueryVariant {
    file: bool,
    as_type: bool,
}

impl QueryVariant {
    fn parse_query(self) -> impl Parser<Output = QueryInput> {
        move |input: ParseStream| {
            let as_type = if self.as_type {
                let as_type = input.parse()?;
                input.parse::<Token![,]>()?;
                Some(as_type)
            } else {
                None
            };

            let lit_str = input.parse::<LitStr>()?;
            let sql = if self.file {
                read_file_src(&lit_str.value(), lit_str.span())?
            } else {
                lit_str.value()
            };

            let args = if input.is_empty() {
                Default::default()
            } else {
                input.parse::<Token![,]>()?;
                input.parse_terminated(parse_arg, Token![,])?
            };

            Ok(QueryInput { as_type, sql, args })
        }
    }
}

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
                ArgType::Named(_) => Err(syn::Error::new_spanned(arg.expr, "named arg")),
                ArgType::Unnamed(_) => Ok(arg.expr),
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
                        name.ok_or_else(|| syn::Error::new_spanned(&arg.expr, "unnamed arg"))
                    }
                }?;
                Ok::<_, syn::Error>((name, arg.expr))
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
    ($in_ident:ident, $out_ident:ident, $as_type:expr, $file:expr) => {
        #[proc_macro]
        pub fn $in_ident(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            let variant = QueryVariant {
                file: $file,
                as_type: $as_type,
            };
            let out_ident = Ident::new(stringify!($out_ident), Span::call_site());
            query_generic(variant, out_ident, input)
        }
    };
}

def_variant!(query, query, false, false);
def_variant!(query_as, query_as, true, false);
def_variant!(query_as_unchecked, query_as_unchecked, true, false);
def_variant!(query_file, query, false, true);
def_variant!(query_file_as, query_as, true, true);
def_variant!(query_file_as_unchecked, query_as_unchecked, true, true);
def_variant!(query_file_scalar, query_scalar, false, true);
def_variant!(
    query_file_scalar_unchecked,
    query_scalar_unchecked,
    false,
    true
);
def_variant!(query_file_unchecked, query_unchecked, false, true);
def_variant!(query_scalar, query_scalar, false, false);
def_variant!(query_scalar_unchecked, query_scalar_unchecked, false, false);
def_variant!(query_unchecked, query_unchecked, false, false);
