#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/abogado-lex/0.0.0", // remember to bump!
)]

pub mod span;
pub mod spanned;
pub mod style;
pub mod token;

use std::{convert::TryInto, num::ParseFloatError};

use docx_rs::{DocumentChild, Docx, ParagraphChild, RunChild};
use thiserror::Error;

pub use span::Span;
pub use style::Style;
pub use token::{Keyword, Op, Sigil, Token};

type S = spanned::S<Token>;

#[derive(Debug, Clone)]
struct Splatted {
    tagged_chars: Vec<(char, usize)>,
    styles: Vec<Style>,
}

fn splat_docx(doc: &Docx) -> Splatted {
    let doc = &doc.document;

    let mut tagged_chars = vec![];
    let mut styles = vec![Style::default()];

    for doc_child in doc.children.iter() {
        // TODO: tables, comments
        if let DocumentChild::Paragraph(p) = doc_child {
            let prop = &p.property;
            let paragraph_style = &prop.style;
            let alignment = &prop.alignment;

            for par_child in p.children.iter() {
                // TODO: handle comments here too maybe?
                if let ParagraphChild::Run(r) = par_child {
                    let run_prop = &r.run_property;

                    let style_for_run = Style {
                        prop: run_prop.clone(),
                        paragraph_style: paragraph_style.clone(),
                        alignment: alignment.clone(),
                    };
                    if &style_for_run != styles.last().unwrap() {
                        styles.push(style_for_run);
                    }
                    let style_id = styles.len() - 1;

                    for run_child in r.children.iter() {
                        // TODO: handle comments here too maybe?
                        if let RunChild::Text(t) = run_child {
                            tagged_chars.extend(t.text.chars().map(|c| (c, style_id)));
                            // println!("    text: {}\n\n", t.text);
                        }
                    }
                }
            }
        }
    }

    Splatted {
        tagged_chars,
        styles,
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LexError {
    #[error("unexpected end of file; was expecting a {looking_for:?}")]
    UnexpectedEof { looking_for: Option<String> },
    #[error("unexpected closing quote")]
    UnexpectedClosingQuote,
    #[error("float parse error: {0}")]
    FloatParseError(#[from] ParseFloatError),
}
// TODO: span these ^

fn collate(
    Splatted {
        tagged_chars,
        mut styles,
    }: Splatted,
) -> Result<Vec<S>, LexError> {
    let mut tokens = vec![];

    let char_iter = tagged_chars.iter().scan(0, |byte_offset, (c, tag)| {
        let starting_offset = *byte_offset;
        *byte_offset += c.len_utf16();
        let ending_offset = *byte_offset;

        Some(((starting_offset, ending_offset), *c, *tag))
    });

    let mut char_iter = char_iter.peekable();
    // states: ready, in_num, in_str, in_word
    while let Some(((start_ofs, end_ofs), c, style_id)) = char_iter.next() {
        const OPEN_QUOTE: char = '“';
        const CLOSE_QUOTE: char = '”';
        const WHITESPACE: &str = " \n\r\t";

        let fold_style = |styles: &mut Vec<Style>, existing: &mut usize, added: usize| {
            if *existing == added {
                // we're good; the styles match
            } else if styles[*existing] == styles[added] {
                // we're good; the styles match
                //
                // we'll update the index since it's what the next tokens may have
                *existing = added;
            } else {
                // if the styles are different, find their intersection:
                let new = styles[*existing].intersect(&styles[added]);
                styles.push(new);
                *existing = styles.len() - 1;
            }
        };

        match c {
            '"' => {
                // Just eat chars until we see another `"`:
                let start_offset = start_ofs;
                let mut token_style_id = style_id;
                let mut string = String::new();
                let end_offset = loop {
                    let ((_, end_ofs), c, style_id) =
                        char_iter.next().ok_or(LexError::UnexpectedEof {
                            looking_for: Some(String::from('"')),
                        })?;

                    fold_style(&mut styles, &mut token_style_id, style_id);

                    if c == '"' {
                        break end_ofs;
                    }

                    string.push(c);
                };

                tokens.push(S {
                    inner: Token::StringConst(string),
                    span: Span {
                        inner: start_offset..end_offset,
                    },
                    style: styles[token_style_id].clone(),
                })
            }
            OPEN_QUOTE => {
                // these can be nested so we keep track of our depth:
                let mut quote_depth = 1;
                let start_offset = start_ofs;
                let mut token_style_id = style_id;
                let mut string = String::new();
                let end_offset = loop {
                    let ((_, end_ofs), c, style_id) =
                        char_iter.next().ok_or(LexError::UnexpectedEof {
                            looking_for: Some(String::from(CLOSE_QUOTE)),
                        })?;

                    match c {
                        OPEN_QUOTE => quote_depth += 1,
                        CLOSE_QUOTE => quote_depth -= 1,
                        _ => {}
                    }

                    fold_style(&mut styles, &mut token_style_id, style_id);

                    if quote_depth == 0 {
                        break end_ofs;
                    }

                    // Note that we do this last; we don't push the closing
                    // quote to the string.
                    string.push(c);
                };

                tokens.push(S {
                    inner: Token::StringConst(string),
                    span: Span {
                        inner: start_offset..end_offset,
                    },
                    style: styles[token_style_id].clone(),
                })
            }
            c if WHITESPACE.contains(c) => {
                // do nothing for white space
            }
            c if (c == '.'
                && char_iter
                    .peek()
                    .filter(|(_, c, _)| c.is_numeric())
                    .is_some())
                || c.is_numeric() =>
            {
                // we've got ourselves a number!

                // we'll eat characters while we're getting numbers and have seen up to 1 dot:
                let mut seen_a_dot = false;
                let starting_offset = start_ofs;
                let mut ending_ofs = start_ofs;
                let mut num = String::from(c);
                let mut token_style_id = style_id;

                let mut extra_dot_token = None;

                while let Some(((_start, _end_pos), c, _style_id)) = char_iter.peek() {
                    if !(c.is_numeric() || *c == '.') {
                        break;
                    }

                    let ((start, end_pos), c, style_id) = if let Some(x) = char_iter.next() {
                        x
                    } else {
                        unreachable!();
                    };

                    // if this is our second dot or if it's not followed by
                    // a numeric char, it's not part
                    // of the number:
                    if c == '.'
                        && (seen_a_dot
                            || char_iter
                                .peek()
                                .filter(|(_, c, _)| c.is_numeric())
                                .is_none())
                    {
                        // since we've consumed it we've got to add it to the tokens
                        extra_dot_token = Some(S {
                            inner: Token::Sigil(Sigil::Dot),
                            span: Span {
                                inner: start..end_pos,
                            },
                            style: styles[style_id].clone(),
                        });
                        break;
                    } else if c == '.' {
                        seen_a_dot = true;
                    }

                    num.push(c);
                    ending_ofs = end_pos;

                    fold_style(&mut styles, &mut token_style_id, style_id);
                }

                let num = num.parse()?;
                tokens.push(S {
                    inner: Token::Num(num),
                    span: Span {
                        inner: starting_offset..ending_ofs,
                    },
                    style: styles[token_style_id].clone(),
                });

                // if the last char is a `.` it's not actually part of the number; see above
                if let Some(extra) = extra_dot_token {
                    tokens.push(extra);
                }
            }
            c if TryInto::<Sigil>::try_into(&*String::from(c)).is_ok() => {
                tokens.push(S {
                    span: Span { inner: start_ofs..end_ofs },
                    style: styles[style_id].clone(),
                    inner: Token::Sigil(TryInto::<Sigil>::try_into(&*String::from(c)).unwrap()),
                })
            },
            c if TryInto::<Op>::try_into(&*String::from(c)).is_ok() => {
                tokens.push(S {
                    span: Span { inner: start_ofs..end_ofs },
                    style: styles[style_id].clone(),
                    inner: Token::Operator(TryInto::<Op>::try_into(&*String::from(c)).unwrap()),
                })
            },
            c => {
                let mut word = String::from(c);
                let mut end = end_ofs;
                let mut token_style_id = style_id;

                loop {
                    // We don't want to consume this if it's a terminator!
                    let ((_start_ofs, end_ofs), c, style_id) =
                        char_iter.peek().unwrap_or(&((0, 0), '.', 0));
                    if WHITESPACE.contains(*c)
                        || token::Sigil::SIGILS.contains(*c)
                        || token::Op::OPERATORS.contains(*c)
                    {
                        break;
                    }

                    end = *end_ofs;

                    fold_style(&mut styles, &mut token_style_id, *style_id);
                    word.push(*c);

                    // But once we know it's not a terminator we can advance the
                    // iterator; we have used this character.
                    let _ = char_iter.next();
                }

                use {crate::Keyword::*, crate::Sigil::*, Op::*, Token::*};
                let inner = match &*word {
                    "+" => Operator(Add),
                    "-" => Operator(Sub),
                    "*" => Operator(Mul),
                    "/" => Operator(Div),

                    "[" => Sigil(StartList),
                    "," => Sigil(Comma),
                    "]" => Sigil(EndList),
                    "(" => Sigil(StartBlock),
                    ")" => Sigil(EndBlock),
                    "?" => Sigil(Question),
                    "!" => Sigil(Exclamation),
                    "." => Sigil(Dot),
                    ";" => Sigil(Semicolon),

                    "set" => Keyword(Set),
                    "to" => Keyword(To),
                    "is" => Keyword(Is),
                    "do" => Keyword(Do),
                    "using" => Keyword(Using),
                    "get" => Keyword(Get),
                    "also" => Keyword(Also),
                    "and" => Keyword(And),
                    "not" => Keyword(Not),
                    "while" => Keyword(While),
                    "keep" => Keyword(Keep),
                    "until" => Keyword(Until),
                    "run" => Keyword(Run),
                    "for" => Keyword(For),
                    "in" => Keyword(In),
                    "procedure" => Keyword(Procedure),
                    "takes" => Keyword(Takes),
                    "does" => Keyword(Does),
                    "otherwise" | "else" => Keyword(Else),
                    "emit" => Keyword(Emit),
                    "from" => Keyword(From),

                    _ => Ident(word),
                };

                tokens.push(S {
                    span: Span {
                        inner: start_ofs..end,
                    },
                    style: styles[token_style_id].clone(),
                    inner,
                });
            }
        }
    }

    Ok(tokens)
}

pub fn lex_docx(doc: &Docx) -> (Result<Vec<S>, LexError>, String) {
    let splatted = splat_docx(doc);
    let full_string = splatted.tagged_chars.iter().map(|p| p.0).collect();

    (collate(splatted), full_string)
}

pub fn lex_cado(inp: String) -> (Result<Vec<S>, LexError>, String) {
    let splatted = Splatted {
        tagged_chars: inp.chars().map(|c| (c, 0)).collect(),
        styles: vec![Default::default()],
    };

    (collate(splatted), inp)
}
