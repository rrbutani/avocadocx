# Structure

  - avocadocx workspace
    + bin: "avocadocx"
  - abogado compiler
    - `abogado-common`
      + things like the `specialize` macro and maybe error reporting stuff
    - `abogado-lex`
      + can come from a plain text file or a docx
      + produces tokens with the metadata attached
    - `abogado-parse`
      + goes from the tokens in `lex` to the ast in `ast`
    - `abogado-ast`
      + just has the ast type and the visitor trait
    - `abogado-passes`
      + has a pass manager (you can add passes and they wrap, etc.; returns an impl PassPipeline)
      + should have passes like:
        * resolve deps
          - finds the imports; grabs em if they're not already there
            * not sure if this should be a pass
            * cycles should be allowed
        * chk namespaces (fonts)
          - text files (i.e. std) should have no font (no namespace) and can be called from anywhere
        * chk privacy (boxes)
  - avocadocx-std
    - just `include!("std.cado")` -> lex -> parse -> AST
  - avocadocx-runtime
  - avocadocx-interpreter
    + depends on `runtime`, just calls it
    + should take a `Writer` to write output into
  - avocadocx-codegen
    + links against `runtime`; includes it in the binaries it makes
    + this is *not* wasm compatible
  - avocadocx-web
    + main page (first):
      * single text box for the google doc id
      * support drag and drop for docx files like UTP does
      * an xterm-js-sys terminal on the page; redirect errors + the output of the intepreter there
      * don't bother with Elm or anything; just an html file that calls our function with the id of the input box and the id of the terminal
        - and we'll do the rest in Rust
    + extra page (later, never):
      * ACE editor box (Elm) w/syntax highlighting
      * an xterm-js-sys terminal
      * a run button
      * Elm drives the top level, etc.
  - `avocadocx` bin:
    + `run` command: takes a google doc ID or a file path (ending in docx or cado)
      * just calls the interpreter after producing the AST, error reporting, etc.
    + `compile` command
      * later, maybe not at all
      * also takes a google doc ID or a file path
      * produces a binary using `codegen`
    + `fetch` command:
      * takes a google doc ID, outputs to a file
    + `translate` command:
      * takes a google doc ID, converts to an AST, pretty prints

  - std:
    + `input`
      * terminal input on desktop, `input` box on web
    + `print`, `println`, etc.
      * terminal output on desktop, the integrated thing on the web

# Misc/Very Extra
  - [ ] need to make the logo
    + [ ] use it as the favicon
    + [ ] use it as the crate logo for all the crates
    + [ ] use it as the gh repo logo thing

  - [ ] (eventually or when bored) the usual CI things
    + [ ] rip the gh publish flow with parcel and friends for the webpage (or do it in bazel)
    + [ ] publish binaries on CI runs, etc.
    + [ ] crates.io, whatever

  - [ ] `ariadne`

  - [ ] spans for the tokens when lexing docx files can be actually correct? like, page 1, line 4 (etc.)
    + unsure if possible

  - [ ] syntax highlighting for `.cado`, the text form

  - [ ] fix the email address on commits?

# Questions
  - [ ] do we want to have a printable form for the extra attributes?
    + I say no; we should have the pretty printer have some way of printing them but I think we don't expose them in `.cado`
      * i.e. we don't extend the grammar to have textual constructs for them
