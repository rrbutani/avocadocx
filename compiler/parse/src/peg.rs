peg::parser!{
    grammar parser() for &[Token] {
        use ast::Spanned as S;
        use ast::Expr;
        use ast::Expr::*;
        use lex::Token::*;
        use lex::Sigil::*;
        use lex::Keyword::*;
        use lex::Op::*;

        rule op() -> Op

        pub rule expr() -> Expr = precedence!{
            a:(@) Op(Add) b:@ {Expr::BinOp(S(BinOp(Box::new(a), Add, Box::new(b)), a | b))}
            --
            a:(@) Op(Mul) b:@ {Expr::BinOp(S(BinOp(Box::new(a), Mul, Box::new(b)), a | b))}
            --
            a:$(Op(Sub)) b:@ {Expr::UnOp(S(UnOp(Sub, Box::new(a)), a | b))}
            --
            a:$(Do) name:$(Ident) b:$(Using) args:expr() ** Comma
                {Expr::Call(S(Call(name, args), a | (args.last().unwrap_or(b))))}
            StartBlock s:statement()* e:expr()? {Expr::Block(Block(s, e))}
            e:expr() b:$(Exclamation) {Expr::Print(S(Print(e), e | b))}
            cond:expr() Question e:expr() (Else other:expr())?
                {Expr::If(S(If(cond, e, other) cond | other))}
            f:$(Num) {Expr::Num(S(Num(f), f.span))}
            i:$(Ident) {Expr::Ident(Ident(i))}
        }

        rule while_loop() -> Statement {
            a:$(While) cond:expr() Keep e:expr()
                {Expr::Statement(S(Statement::While(cond, e), a | e))}
        }

        rule for_loop() -> Statement {
            a:$(Run) body:expr() For i:$(Ident) In e:expr()
                {Expr::Statement(S(Statement::For(body, i, e)), a | e)}
        }

        rule proc() -> Statement {
            a:$(Procedure) name:$(Ident) Takes args:$(Ident) ** Comma Does body:expr()
                {Statement::Procedure(S(Procedure(name, args, body)), a | body)}
        }

        pub rule statement() -> Statement {
            s:(
                (e:expr() p:$(Sigil(Dot) / Sigil(Semicolon))) {Statement::Expr(S(Expr(e), e | p))}
                / while_loop() / for_loop() / proc()
            ) {s}
        }
    }
}
