#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/avocadocx-interpreter/0.0.0", // remember to bump!
)]

use abogado_parse::ast::*;
use abogado_lex::token::Op::*;
use std::collections::HashMap;

enum Value {
    Num(f64),
    String(String),
    List(Vec<Value>),
}

impl Value {
    pub fn falsey(&self) -> bool {
        use Self::*;
        match self {
            Num(f) => f.partial_eq(0.0),
            String(s) => s.is_empty,
            List(l) => l.is_empty() || l.all(|v| v.falsey()),
        }
    }
    pub fn truthy(&self) -> bool {
        !self.falsey()
    }
}

#[derive(Default, Debug)]
struct Namespace {
    inner: HashMap<String, Value>,
    parent: Option<Box<Namespace>>,
}

impl Namespace{
    pub fn resolve(s: &str) -> Option<Value> {
        inner.get(s).or_else(|| parent.map(|p| parent.resolve()))
    }
}

#[derive(Default)]
struct Context {
    namespace: Namespace,
    functions: HashMap<String, (Vec<String>, Expr)>,
}

fn run_expr(ctx: &mut Context, e: Expr) -> Result<Value, ()> {
    match e{
        Expr::Num(f) => Value::Num(f.inner),
        Expr::String(s) => Value::String(s.inner),
        Expr::List(l) => Value::List(l.inner.map(|e|run_expr(ctx, e)?).collect()),
        Expr::Ident(i) => ctx.namespace.resolve(i),
        Expr::Assign(Assign{name, to}) => { let val = run_expr(ctx, to.inner)?; ctx.namespace.insert(name.inner, val); val},
        Expr::Print(e) => { let val = run_expr(ctx, e.inner)?; println!("{:?}", val); val},
        Expr::Block(b) => todo!(),
        Expr::UnOp(UnOp{op, e}) => {
            let val = run_expr(ctx, e.inner)?;
            match op.inner{
                Sub => match val{
                    if let Value::Num(n) = val {-val} else{ Err((todo!("gotto index with ints"))) }
                },
                Not => !(val.truthy()),
                _ => unreachable!();
            }
        },
        Expr::UnOp(BinOp{lhs, op, rhs}) => {
            let lhs = run_expr(ctx, lhs.inner)?;
            let rhs = run_expr(ctx, rhs.inner)?;
            match op.inner {
                Add => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {a + b},
                    (Value::List(l), b) => {l.append(b)},
                },
                Sub => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {a - b},
                },
                Mul => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {a * b},
                },
                Div => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {a / b},
                },
                _ => todo!("other operator permutations");
            }
        },
        Expr::Call(Call{name, args}) => {let (params, body) = ctx.functions.get(name).or_else(|| todo!("unknown func"));
            todo!("zip args with params, insert into namespace as new scope, and execute (pop scope when done)")
        },
        Expr::Get(Get{list, index}) => {
            let list = run_expr(ctx, list)?;
            let index = run_expr(ctx, list)?;
            match list {
                Value::List(l) => {
                    if let Value::Num(n) = index { l.get(n).unwrap_or_else(|| todo!("OOB error"))}
                    else{
                        Err((todo!("gotto index with ints")))
                    }
                },
                Value::String(_s) => todo!("return substring of single char")
                _ => return todo!("return error for non lists"),
            }
        }
    }
}

fn run_statement(ctx: &mut Context, s: Statement) -> Result<(), ()> {
    match s {
        Statement::Expr(e) => run_expr(ctx, e),
        Statement::While(While { cond, body }) => {
            while run_expr(ctx, cond)?.falsey() {
                run_expr(ctx, body)?;
            }
        }
        Statement::Procedure(Procedure{name, args, body}) => {
            ctx.functions.insert(name.inner, (args.inner, body.inner)
        },
        Statement::For(name, list, body) => {
            let list = run_expr(ctx, list)?;
            match list {
                Value::List(l) => { todo!("bind name and execute body with updated namespace")},
                Value::String(_s) => todo!("iterate over substring of single char")
                _ => return todo!("return error for non lists"),
            }
        },
    }
    Ok(())
}

pub fn run_program(program: Vec<Statement>) -> Result<(), ()> {
    let mut context = Context::Default();

    for statement in program {
        run_statement(&mut context, statement)?;
    }

    Ok(())
}
