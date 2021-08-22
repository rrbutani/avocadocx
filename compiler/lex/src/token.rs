use docx_rs::{Justification, Paragraph, ParagraphProperty, ParagraphStyle, RunProperty};
use std::{convert::TryFrom, fmt::{self, Display, Write}, ops::{BitOr, Range}};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub span: Span,
    pub style: Style,
    pub inner: Inner,
}

impl BitOr for Token {
    type Output = Span;

    fn bitor(self, rhs: Token) -> Span {
        self.span | rhs.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub inner: Range<usize>,
    // TODO: add page number? (optional; for docx inputs)
}

impl Span {
    fn union(
        &self,
        Span {
            inner: Range { start, end },
        }: Span,
    ) -> Span {
        Span {
            inner: Range {
                start: self.inner.start.min(start),
                end: self.inner.end.max(end),
            },
        }
    }
}

impl BitOr for Span {
    type Output = Span;

    fn bitor(self, rhs: Span) -> Span { self.union(rhs) }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    pub prop: RunProperty,
    pub paragraph_style: Option<ParagraphStyle>,
    pub alignment: Option<Justification>,
}

impl Style {
    pub fn intersect(&self, other: &Style) -> Style {
        fn same_or_none<T: PartialEq + Clone>(a: &Option<T>, b: &Option<T>) -> Option<T> {
            if a == b {
                a.clone()
            } else {
                None
            }
        }

        macro_rules! prop {
            ($field:ident) => {
                same_or_none(&self.prop.$field, &other.prop.$field)
            };
        }

        Style {
            prop: RunProperty {
                sz: prop!(sz),
                sz_cs: prop!(sz_cs),
                color: prop!(color),
                highlight: prop!(highlight),
                vert_align: prop!(vert_align),
                underline: prop!(underline),
                bold: prop!(bold),
                bold_cs: prop!(bold_cs),
                italic: prop!(italic),
                italic_cs: prop!(italic_cs),
                vanish: prop!(vanish),
                spacing: prop!(spacing),
                fonts: prop!(fonts),
                text_border: prop!(text_border),
                del: prop!(del),
                ins: prop!(ins),
            },
            paragraph_style: match (&self.paragraph_style, &other.paragraph_style) {
                (sty @ Some(a), Some(b)) => {
                    if a == b {
                        sty.clone()
                    } else {
                        Some(ParagraphStyle {
                            val: "Normal".to_string(),
                        })
                    }
                }
                _ => None,
            },
            alignment: match (&self.alignment, &other.alignment) {
                (jus @ Some(a), Some(b)) if a == b => jus.clone(),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inner {
    Punc,
    StringConst(String),
    Num(f64),
    Ident(String),
    KeyWord(Keyword),
    Sigil(Sigil),
    Operator(Op),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Set,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    const SIGILS: &'static str = "[](),?!.;";
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    const OPERATORS: &'static str = "+-*/";
}

impl Display for Op {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {

    }
}

impl TryFrom<&str> for Op {
    type Error = ();
}
