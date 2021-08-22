#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/avocadocx-interpreter/0.0.0", // remember to bump!
)]

// use abogado_parse::ast::*;
use abogado_lex::token::Op::*;
use abogado_parse::ast::{
    Assign, BinOp, Block, Call, Expr, For, Get, If, List, Procedure, Statement, UnOp, While, UnaryOperator,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Num(f64),
    String(String),
    List(Vec<Value>),
}

impl Value {
    pub fn falsey(&self) -> bool {
        match self {
            Value::Num(f) => (*f).eq(&0.0),
            Value::String(s) => s.is_empty(),
            Value::List(l) => l.is_empty() || l.iter().all(|v| v.falsey()),
        }
    }
    pub fn truthy(&self) -> bool {
        !self.falsey()
    }
}

#[derive(Default, Debug)]
struct Namespace {
    inner: HashMap<String, Value>,
    child: Option<Box<Namespace>>,
}

impl Namespace {
    pub fn resolve(&self, s: &str) -> Option<Value> {
        self.child
            .as_ref()
            .map(|c| c.resolve(s))
            .flatten()
            .or_else(|| self.inner.get(s).cloned())
    }

    pub fn push(&mut self, new: HashMap<String, Value>) {
        // if let Some(mut c) = &self.child {
        //     c.push(new)
        // } else {
        //     self.child = Some(Box::new(Namespace {
        //         inner: new,
        //         child: None,
        //     }))
        // }
    }

    pub fn pop(&mut self) {
        // if let Some(mut c) = &self.child {
        //     c.pop()
        // } else {
        //     self.child = None
        // }
    }
}

#[derive(Default)]
struct Context {
    namespace: Namespace,
    functions: HashMap<String, (Vec<String>, Expr)>,
}

fn run_expr(ctx: &mut Context, e: Expr) -> Result<Value, ()> {
    let res = match e {
        Expr::Num(f) => Value::Num(f),
        Expr::String(s) => Value::String(s),
        Expr::List(List(l)) => Value::List(
            l.into_iter()
                .map(|e| run_expr(ctx, e.inner.clone()).expect("todo"))
                .collect(),
        ),
        Expr::Ident(i) => ctx
            .namespace
            .resolve(&i)
            .expect("todo, err for unknown var"),
        Expr::Assign(Assign { name, to }) => {
            let val = run_expr(ctx, to.inner)?;
            ctx.namespace.inner.insert(name.inner, val.clone());
            val
        }
        Expr::Print(e) => {
            let val = run_expr(ctx, (*e.inner).clone())?;
            println!("{:?}", val);
            val
        }
        Expr::Block(Block { body, end }) => todo!(),
        Expr::If(If {
            cond,
            then,
            otherwise,
        }) => {
            if run_expr(ctx, cond.inner)?.truthy() {
                run_expr(ctx, then.inner)?
            } else {
                if let Some(o) = otherwise {
                    run_expr(ctx, o.inner)?
                } else {
                    Value::Num(0.0)
                }
            }
        }
        Expr::UnOp(UnOp { op, expr }) => {
            let val = run_expr(ctx, expr.inner)?;
            Value::Num(match op.inner {
                UnaryOperator::Neg => {
                    if let Value::Num(n) = val {
                        -n
                    } else {
                        todo!("gotto index with ints")
                    }
                }
                UnaryOperator::Not => (!val.truthy()) as i32 as f64,
            })
        }
        Expr::BinOp(BinOp { lhs, op, rhs }) => {
            let lhs = run_expr(ctx, lhs.inner)?;
            let rhs = run_expr(ctx, rhs.inner)?;
            match op.inner {
                Add => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => Value::Num(a + b),
                    (Value::List(mut l), b) => {
                        l.push(b);
                        Value::List(l)
                    }
                    _ => todo!("invalid combo"),
                },
                Sub => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => Value::Num(a - b),
                    _ => todo!("invalid combo"),
                },
                Mul => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => Value::Num(a * b),
                    _ => todo!("invalid combo"),
                },
                Div => match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => Value::Num(a / b),
                    _ => todo!("invalid combo"),
                },
                _ => todo!("other operator permutations"),
            }
        }
        Expr::Call(Call { name, args }) => {
            let (params, body) = ctx
                .functions
                .get(&name.inner)
                .unwrap_or_else(|| todo!("unknown func"));
            let body = body.clone();
            let params = params.clone();
            let args = args
                .into_iter()
                .map(|a| run_expr(ctx, a.inner).unwrap_or_else(|_| todo!("rip")));
            let bindings = params.into_iter().zip(args.into_iter()).collect();
            ctx.namespace.push(bindings);
            let retval = run_expr(ctx, body)?;
            ctx.namespace.pop();
            retval
        }
        Expr::Get(Get { index, from }) => {
            let from = run_expr(ctx, from.inner)?;
            let index = run_expr(ctx, index.inner)?;
            match from {
                Value::List(l) => {
                    if let Value::Num(n) = index {
                        l.get(n as usize)
                            .unwrap_or_else(|| todo!("out of bounds"))
                            .clone()
                    }
                    // TODO: oob error
                    else {
                        todo!("gotto index with ints")
                    }
                }
                Value::String(_s) => todo!("return substring of single char"),
                _ => todo!("return error for non lists"),
            }
        }
    };
    Ok(res)
}

fn run_statement(ctx: &mut Context, s: Statement) -> Result<(), ()> {
    match s {
        Statement::Expr(e) => {
            run_expr(ctx, e.inner)?;
        }
        Statement::While(While { cond, body }) => {
            while run_expr(ctx, cond.inner.clone())?.falsey() {
                run_expr(ctx, body.inner.clone())?;
            }
        }
        Statement::Procedure(Procedure { name, args, body }) => {
            let args = args.into_iter().map(|a| a.inner).collect::<Vec<_>>();
            ctx.functions.insert(name.inner, (args, body.inner));
        }
        Statement::For(For { name, list, body }) => {
            let list = run_expr(ctx, list.inner)?;
            match list {
                Value::List(l) => {
                    todo!("bind name and execute body with updated namespace")
                }
                Value::String(_s) => todo!("iterate over substring of single char"),
                _ => todo!("return error for non lists"),
            };
        }
    };
    Ok(())
}

pub fn run_program(program: Vec<Statement>) -> Result<(), ()> {
    let mut context = Context::default();

    for statement in program {
        run_statement(&mut context, statement)?;
    }

    Ok(())
}
