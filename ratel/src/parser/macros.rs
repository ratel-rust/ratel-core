/// If the next token matches `$p`, consume that token and execute `$eval`.
#[macro_export]
macro_rules! allow {
    ($parser:ident, $p:pat => $eval:expr) => {
        match $parser.lexer.token {
            $p => {
                $parser.lexer.consume();
                $eval;
            },
            _ => {}
        }
    }
}

/// Return an error if the next token doesn't match $p.
#[macro_export]
macro_rules! expect {
    ($parser:ident, $p:pat) => {
        match $parser.lexer.token {
            $p => $parser.lexer.consume(),
            _  => $parser.error()
        }
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_expr {
    ($src:expr, $expr:expr) => ({
        let mut body = parse($src).unwrap().body().iter();

        match body.next().map(|s| s.item).unwrap() {
            Statement::Expression(ref expression) => assert_eq!(expression.item, Expression::from($expr)),
            _ => panic!("Statement isn't an expression!")
        }

        assert_eq!(body.next(), None);
    })
}
