
Attrs that we'll grab:
  - italics, bold, underline
  - strikethrough (doesn't run)
  - color, background color
  - size
  - font (namespaces)
  - links (TODO; needs `read_document_rels`)
  - alignment (iteration order, list constants, arg lists)
  - border (privacy!)

Attrs/things that we'll ignore, probably:
  - images
  - containers (maybe?)
  - bullets, lists
  - indentation
  - margins
  - word art


# the grammar

```
keep doing (
    set the_end to never

    set never to False
) until the_end
```

```
# stdlib

do both a b
do either a b

set true to one
set untrue to not true

input
```

```
stmt:
  <expr> <punc>
  while <expr> run <expr>
  keep running <expr> until <expr>
  run <expr> for <ident> in <expr>
  procedure <ident> takes <list> does <expr>
  # todo: import _ from <link>

<punc> = ";.\n"

block:
  `(` stmt* (<expr>)? `)`

expr:
  is <expr> ? <expr> (otherwise <expr>)?
  set <ident> to <expr>
  <block>
  get <expr> from <expr>
  emit <expr> ! # prints
  do <ident> using <list>
  <expr> <binop> <expr> # +, -, *, /, =, >, <
  <unop> <expr> # not, -, neg
  <ident>
  <const>

const:
  <num>(st|nd|rd|th)?
  <english-numbers>: one | two | three | four | five | six | seven | eight | nine | ten | eleven | twelve | ...
  <str>
  [ <list> ]

str:
  ”.*”

§

list: (note: bullets should work too)
  <expr>
  <expr> as well as <expr>
  (<expr>,)+ <expr>, and <expr>
```

terminated with `,` or `;` or a newline
