#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/abogado-common/0.0.0", // remember to bump!
)]

#[macro_export]
macro_rules! specialize {
    (
        wasm => {
            $($w:item)*
        }
        other => {
            $($o:item)*
        }
    ) => {
        $(
            #[cfg(target_arch = "wasm32")]
            #[cfg_attr(all(docs, not(doctest)), doc(cfg(target_arch = "wasm32")))]
            $w
        )*
        $(
            #[cfg(not(target_arch = "wasm32"))]
            #[cfg_attr(all(docs, not(doctest)), doc(cfg(not(target_arch = "wasm32"))))]
            $o
        )*
    };
}

#[macro_export]
macro_rules! wasm {
    ($($i:item)*) => {
        $crate::specialize! {
            wasm => {
                $($i)*
            }
            other => {}
        }
    };
}

#[macro_export]
macro_rules! not_wasm {
    ($($i:item)*) => {
        $crate::specialize! {
            wasm => {}
            other => {
                $($i)*
            }
        }
    };
}

pub use docx_rs::Docx;

/// Helpers to grab and parse a [`Docx`](docx_rs::Docx) instance.
pub mod get {
    use std::path::Path;

    use docx_rs::{read_docx, Docx, ReaderError};
    use reqwest::{Client, Error as ReqError};
    use thiserror::Error;

    not_wasm! {
        /// Errors that can happen when grabbing a [`Docx`] from a file.
        #[derive(Error, Debug)]
        pub enum SourceFromFileError {
            #[error("failed to load file: `{0}`")]
            LoadFailed(#[from] std::io::Error),
            #[error(transparent)]
            FailedToParse(#[from] ReaderError),
        }

        /// Grabs a [`Docx`] from a file path.
        pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Docx, SourceFromFileError> {
            let inp = std::fs::read(path)?;
            Ok(read_docx(&inp)?)
        }
    }

    /// Errors that can happen when grabbing a [`Docx`] from a Google Docs.
    #[derive(Error, Debug)]
    pub enum SourceFromGoogleDocsError {
        #[error("failed to fetch: `{0}`")]
        FetchFailed(#[from] ReqError),
        #[error(transparent)]
        FailedToParse(#[from] ReaderError),
    }

    /// Grabs a [`Docx`] from Google Docs.
    pub async fn from_google_docs(id: &str) -> Result<Docx, SourceFromGoogleDocsError> {
        let url = format!(
            "https://docs.google.com/document/export?format=docx&id={}",
            id
        );

        let inp = Client::new()
            .get(url)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(read_docx(&inp)?)
    }
}
