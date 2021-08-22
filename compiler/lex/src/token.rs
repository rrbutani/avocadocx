use std::{convert::TryFrom, fmt::{self, Display, Write}, ops::{BitOr, BitAnd, Range}};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Punc,
    StringConst(String),
    Num(f64),
    Ident(String),
    Keyword(Keyword),
    Sigil(Sigil),
    Operator(Op),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Set,
    To,
    Is,
    Do,
    Using,
    Get,
    Same,
    Different,
    More,
    Less,
    Also,
    And,
    Not,
    While,
    Keep,
    Until,
    Run,
    For,
    In,
    Procedure,
    Takes,
    Does,
    Else,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sigil {
    StartList = 0,
    EndList = 1,
    StartBlock = 2,
    EndBlock = 3,
    Comma = 4,
    Question = 5,
    Exclamation = 6,
    Dot = 7,
    Semicolon = 8,
}

impl Sigil {
    pub const SIGILS: &'static str = "[](),?!.;";
}

impl Display for Sigil {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", Sigil::SIGILS.as_bytes()[*self as usize] as char)
    }
}

impl TryFrom<&str> for Sigil {
    type Error = ();

    fn try_from(c: &str) -> Result<Self, ()> {
        use Sigil::*;

        Ok(match c {
            "[" => StartList,
            "]" => EndList,
            "(" => StartBlock,
            ")" => EndBlock,
            "," => Comma,
            "?" => Question,
            "!" => Exclamation,
            "." => Dot,
            ";" => Semicolon,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
}

impl Op {
    pub const OPERATORS: &'static str = "+-*/";
}

impl Display for Op {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", Self::OPERATORS.as_bytes()[*self as usize] as char)
    }
}

impl TryFrom<&str> for Op {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "/" => Self::Div,
            _ => return Err(()),
        })
    }
}

// TODO: use `string_enum` for ^
