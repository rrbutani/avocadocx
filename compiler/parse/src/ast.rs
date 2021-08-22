use std::ops::{Deref, DerefMut};

use abogado_lex::token::Token;
use Token::*;


#[derive(Debug)]
pub struct Spanned<T>(pub T, pub Span);

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

type S<T> = Spanned<T>;

#[derive(Debug)]
pub enum Statement {
    Expr(S<Expr>),
    While(S<While>),
    // Until(Expr, Expr),
    For(S<For>),
    Procedure(S<Procedure>),
}

#[derive(Debug)]
pub enum Expr {
    Assign(S<Assign>),
    Block(S<Block>),
    Print(S<Box<Expr>>),
    If(S<If>),
    Call(S<Call>),
    BinOp(S<BinOp>),
    UnOp(S<UnOp>),// UnOp(S<UnOp>),
    Num(S<f64>),
    Ident(Ident),
}



#[derive(Debug)]
struct Ident(String, Span);
#[derive(Debug)]
pub struct While { cond: Expr, body: Expr}
#[derive(Debug)]
pub struct For{name: Ident, list: Expr, body: Expr}
#[derive(Debug)]
pub struct Procedure{name: Ident, args: Vec<Ident>, body: Expr}
#[derive(Debug)]
pub struct Assign{name: Ident, to: Box<Expr>}
#[derive(Debug)]
pub struct Block{body: Vec<Statement>, end: Option<Expr>}
#[derive(Debug)]
pub struct If{cond: Box<Expr>, then: Box<Expr>, otherwise: Option<Expr>}
#[derive(Debug)]
pub struct Call{name: Ident, args: Vec<Expr>}
#[derive(Debug)]
pub struct BinOp{lhs: Box<Expr>, op: Op, rhs: Box<Expr>}
#[derive(Debug)]
pub struct UnOp{op: Op, expr: Box<Expr>}
