use std::fmt::{self, Display};

use abogado_lex::{spanned::S, Op};

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(S<Expr>),
    While(While),
    // Until(Expr, Expr),
    For(For),
    Procedure(Procedure),
}

impl Display for Statement {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Statement::*;

        match self {
            Expr(e) => write!(fmt, "{}", e.inner),
            While(w) => write!(fmt, "{}", w),
            For(f) => write!(fmt, "{}", f),
            Procedure(p) => write!(fmt, "{}", p),
        }?;

        write!(fmt, ";")
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    If(If),
    BinOp(BinOp),

    UnOp(UnOp),
    Call(Call),

    Num(f64),
    String(String),
    Ident(Ident),
    List(List),
    Assign(Assign),

    Print(S<Box<Expr>>),
    Get(Get),
    Block(Block),
    //TODO: is op
}

impl Display for Expr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;

        match self {
            Assign(a) => write!(fmt, "{}", a),
            Block(b) => write!(fmt, "{}", b),
            Print(p) => write!(fmt, "({})!", p.inner),
            If(i) => write!(fmt, "{}", i),
            Call(c) => write!(fmt, "{}", c),
            BinOp(b) => write!(fmt, "{}", b),
            UnOp(u) => write!(fmt, "{}", u),
            Num(n) => write!(fmt, "{}", n),
            String(s) => write!(fmt, "\"{}\"", s),
            Ident(i) => write!(fmt, "{}", i),
            List(l) => write!(fmt, "{}", l),
            Get(g) => write!(fmt, "{}", g),
        }
    }
}

#[derive(Debug, Clone)]
pub struct List(pub Vec<S<Expr>>);
impl Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|a| a.inner.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

pub type Ident = String;

#[derive(Debug, Clone)]
pub struct While {
    pub cond: Box<S<Expr>>,
    pub body: Box<S<Expr>>,
}
impl Display for While {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while {} do {}", self.cond.inner, self.body.inner)
    }
}
#[derive(Debug, Clone)]
pub struct For {
    pub name: S<Ident>,
    pub list: Box<S<Expr>>,
    pub body: Box<S<Expr>>,
}
impl Display for For {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "for {} in {} do {}",
            self.name.inner, self.list.inner, self.body.inner
        )
    }
}
#[derive(Debug, Clone)]
pub struct Procedure {
    pub name: S<Ident>,
    pub args: Vec<S<Ident>>,
    pub body: Box<S<Expr>>,
}
impl Display for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "function {} ({}) {}",
            self.name.inner,
            self.args
                .iter()
                .map(|a| a.inner.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.body.inner
        )
    }
}
#[derive(Debug, Clone)]
pub struct Assign {
    pub name: S<Ident>,
    pub to: Box<S<Expr>>,
}
impl Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name.inner, self.to.inner)
    }
}

#[derive(Debug, Clone)]
pub struct Get {
    pub index: Box<S<Expr>>,
    pub from: Box<S<Expr>>,
}

impl Display for Get {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})[{}]", self.from.inner, self.index.inner)
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub body: Vec<S<Statement>>,
    pub end: Option<Box<S<Expr>>>,
}
impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        for s in self.body.iter() {
            write!(f, "    ")?;
            write!(f, "{}", s.inner)?;
            writeln!(f)?;
        }

        if let Some(ref last) = self.end {
            write!(f, "    ")?;
            write!(f, "{}", last.inner)?;
            writeln!(f)?;
        }

        write!(f, "}}")
    }
}
#[derive(Debug, Clone)]
pub struct If {
    pub cond: Box<S<Expr>>,
    pub then: Box<S<Expr>>,
    pub otherwise: Option<Box<S<Expr>>>,
}
impl Display for If {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if {} then {}", self.cond.inner, self.then.inner)?;
        if let Some(ref e) = self.otherwise {
            write!(f, " otherwise {}", e.inner)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Call {
    pub name: S<Ident>,
    pub args: Vec<S<Expr>>,
}
impl Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name.inner,
            self.args
                .iter()
                .map(|a| a.inner.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
#[derive(Debug, Clone)]
pub struct BinOp {
    pub lhs: Box<S<Expr>>,
    pub op: S<Op>,
    pub rhs: Box<S<Expr>>,
}
impl Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.lhs.inner, self.op.inner, self.rhs.inner
        )
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Neg => write!(f, "-"),
            UnaryOperator::Not => write!(f, "NOT "),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnOp {
    pub op: S<UnaryOperator>,
    pub expr: Box<S<Expr>>,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}{})", self.op.inner, self.expr.inner)
    }
}
