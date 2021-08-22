#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/abogado-parse/0.0.0", // remember to bump!
)]

pub mod ast;

use abogado_lex as lex;

use chumsky::prelude::*;
pub use chumsky::Parser;

use ast::*;
use lex::{
    spanned::S,
    Keyword,
    Op, Sigil, Span, Token,
};

type Tok = S<Token>;
macro_rules! kw_filters {
    ($($kw:ident => $kw_var:tt),* $(,)?) => {$(
        #[allow(unused)]
        fn $kw() -> impl Clone + Parser<Tok, Tok, Error = Simple<Tok, Span>> {
            filter::<_, _, Simple<Tok, Span>>(|t: &Tok| matches!(t.inner, Token::Keyword(Keyword::$kw_var)))
                .labelled(stringify!($kw_var))
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
    also       => Also,
    and        => And,
    not        => Not,
    while_loop => While,
    keep       => Keep,
    until      => Until,
    run        => Run,
    for_loop   => For,
    inside     => In,
    procedure  => Procedure,
    takes      => Takes,
    does       => Does,
    otherwise  => Else,
    emit       => Emit,
    from       => From,
}

macro_rules! sigil_filters {
    ($($sigil:ident => $sigil_var:tt),* $(,)?) => {$(
        #[allow(unused)]
        fn $sigil() -> impl Clone + Parser<Tok, Tok, Error = Simple<Tok, Span>> {
            filter::<_, _, Simple<Tok, Span>>(|t: &Tok| matches!(t.inner, Token::Sigil(Sigil::$sigil_var)))
                .labelled(stringify!($sigil))
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

macro_rules! bin_op_filters {
    ($($op:ident => $op_ident:tt),* $(,)?) => {$(
        #[allow(unused)]
        fn $op() -> impl Clone + Parser<Tok, Tok, Error = Simple<Tok, Span>> {
            filter::<_, _, Simple<Tok, Span>>(|t: &Tok| matches!(t.inner, Token::Operator(Op::$op_ident)))
        }
    )*};
}

bin_op_filters! {
    add => Add,
    sub => Sub,
    mul => Mul,
    div => Div,
    eq  => Eq,
    lt  => Lt,
    gt  => Gt,
}

macro_rules! un_op_filters {
    ($($op:ident => $op_ident:pat),* $(,)?) => {$(
        #[allow(unused)]
        fn $op() -> impl Clone + Parser<Tok, Tok, Error = Simple<Tok, Span>> {
            filter::<_, _, Simple<Tok, Span>>(|t: &Tok| matches!(t.inner, $op_ident))
        }
    )*};
}

un_op_filters! {
    un_neg => Token::Operator(Op::Sub),
    un_not => Token::Keyword(Keyword::Not),
}

fn ident() -> impl Clone + Parser<Tok, S<Ident>, Error = Simple<Tok, Span>> {
    filter(|t: &Tok| matches!(t.inner, Token::Ident(_))).map(|t: Tok| {
        t.map(|tok| match tok {
            Token::Ident(ident) => ident,
            _ => unreachable!(),
        })
    })
}

// TODO
//
// eventually support (for comma):
//   - inner
//   - inner as well as inner
//   - inner,+ inner, and inner
fn delimited<T>(
    inner: impl Clone + Parser<Tok, S<T>, Error = Simple<Tok, Span>>,
    delimiter: Sigil,
) -> impl Clone + Parser<Tok, Vec<S<T>>, Error = Simple<Tok, Span>> {
    let comma =
        filter(move |tok: &Tok| matches!(tok.inner, Token::Sigil(sigil) if sigil == delimiter));

    inner
        .clone()
        .chain(comma.clone().padding_for(inner.clone()).repeated())
        .or_not()
        .map(|m| m.unwrap_or(vec![]))
}

pub fn expr() -> impl Clone + Parser<Tok, S<Expr>, Error = Simple<Tok, Span>> {
    recursive::<Tok, S<Expr>, _, _, Simple<Tok, Span>>(|expr| {
        let list = start_list()
            .then(delimited(expr.clone(), Sigil::Comma))
            .then(end_list())
            .map(|((start, content), end)| {
                S{
                    span: content.iter().fold(start.clone() | start.clone(), |acc, i| acc | i.clone()) | end.clone(),
                    style: content.iter().fold(start.clone() & start.clone(), |acc, i| acc & i.clone()) & end.clone(),
                    inner: Expr::List(ast::List(content)),
                }
            })
            .labelled("list");

        let num = filter(|t: &Tok| matches!(t.inner, Token::Num(_)))
            .map(|t: Tok| {
                t.map(|tok| match tok {
                    Token::Num(num) => Expr::Num(num),
                    _ => unreachable!(),
                })
            })
            .labelled("num");
        let string = filter(|t: &Tok| matches!(t.inner, Token::StringConst(_)))
            .map(|t: Tok| {
                t.map(|tok| match tok {
                    Token::StringConst(s) => Expr::String(s),
                    _ => unreachable!(),
                })
            })
            .labelled("string");

        let op = un_not()
            .map(|t| t.map(|_| UnaryOperator::Not))
            .or(un_neg().map(|t| t.map(|_| UnaryOperator::Neg)));

        let atom = recursive::<Tok, S<Expr>, _, _, Simple<Tok, Span>>(|atom| {
            let un_op = op.then(atom.clone()).map(|(op, arg): (S<UnaryOperator>, S<Expr>)| {
                S {
                    span: op.clone() | arg.clone(),
                    style: op.clone() & arg.clone(),
                    inner: Expr::UnOp(UnOp {
                        op,
                        expr: Box::new(arg),
                    }),
                }
            })
            .labelled("unary op");

            ident()
                .map(|i| i.map(Expr::Ident))
                .or(num)
                .or(string)
                .or(list)
                .or(un_op)
                .labelled("atom")
        });

        let extract_operator_from_token = |t: Tok| {
            t.map(|t: Token| match t {
                Token::Operator(o) => o,
                _ => unreachable!(),
            })
        };

        let op = mul()
            .map(extract_operator_from_token)
            .or(div().map(extract_operator_from_token));
        let product = atom
            .clone()
            .then(op.then(atom).repeated())
            .foldl(|lhs, (op, rhs)| S {
                span: lhs.clone() | op.clone() | rhs.clone(),
                style: lhs.clone() & op.clone() & rhs.clone(),
                inner: Expr::BinOp(BinOp {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                }),
            })
            .labelled("product");

        let op = add()
            .map(extract_operator_from_token)
            .or(sub().map(extract_operator_from_token));
        let sum = product
            .clone()
            .then(op.then(product.clone()).repeated())
            .foldl(|lhs, (op, rhs)| S {
                span: lhs.clone() | op.clone() | rhs.clone(),
                style: lhs.clone() & op.clone() & rhs.clone(),
                inner: Expr::BinOp(BinOp {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                }),
            })
            .labelled("sum");

        let op = eq()
            .map(extract_operator_from_token)
            .or(lt().map(extract_operator_from_token))
            .or(gt().map(extract_operator_from_token));
        let compare = sum
            .clone()
            .then(op.then(sum.clone()).repeated())
            .foldl(|lhs, (op, rhs)| S {
                span: lhs.clone() | op.clone() | rhs.clone(),
                style: lhs.clone() & op.clone() & rhs.clone(),
                inner: Expr::BinOp(BinOp {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                }),
            })
            .labelled("compare");

        let emit = emit()
            .then(expr.clone())
            .then(exclamation())
            .map(|((emit, expr), exclam)| S {
                span: emit.clone() | expr.clone() | exclam.clone(),
                style: emit.clone() & expr.clone() & exclam.clone(),
                inner: Expr::Print(expr.map(Box::new)),
            })
            .labelled("print");

        let get = get()
            .then(expr.clone())
            .then(from())
            .then(expr.clone())
            .map(|(((g, idx), f), list)| S {
                span: g.clone() | idx.clone() | f.clone() | list.clone(),
                style: g.clone() & idx.clone() & f.clone() & list.clone(),
                inner: Expr::Get(ast::Get {
                    index: Box::new(idx),
                    from: Box::new(list),
                }),
            })
            .labelled("list get");

        // TODO: the trailing_dot logic is never run!
        let block = start_block()
            .then(delimited(expr.clone(), Sigil::Dot))
            .then(dot().or_not())
            .then(end_block())
            .map(|(((s, exprs), trailing_dot), e)| S {
                span: exprs
                    .iter()
                    .fold(s.clone() | s.clone(), |acc, e| acc | e.clone())
                    | e.clone(),
                style: exprs
                    .iter()
                    .fold(s.clone() & s.clone(), |acc, e| acc & e.clone())
                    & e.clone(),
                inner: Expr::Block(Block {
                    body: {
                        let exprs = if trailing_dot.is_none() && exprs.len() >= 1 {
                            &exprs[..exprs.len() - 1]
                        } else {
                            &exprs[..]
                        };

                        // the outer S<Statement>'s span should include the `.`... but it doesn't!
                        //
                        // this is okay for now (TODO)
                        exprs
                            .iter()
                            .map(|e| S {
                                span: e.span.clone(),
                                style: e.style.clone(),
                                inner: Statement::Expr(e.clone()),
                            })
                            .collect()
                    },
                    end: {
                        if trailing_dot.is_none() && exprs.len() >= 1 {
                            Some(Box::new(exprs.last().unwrap().clone()))
                        } else {
                            None
                        }
                    },
                }),
            })
            .labelled("block");

        let call = call()
            .then(ident())
            .then(using())
            .then(delimited(expr.clone(), Sigil::Comma))
            .map(|(((c, func), u), args)| S {
                span: args
                    .iter()
                    .fold(c.clone() | func.clone() | u.clone(), |acc, a| {
                        acc | a.clone()
                    }),
                style: args
                    .iter()
                    .fold(c.clone() & func.clone() & u.clone(), |acc, a| {
                        acc & a.clone()
                    }),
                inner: Expr::Call(Call { name: func, args }),
            })
            .labelled("call");

        let assign =
            set()
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
                })
                .labelled("assignment");

        let conditional = is()
            .then(expr.clone())
            .then(question())
            .then(expr.clone())
            .then(otherwise().then(expr.clone()).or_not())
            .map(|((((is, cond), ques), body), else_)| {
                let mut style = is.clone() & cond.clone() & ques.clone() & body.clone();
                let mut span = is | cond.clone() | ques | body.clone();

                let otherwise = if let Some((otherwise, else_)) = else_ {
                    style = style & otherwise.clone() & else_.clone();
                    span = span | otherwise | else_.clone();

                    Some(Box::new(else_))
                } else {
                    None
                };

                S {
                    style,
                    span,
                    inner: Expr::If(If {
                        cond: Box::new(cond),
                        then: Box::new(body),
                        otherwise,
                    }),
                }
            })
            .labelled("conditional");

        conditional
            .or(assign)
            .or(call)
            .or(block)
            .or(get)
            .or(emit)
            .or(compare)
    })
}

pub fn statement() -> impl Parser<Tok, S<Statement>, Error = Simple<Tok, Span>> {
    let expr_statement = expr().then(dot()).map(|(exp, punc)| S {
        inner: Statement::Expr(exp.clone()),
        span: exp.clone() | punc.clone(),
        style: exp.clone() & punc.clone(),
    });

    let for_loop = run()
        .then(expr())
        .then(for_loop())
        .then(ident())
        .then(inside())
        .then(expr())
        .map(|(((((r, expr), f), binding), i), iteree)| S {
            span: r.clone() | expr.clone() | f.clone() | binding.clone() | i.clone() | iteree.clone(),
            style:  r.clone() & expr.clone() & f.clone() & binding.clone() & i.clone() & iteree.clone(),
            inner: Statement::For(ast::For {
                name: binding,
                list: Box::new(iteree),
                body: Box::new(expr),
            }),
        });

    let while_loop = while_loop()
        .then(expr())
        .then(run())
        .then(expr())
        .map(|(((start, cond), run), body)| {
            S{
                span: start.clone() | body.clone(),
                style: start.clone() & cond.clone() & run.clone() & body.clone(),
                inner:Statement::While(ast::While{
                    cond: Box::new(cond),
                    body: Box::new(body),
                }),
            }
        });

    let proc = procedure()
        .then(ident())
        .then(takes())
        .then(delimited(ident(), Sigil::Comma))
        .then(does())
        .then(expr())
        .map(|(((((p, name), t), args), d), body)| S {
            span: args.iter().fold(p.clone() | name.clone() | t.clone(), |acc, a| acc | a.clone()) | d.clone() | body.clone(),
            style: args.iter().fold(p.clone() & name.clone() & t.clone(), |acc, a| acc & a.clone()) & d.clone() & body.clone(),
            inner: Statement::Procedure(ast::Procedure {
                name,
                args,
                body: Box::new(body),
            }),
        });

    proc.or(while_loop).or(for_loop).or(expr_statement)
}
