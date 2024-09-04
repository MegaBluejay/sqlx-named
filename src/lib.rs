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
    Expr, LitStr, Token, Type,
};

fn get_name(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Path(path) => Ok(path.path.require_ident()?.to_string()),
        _ => Err(syn::Error::new_spanned(expr, "invalid arg name")),
    }
}

fn parse_arg(input: ParseStream) -> syn::Result<(String, Expr)> {
    let expr = input.parse::<Expr>()?;

    Ok(match expr {
        Expr::Path(path) => (path.path.require_ident()?.to_string(), Expr::Path(path)),
        Expr::Cast(cast) => (get_name(&cast.expr)?, Expr::Cast(cast)),
        Expr::Assign(ass) => (get_name(&ass.left)?, *ass.right),
        _ => return Err(syn::Error::new_spanned(expr, "invalid arg")),
    })
}

fn parse_args(input: ParseStream) -> syn::Result<IndexMap<String, Expr>> {
    Ok(input
        .parse_terminated(parse_arg, Token![,])?
        .into_iter()
        .collect())
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

struct QueryInput {
    as_type: Option<Type>,
    sql: String,
    args: IndexMap<String, Expr>,
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
                parse_args(input)?
            };

            Ok(QueryInput { as_type, sql, args })
        }
    }
}

fn expand(input: QueryInput, out_ident: Ident) -> proc_macro2::TokenStream {
    let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, &input.sql)
        .tokenize()
        .expect("failed to tokenize sql");

    let mut used = HashSet::new();

    for token in &mut tokens {
        if let sqlparser::tokenizer::Token::Placeholder(placeholder) = token {
            let arg = &placeholder[1..];
            let Some(index) = input.args.get_index_of(arg) else {
                panic!("arg not given: {}", arg);
            };
            used.insert(arg);
            *placeholder = format!("${}", index + 1);
        }
    }

    let unused = input
        .args
        .keys()
        .filter(|arg| !used.contains(arg))
        .collect::<Vec<_>>();

    if !unused.is_empty() {
        panic!("unused args: {:?}", unused.as_slice());
    }

    let as_type = input.as_type.map(|as_type| quote! { #as_type, });

    let sql = tokens
        .into_iter()
        .map(|token| token.to_string())
        .collect::<Vec<_>>()
        .concat();

    let args = input.args.into_values();

    quote! {
        ::sqlx::#out_ident!(#as_type #sql, #(#args),*)
    }
}

fn query_generic(
    variant: QueryVariant,
    out_ident: Ident,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = match variant.parse_query().parse(input) {
        Ok(input) => input,
        Err(err) => return proc_macro::TokenStream::from(err.to_compile_error()),
    };
    expand(input, out_ident).into()
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
