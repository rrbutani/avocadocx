#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/avocadocx/0.0.0", // remember to bump!
)]

use std::str::FromStr;
use std::{io, fs};
use std::path::Path;

use structopt::StructOpt;

use abogado_common::{Docx, get::{self, SourceFromFileError, SourceFromGoogleDocsError}};

// TODO: maybe have a driver crate?

#[derive(Debug)]
enum AvocadoxInput {
    // A `.cado` (plain-text) source.
    CadoSource { fname: String, contents: String },
    // Can come from a file or a URL.
    //
    // A `.docx` source.
    DocxSource {
        // The file name if the source is a file, the doc name if the source is
        // a URL.
        name: String,
        doc: Docx,
    },
}

impl AvocadoxInput {
    fn from_cado_file<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        Ok(AvocadoxInput::CadoSource {
            fname: path.as_ref().file_name().unwrap().to_str().unwrap().to_string(),
            contents: fs::read_to_string(path)?,
        })
    }

    async fn from_google_docs(id: &str) -> Result<Self, SourceFromGoogleDocsError> {
        get::from_google_docs(id).await.map(|doc| AvocadoxInput::DocxSource {
            name: "<main module>".to_string(), // TODO: fetch the document name!
            doc,
        })
    }

    fn from_docx_file<P: AsRef<Path>>(path: P) -> Result<Self, SourceFromFileError> {
        get::from_file(path.as_ref()).map(|doc| AvocadoxInput::DocxSource {
            name: path.as_ref().file_name().unwrap().to_str().unwrap().to_string(),
            doc,
        })
    }
}

#[derive(Debug)]
enum Input {
    CadoFile(String),
    DocxFile(String),
    GoogleDocId(String),
}

impl FromStr for Input {
    type Err = String;

    // TODO: use a reporting error type
    fn from_str(s: &str) -> Result<Self, String> {
        use Input::*;

        let res = match s {
            _ if s.ends_with(".cado") => CadoFile(s.to_string()),
            _ if s.ends_with(".docx") => DocxFile(s.to_string()),
            id if id.len() == 44 => GoogleDocId(id.to_string()),
            _ => return Err(
                format!("invalid input source: {}; must be a Google Sheet Id or a file path", s)
            )
        };

        Ok(res)
    }
}

#[derive(Debug, StructOpt)]
enum Args {
    /// Run an `avocadocx` program.
    Run {
        /// Input program file.
        ///
        /// Can be a file (.cado or .docx) or a Google Doc Id.
        input: Input,
    },
    Compile { },
    Fetch { },
    Translate { },
}


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::from_args();

    let source = match args {
        Args::Run  { input } => input,
        _ => todo!(),
    };

    let inp = match source {
        Input::CadoFile(f) => AvocadoxInput::from_cado_file(f)?,
        Input::DocxFile(f) => AvocadoxInput::from_docx_file(f)?,
        Input::GoogleDocId(id) => AvocadoxInput::from_google_docs(&id).await?,

    };

    let (name, (tokens, string)) = match inp {
        AvocadoxInput::DocxSource {
            name, doc
        } => (name, abogado_lex::lex_docx(&doc)),
        AvocadoxInput::CadoSource { fname, contents } => (fname, abogado_lex::lex_cado(contents)),
    };

    let tokens = tokens?;

    println!("{:#?}", tokens);

    let exp = abogado_parse::parser::expr();

    println!("{:#?}", expr);

    Ok(())
}
