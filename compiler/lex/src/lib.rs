#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/abogado-lex/0.0.0", // remember to bump!
)]

pub mod token;
pub mod style;
pub mod span;
pub mod spanned;

use docx_rs::{DocumentChild, Docx, Paragraph, ParagraphChild, Run, RunChild};
use thiserror::Error;

pub use token::{
    Token, Keyword, Sigil, Op,
};
pub use style::Style;
pub use span::Span;

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
            c if c.is_numeric() => {}
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

                use {Token::*, crate::Keyword::*, Op::*, crate::Sigil::*};
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
                    // "same" => Keyword(Same),
                    // "different" => Keyword(Different),
                    // "more" => Keyword(More),
                    // "less" => Keyword(Less),
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
