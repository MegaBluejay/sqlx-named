use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream, Parser},
    parse_quote,
    punctuated::Punctuated,
    token::{Bracket, Paren},
    AngleBracketedGenericArguments, Expr, Ident, LitInt, LitStr, Member, Token, Type,
};

use crate::util::read_file_src;

pub enum ArgType {
    Unnamed(Option<String>),
    Named(String),
}

pub struct Arg {
    pub typ: ArgType,
    pub val: Expr,
}

pub struct QueryInput {
    pub as_type: Option<Type>,
    pub sql: String,
    pub args: Punctuated<Arg, Token![,]>,
}

pub struct QueryVariant {
    pub file: bool,
    pub as_type: bool,
}

impl QueryVariant {
    pub fn parse_query(self) -> impl Parser<Output = QueryInput> {
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
                extract_all(input.parse_terminated(RawArg::parse, Token![,])?)?
            };

            Ok(QueryInput { as_type, sql, args })
        }
    }
}

fn extract_all(raw_seq: Punctuated<RawArg, Token![,]>) -> syn::Result<Punctuated<Arg, Token![,]>> {
    let mut out = Punctuated::new();
    for pair in raw_seq.into_pairs() {
        let (raw, comma) = pair.into_tuple();
        raw.extract(&mut out)?;
        if let Some(comma) = (!out.trailing_punct()).then_some(comma).flatten() {
            out.push_punct(comma);
        }
    }
    Ok(out)
}

enum RawArg {
    Single(Expr),
    Splat {
        _splat_token: Token![..],
        parent: Expr,
        _bracket_token: Bracket,
        children: Punctuated<RawChild, Token![,]>,
    },
}

fn get_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(path) => Some(path.path.get_ident()?.to_string()),
        _ => None,
    }
}

impl RawArg {
    fn extract(self, out: &mut Punctuated<Arg, Token![,]>) -> syn::Result<()> {
        match self {
            RawArg::Single(expr) => {
                let arg = match expr {
                    Expr::Path(path) => Arg {
                        typ: ArgType::Unnamed(path.path.get_ident().map(|ident| ident.to_string())),
                        val: Expr::Path(path),
                    },
                    Expr::Cast(cast) => Arg {
                        typ: ArgType::Unnamed(get_name(&cast.expr)),
                        val: Expr::Cast(cast),
                    },
                    Expr::Assign(ass) => Arg {
                        typ: ArgType::Named(get_name(&ass.left).ok_or_else(|| {
                            syn::Error::new_spanned(ass.left, "invalid arg name")
                        })?),
                        val: *ass.right,
                    },
                    expr => Arg {
                        typ: ArgType::Unnamed(None),
                        val: expr,
                    },
                };
                out.push(arg);
            }
            RawArg::Splat {
                _splat_token: _,
                parent,
                _bracket_token: _,
                children,
            } => {
                for pair in children.into_pairs() {
                    let (child, comma) = pair.into_tuple();

                    let RawChild {
                        assign,
                        dot_token,
                        target,
                        cast,
                    } = child;

                    let arg = Arg {
                        typ: match (assign, &target) {
                            (Some(ass), _) => ArgType::Named(ass.name.to_string()),
                            (_, RawChildTarget::Member(Member::Named(name))) => {
                                ArgType::Unnamed(Some(name.to_string()))
                            }
                            _ => ArgType::Unnamed(None),
                        },
                        val: parse_quote! { #parent #dot_token #target #cast },
                    };

                    out.push(arg);

                    if let Some(comma) = comma {
                        out.push_punct(comma);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Parse for RawArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![..]) {
            let content;
            RawArg::Splat {
                _splat_token: input.parse()?,
                parent: input.parse()?,
                _bracket_token: bracketed!(content in input),
                children: content.parse_terminated(RawChild::parse, Token![,])?,
            }
        } else {
            RawArg::Single(input.parse()?)
        })
    }
}

struct RawChild {
    assign: Option<Assign>,
    dot_token: Token![.],
    target: RawChildTarget,
    cast: Option<Cast>,
}

impl Parse for RawChild {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            assign: if input.peek(Ident) {
                Some(input.parse()?)
            } else {
                None
            },
            dot_token: input.parse()?,
            target: input.parse()?,
            cast: if input.peek(Token![as]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}

struct Assign {
    name: Ident,
    _eq_token: Token![=],
}

impl Parse for Assign {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _eq_token: input.parse()?,
        })
    }
}

struct Cast {
    as_token: Token![as],
    typ: Type,
}

impl Parse for Cast {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            as_token: input.parse()?,
            typ: input.parse()?,
        })
    }
}

impl ToTokens for Cast {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { as_token, typ } = self;
        as_token.to_tokens(tokens);
        typ.to_tokens(tokens);
    }
}

enum RawChildTarget {
    Member(Member),
    Method {
        method: Ident,
        turbofish: Option<AngleBracketedGenericArguments>,
        paren_token: Paren,
        args: Punctuated<Expr, Token![,]>,
    },
}

impl Parse for RawChildTarget {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(LitInt) {
            RawChildTarget::Member(Member::Unnamed(input.parse()?))
        } else {
            let ident = input.parse()?;
            let turbofish = if input.peek(Token![::]) {
                Some(AngleBracketedGenericArguments::parse_turbofish(input)?)
            } else {
                None
            };
            if turbofish.is_some() || input.peek(Paren) {
                let content;
                RawChildTarget::Method {
                    method: ident,
                    turbofish,
                    paren_token: parenthesized!(content in input),
                    args: content.parse_terminated(Expr::parse, Token![,])?,
                }
            } else {
                RawChildTarget::Member(Member::Named(ident))
            }
        })
    }
}

impl ToTokens for RawChildTarget {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RawChildTarget::Member(member) => member.to_tokens(tokens),
            RawChildTarget::Method {
                method,
                turbofish,
                paren_token,
                args,
            } => {
                method.to_tokens(tokens);
                turbofish.to_tokens(tokens);
                paren_token.surround(tokens, |tokens| {
                    args.to_tokens(tokens);
                });
            }
        }
    }
}
