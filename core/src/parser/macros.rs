/// Peek on the next token. Return with an error if lexer fails.
#[macro_export]
macro_rules! peek {
    ($parser:ident) => {
        match $parser.token {
            Some(token) => token,

            None => {
                let token = $parser.lexer.get_token();

                $parser.token = Some(token);

                token
            }
        }
    }
}

/// Get the next token. Return with an error if lexer fails.
#[macro_export]
macro_rules! next {
    ($parser:ident) => {
        match $parser.token.take() {
            Some(token) => token,
            None        => $parser.lexer.get_token()
        }
    }
}

// #[macro_export]
// macro_rules! next_raw_ident {
//     ($parser:ident) => ({
//         use ast::OperatorKind::*;
//         use ast::Value::*;

//         match next!($parser) {
//             Identifier(ident)    => ident,
//             Break                => "break",
//             Do                   => "do",
//             Case                 => "case",
//             Else                 => "else",
//             Catch                => "catch",
//             Export               => "export",
//             Class                => "class",
//             Extends              => "extends",
//             Return               => "return",
//             While                => "while",
//             Finally              => "finally",
//             Super                => "super",
//             With                 => "with",
//             Continue             => "continue",
//             For                  => "for",
//             Switch               => "switch",
//             Yield                => "yield",
//             Debugger             => "debugger",
//             Function             => "function",
//             This                 => "this",
//             Default              => "default",
//             If                   => "if",
//             Throw                => "throw",
//             Import               => "import",
//             Try                  => "try",
//             Static               => "static",
//             Operator(New)        => "new",
//             Operator(Typeof)     => "typeof",
//             Operator(Void)       => "void",
//             Operator(Delete)     => "delete",
//             Operator(Instanceof) => "instanceof",
//             Literal(True)        => "true",
//             Literal(False)       => "false",
//             Literal(Null)        => "null",
//             Literal(Undefined)   => "undefined",

//             _                    => unexpected_token!($parser),
//         }
//     })
// }
macro_rules! expect_raw_ident {
    ($parser:ident) => ({
        debug_assert!($parser.token == None);

        match $parser.lexer.get_raw_identifier() {
            Identifier(ident) => ident,
            _                 => unexpected_token!($parser)
        }
    })
}

/// If the next token matches `$p`, consume that token and execute `$eval`.
#[macro_export]
macro_rules! allow {
    ($parser:ident, $p:pat => $eval:expr) => {
        match peek!($parser) {
            $p => {
                $parser.consume();
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
        match next!($parser) {
            $p => {},
            _  => unexpected_token!($parser)
        }
    }
}

/// Expect the next token to be an Identifier, extracting the OwnedSlice
/// out of it. Returns an error otherwise.
#[macro_export]
macro_rules! expect_identifier {
    ($parser:ident) => {
        match next!($parser) {
            Token::Identifier(ident) => ident,
            _                        => unexpected_token!($parser)
        }
    }
}

/// Expecta semicolon to terminate a statement. Will assume a semicolon
/// following the ASI rules.
#[macro_export]
macro_rules! expect_semicolon {
    ($parser:ident) => {
        // TODO: Lexer needs to flag when a new line character has been
        //       consumed to satisfy all ASI rules
        match peek!($parser) {
            Semicolon     => $parser.consume(),

            ParenClose    |
            BraceClose    |
            EndOfProgram  => {},

            _             => {
                if !$parser.lexer.asi() {
                    unexpected_token!($parser)
                }
            }
        }
    }
}

/// Return an error for current token.
#[macro_export]
macro_rules! unexpected_token {
    ($parser:ident) => {
        return Err($parser.lexer.invalid_token())
    };
}

#[cfg(test)]
#[macro_rules]
macro_rules! assert_ident {
    ($expect:expr, $item:expr) => {
        assert_eq!(Identifier($expect.into()), $item);
    }
}

#[cfg(test)]
#[macro_rules]
macro_rules! assert_list {
    ($iter:expr $( ,$item:expr)*) => ({
        let mut iter = $iter;
        $(
            assert_eq!($item, *iter.next().unwrap());
        )*
        assert_eq!(None, iter.next());
    })
}
