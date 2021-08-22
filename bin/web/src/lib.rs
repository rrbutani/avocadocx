#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]
#![cfg_attr(all(docs, not(doctest)), feature(external_doc))]
#![cfg_attr(all(docs, not(doctest)), doc(include_str!("../README.md")))]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(attr(deny(rust_2018_idioms, warnings), allow(unused_extern_crates))))]
#![doc(
    html_logo_url = "{{{ TODO }}}",
    html_root_url = "https://docs.rs/avocadocx-web/0.0.0", // remember to bump!
)]

use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use log::debug;

use xterm_js_sys::xterm::{LogLevel, Terminal, TerminalOptions, Theme};

#[wasm_bindgen]
pub async fn run(id: String) -> Result<String, JsValue> {
    set_panic_hook();
    console_log::init_with_level(log::Level::Trace);

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let terminal_div = document
        .get_element_by_id("terminal")
        .expect("should have a terminal div");

    let term_orig = Terminal::new(Some(
        TerminalOptions::default()
            .with_log_level(LogLevel::Debug)
            // .with_theme(Theme::nord())
            .with_font_family("'Fira Mono', monospace".to_string())
            .with_font_size(11.0),
    ));

    term_orig.open(terminal_div);
    let term = term_orig.clone();

    let file = abogado_common::get::from_google_docs(&*id).await.unwrap();
    let (tokens, file) = abogado_lex::lex_docx(&file);

    let tokens = tokens.unwrap();
    let tokens = tokens.into_iter().map(|t| { let s = t.span.clone(); (t, s) }).collect::<Vec<_>>();

    use abogado_parse::Parser as _;

    term.focus();

    term.write(String::from("\x1B[35;31m hello!\n"));
    term.write(String::from("\x1B[1;3;31mxterm.js\x1B[0m with 🦀\n$ "));


    debug!("{:#?}", tokens.iter().map(|(t, _s)| t).collect::<Vec<_>>());

    let program = abogado_parse::statement().repeated().parse(tokens).unwrap();

    for statement in program {
        debug!("{}", statement.inner);
        term.write(format!("{}\n", statement.inner));
    }

    Ok("ds".to_string())
}
