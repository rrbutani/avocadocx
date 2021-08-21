use docx_rs::{read_docx, read_zip, Docx};

use std::env::args;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::Path;

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
            $w
        )*
        $(
            #[cfg(not(target_arch = "wasm32"))]
            $o
        )*
    };
}

specialize! {
    wasm => {
        type Err = js_sys::JsValue;
    }
    other => {
        type Err = Box<dyn Error + Send + Sync + 'static>;
    }
}

fn from_file<P: AsRef<Path>>(path: P) -> Result<Docx, Err> {
    let inp = fs::read(path)?;
    Ok(read_docx(&inp)?)
}

async fn from_id(id: &str) -> Result<Docx, Err> {
    let url = format!(
        "https://docs.google.com/document/export?format=docx&id={}",
        id
    );

    let inp = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .bytes()
        .await?; // pls help
    Ok(read_docx(&inp)?)
}

#[cfg_attr(not(target_arch = "wasm32"), tokio::main)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)] // TODO
async fn main() -> Result<(), Err> {
    let input = args().skip(1).next().unwrap(); // TODO: don't grab from env, etc.
    let doc = if input.ends_with("docx") {
        from_file(&input)?
    } else {
        from_id(&input).await?
    };

    println!("{:#?}", doc);

    Ok(())
}
