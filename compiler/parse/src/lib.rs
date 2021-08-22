#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/abogado-parse/0.0.0", // remember to bump!
)]

mod ast;

use abogado_lex as lex;

use chumsky::prelude::*;
pub use chumsky::Parser;

use ast::{Expr, Ident, If, Statement};
use lex::{
    spanned::S,
    Keyword::{self, *},
    Sigil, Span, Token,
};

type Tok = S<Token>;
macro_rules! kw_filters {
    ($($kw:ident => $kw_var:tt),* $(,)?) => {$(
        #[allow(unused)]
        fn $kw() -> impl Clone + Parser<Tok, Tok, Error = Simple<Tok, Span>> {
            filter::<_, _, Simple<Tok, Span>>(|t: &Tok| matches!(t.inner, Token::Keyword(Keyword::$kw_var)))
        }
    )*}
}

kw_filters! {
    set        => Set,
    to         => To,
    is         => Is,
    call       => Do,
    using      => Using,
    get        => Get,
    same       => Same,
    different  => Different,
    more       => More,
    less       => Less,
    also       => Also,
    and        => And,
    not        => Not,
    while_     => While,
    keep       => Keep,
    until      => Until,
    run        => Run,
    for_loop   => For,
    inside     => In,
    procedure  => Procedure,
    takes      => Takes,
    does       => Does,
    otherwise  => Else,
}

macro_rules! sigil_filters {
    ($($sigil:ident => $sigil_var:tt),* $(,)?) => {$(
        #[allow(unused)]
        fn $sigil() -> impl Clone + Parser<Tok, Tok, Error = Simple<Tok, Span>> {
            filter::<_, _, Simple<Tok, Span>>(|t: &Tok| matches!(t.inner, Token::Sigil(Sigil::$sigil_var)))
        }
    )*}
}

sigil_filters! {
    start_list  => StartList,
    end_list    => EndList,
    start_block => StartBlock,
    end_block   => EndBlock,
    comma       => Comma,
    question    => Question,
    exclamation => Exclamation,
    dot         => Dot,
    semicolon   => Semicolon,
}

fn ident() -> impl Parser<Tok, S<Ident>, Error = Simple<Tok, Span>> {
    filter(|t: &Tok| matches!(t.inner, Token::Ident(_))).map(|t: Tok| {
        t.map(|tok| match tok {
            Token::Ident(ident) => ident,
            _ => unreachable!(),
        })
    })
}

// TODO
//
// eventually support:
//   - inner
//   - inner as well as inner
//   - inner,+ inner, and inner
fn comma_delimited<T>(
    inner: impl Clone + Parser<Tok, S<T>, Error = Simple<Tok, Span>>,
) -> impl Clone + Parser<Tok, Vec<S<T>>, Error = Simple<Tok, Span>> {
    let comma = filter(|tok: &Tok| matches!(tok.inner, Token::Sigil(Sigil::Comma)));

    inner
        .clone()
        .chain(comma.clone().padding_for(inner.clone()).repeated())
        .or_not()
        .map(|m| m.unwrap_or(vec![]))
}


pub fn expr() -> impl Clone + Parser<Tok, S<Expr>, Error = Simple<Tok, Span>> {
    recursive::<Tok, S<Expr>, _, _, Simple<Tok, Span>>(|expr| {
        let assign = set()
            .then(ident())
            .then(to())
            .then(expr.clone())
            .map(|(((set, name), to), expr)| S {
                inner: Expr::Assign(ast::Assign {
                    name: name.clone(),
                    to: Box::new(expr.clone()),
                }),
                span: set.clone() | to.clone(),
                style: set & name & to & expr,
            });

        let conditional = question().padding_for(expr.clone())
            .then(question())
            .then(expr.clone())
            .then(otherwise().then(expr.clone()).or_not())
            .map(|(((cond, ques), body), else_)| {
                let mut style = cond.clone() & ques.clone() & body.clone();
                let mut span = cond.clone() | ques | body.clone();

                let otherwise = if let Some((otherwise, else_)) = else_ {
                    style = style & otherwise.clone() & else_.clone();
                    span = span | otherwise | else_.clone();

                    Some(Box::new(else_))
                } else {
                    None
                };

                S {
                    style, span, inner: Expr::If(If {
                        cond: Box::new(cond),
                        then: Box::new(body),
                        otherwise,
                    })
                }
            });

        assign
            .or(ident().map(|i| i.map(Expr::Ident)))
            .or(conditional)
    })
}

pub fn statement() -> impl Parser<Tok, S<Statement>, Error = Simple<Tok, Span>> {
    let expr = expr().then(dot()).map(|(exp, punc)| {
        S{inner: Statement::Expr(exp.clone()),
            span: exp.clone() | punc.clone(),
            style: exp.clone() & punc.clone(),
        }
    });

    expr
}
