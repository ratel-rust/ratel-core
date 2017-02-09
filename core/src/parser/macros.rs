/// Peek on the next token. Return with an error if tokenizer fails.
#[macro_export]
macro_rules! peek {
    ($parser:ident) => {
        match $parser.token {
            Some(token) => token,

            None => {
                let token = try!($parser.tokenizer.get_token());

                $parser.token = Some(token);

                token
            }
        }
    }
}

/// Get the next token. Return with an error if tokenizer fails.
#[macro_export]
macro_rules! next {
    ($parser:ident) => {
        match $parser.token {
            Some(token) => {
                $parser.consume();

                token
            },
            None => try!($parser.tokenizer.get_token())
        }
    }
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
            Identifier(ident) => ident,
            _                 => unexpected_token!($parser)
        }
    }
}

/// Expecta semicolon to terminate a statement. Will assume a semicolon
/// following the ASI rules.
#[macro_export]
macro_rules! expect_semicolon {
    ($parser:ident) => {
        // TODO: Tokenizer needs to flag when a new line character has been
        //       consumed to satisfy all ASI rules
        match peek!($parser) {
            Semicolon     => $parser.consume(),

            ParenClose    |
            BraceClose    |
            EndOfProgram  => {},

            _             => {
                if !$parser.tokenizer.asi() {
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
        return Err($parser.tokenizer.invalid_token())
    };
}
