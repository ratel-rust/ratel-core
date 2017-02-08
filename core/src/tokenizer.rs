use std::str;
use grammar::Slice;
use lexicon::Token;
use lexicon::Token::*;
use lexicon::ReservedKind::*;
use lexicon::TemplateKind;
use operator::OperatorKind::*;
use grammar::Expression;
use grammar::VariableDeclarationKind::*;
use grammar::Value;
use error::{ Error, Result };

/// Helper macro for declaring byte-handler functions with correlating constants.
/// This becomes handy due to a lookup table present below.
macro_rules! define_handlers {
    { $(const $static_name:ident: $name:ident |$tok:pat, $byte:pat| $code:block)* } => {
        $(
            fn $name($tok: &mut Tokenizer, $byte: u8) -> Result<Token> $code

            const $static_name: fn(&mut Tokenizer, u8) -> Result<Token> = $name;
        )*
    }
}

macro_rules! expect_byte {
    ($tok:ident) => ({
        if $tok.is_eof() {
            return Err(Error::UnexpectedEndOfProgram);
        }

        let byte = $tok.read_byte();
        $tok.bump();

        byte
    })
}

/// Lookup table mapping any incoming byte to a handler function defined below.
static BYTE_HANDLERS: [fn(&mut Tokenizer, u8) -> Result<Token>; 256] = [
//   0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F   //
    ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, // 0
    ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, // 1
    ___, EXL, QOT, ___, IDT, PRC, AMP, QOT, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, MOR, QST, // 3
    ___, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, IDT, BTC, CRT, IDT, // 5
    TPL, IDT, L_B, L_C, L_D, L_E, L_F, IDT, IDT, L_I, IDT, IDT, L_L, IDT, L_N, IDT, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, BEO, PIP, BEC, TLD, ___, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];

// Handler function definitions:
define_handlers! {
    const ___: invalid_byte |tok, _| {
        Err(tok.invalid_character())
    }

    // ;
    const SEM: semicolon |tok, _| {
        tok.bump();

        Ok(Semicolon)
    }

    // :
    const COL: colon |tok, _| {
        tok.bump();

        Ok(Colon)
    }

    // ,
    const COM: comma |tok, _| {
        tok.bump();

        Ok(Comma)
    }

    // (
    const PNO: paren_open |tok, _| {
        tok.bump();

        Ok(ParenOpen)
    }

    // )
    const PNC: paren_close |tok, _| {
        tok.bump();

        Ok(ParenClose)
    }

    // [
    const BTO: bracket_open |tok, _| {
        tok.bump();

        Ok(BracketOpen)
    }

    // ]
    const BTC: bracket_close |tok, _| {
        tok.bump();

        Ok(BracketClose)
    }

    // {
    const BEO: brace_open |tok, _| {
        tok.bump();

        Ok(BraceOpen)
    }

    // }
    const BEC: brace_close |tok, _| {
        tok.bump();

        Ok(BraceClose)
    }

    // =
    const EQL: equal_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        StrictEquality
                    },

                    _ => Equality
                }
            },

            b'>' => {
                tok.bump();

                FatArrow
            },

            _ => Assign
        };

        Ok(Operator(op))
    }

    // !
    const EXL: exclamation_mark |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        StrictInequality
                    },

                    _ => Inequality
                }
            },

            _ => LogicalNot
        };

        Ok(Operator(op))
    }

    // <
    const LSS: less_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'<' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        BSLAssign
                    },

                    _ => BitShiftLeft
                }
            },

            b'=' => {
                tok.bump();

                LesserEquals
            },

            _ => Lesser
        };

        Ok(Operator(op))
    }

    // >
    const MOR: more_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'>' => {
                tok.bump();

                match tok.peek_byte() {
                    b'>' => {
                        tok.bump();

                        match tok.peek_byte() {
                            b'=' => {
                                tok.bump();

                                UBSRAssign
                            }

                            _ => UBitShiftRight
                        }
                    },

                    b'=' => BSRAssign,

                    _    => BitShiftRight
                }
            },

            b'=' => {
                tok.bump();

                GreaterEquals
            },

            _ => Greater
        };

        Ok(Operator(op))
    }

    // ?
    const QST: question_mark |tok, _| {
        tok.bump();

        Ok(Operator(Conditional))
    }

    // ~
    const TLD: tilde |tok, _| {
        tok.bump();

        Ok(Operator(BitwiseNot))
    }

    // ^
    const CRT: caret |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                BitXorAssign
            },

            _ => BitwiseXor
        };

        Ok(Operator(op))
    }

    // &
    const AMP: ampersand |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'&' => {
                tok.bump();

                LogicalAnd
            },

            b'=' => {
                tok.bump();

                BitAndAssign
            },

            _ => BitwiseAnd
        };

        Ok(Operator(op))
    }

    // |
    const PIP: pipe |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'|' => {
                tok.bump();

                LogicalOr
            },

            b'=' => {
                tok.bump();

                BitOrAssign
            },

            _ => BitwiseOr
        };

        Ok(Operator(op))
    }

    // +
    const PLS: plus_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'+' => {
                tok.bump();

                Increment
            },

            b'=' => {
                tok.bump();

                AddAssign
            },

            _ => Addition
        };

        Ok(Operator(op))
    }

    // -
    const MIN: minus_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'-' => {
                tok.bump();

                Decrement
            },

            b'=' => {
                tok.bump();

                SubstractAssign
            },

            _ => Substraction
        };

        Ok(Operator(op))
    }

    // *
    const ATR: asterisk |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'*' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        ExponentAssign
                    },

                    _ => Exponent
                }
            },

            b'=' => {
                tok.bump();

                MultiplyAssign
            },

            _ => Multiplication
        };

        Ok(Operator(op))
    }

    // /
    const SLH: slash |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            // regular comment
            b'/' => {
                tok.bump();

                // Keep consuming bytes until new line or end of source
                while !tok.is_eof() && tok.read_byte() != b'\n' {
                    tok.bump();
                }

                return tok.get_token();
            },

            // block comment
            b'*' => {
                tok.bump();

                // Keep consuming bytes until */ happens in a row
                while expect_byte!(tok) != b'*' ||
                      expect_byte!(tok) != b'/' {}

                return tok.get_token();
            },

            b'=' => {
                tok.bump();

                DivideAssign
            }

            _ => Division
        };

        Ok(Operator(op))
    }

    // %
    const PRC: percent |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                RemainderAssign
            },

            _ => Remainder
        };

        Ok(Operator(op))
    }

    // Non-keyword Identifier: starting with a letter, _ or $
    const IDT: identifier |tok, _| {
        Ok(Identifier(tok.consume_label_characters()))
    }

    // Identifier or keyword starting with a letter `b`
    const L_B: label_b |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "break"      => Break,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `c`
    const L_C: label_c |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "const"      => Declaration(Const),
            "case"       => Case,
            "class"      => Class,
            "catch"      => Catch,
            "continue"   => Continue,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `d`
    const L_D: label_d |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "delete"     => Operator(Delete),
            "do"         => Do,
            "debugger"   => Debugger,
            "default"    => Default,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `e`
    const L_E: label_e |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "else"       => Else,
            "export"     => Export,
            "extends"    => Extends,
            "enum"       => Reserved(Enum),
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `f`
    const L_F: label_f |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "finally"    => Finally,
            "for"        => For,
            "function"   => Function,
            "false"      => LitBoolean(false),
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `i`
    const L_I: label_i |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "in"         => Operator(In),
            "instanceof" => Operator(Instanceof),
            "if"         => If,
            "import"     => Import,
            "implements" => Reserved(Implements),
            "interface"  => Reserved(Interface),
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `l`
    const L_L: label_l |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "let"        => Declaration(Let),
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `n`
    const L_N: label_n |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "new"        => Operator(New),
            "null"       => Null,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `p`
    const L_P: label_p |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "package"    => Reserved(Package),
            "protected"  => Reserved(Protected),
            "private"    => Reserved(Private),
            "public"     => Reserved(Public),
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `r`
    const L_R: label_r |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "return"     => Return,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `s`
    const L_S: label_s |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "super"      => Super,
            "switch"     => Switch,
            "static"     => Static,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `t`
    const L_T: label_t |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "typeof"     => Operator(Typeof),
            "this"       => This,
            "throw"      => Throw,
            "try"        => Try,
            "true"       => LitBoolean(true),
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `u`
    const L_U: label_u |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "undefined"  => Undefined,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `v`
    const L_V: label_v |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "void"       => Operator(Void),
            "var"        => Declaration(Var),
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `w`
    const L_W: label_w |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "while"      => While,
            "with"       => With,
            _            => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `y`
    const L_Y: label_y |tok, _| {
        let slice = tok.consume_label_characters();

        Ok(match slice.as_str(tok.source) {
            "yield"      => Yield,
            _            => Identifier(slice),
        })
    }

    // Unicode character
    const UNI: unicode |tok, _| {
        let start = tok.index;

        let first = tok.source[start..].chars().next().expect("Has to have one");

        if !first.is_alphanumeric() {
            return Err(Error::UnexpectedToken {
                start: start,
                end: start + 1
            });
        }

        // `consume_label_characters` bumps one at the beginning,
        // so we subtract it here.
        tok.index += first.len_utf8() - 1;

        tok.consume_label_characters();

        let ident = Slice::new(start, tok.index);

        Ok(Identifier(ident))
    }

    // 0
    const ZER: zero |tok, _| {
        let start = tok.index;

        tok.bump();

        match tok.peek_byte() {
            b'b' | b'B' => {
                tok.bump();

                return Ok(tok.read_binary());
            },

            b'o' | b'O' => {
                tok.bump();

                return Ok(tok.read_octal(start));
            },

            b'x' | b'X' => {
                tok.bump();

                return Ok(tok.read_hexadec(start));
            },

            _ => {}
        }

        while !tok.is_eof() {
            match tok.read_byte() {
                b'0'...b'9' => {
                    tok.bump();
                },
                b'.' => {
                    tok.bump();

                    return Ok(tok.read_float(start));
                },
                b'e' | b'E' => {
                    tok.bump();
                    return Ok(tok.read_scientific(start));
                }
                _ => break,
            }
        }

        let value = Slice::new(start, tok.index);

        Ok(LitNumber(value))
    }

    // 1 to 9
    const DIG: digit |tok, _| {
        let start = tok.index;

        tok.bump();

        while !tok.is_eof() {
            match tok.read_byte() {
                b'0'...b'9' => {
                    tok.bump();
                },
                b'.' => {
                    tok.bump();

                    return Ok(tok.read_float(start));
                },
                b'e' | b'E' => {
                    tok.bump();
                    return Ok(tok.read_scientific(start));
                },
                _ => break,
            }
        }

        let value = Slice::new(start, tok.index);

        Ok(LitNumber(value))
    }

    // .
    const PRD: period |tok, _| {
        let start = tok.index;

        tok.bump();

        match tok.peek_byte() {
            b'0'...b'9' => {
                tok.bump();

                Ok(tok.read_float(start))
            },

            b'.' => {
                tok.bump();

                match tok.peek_byte() {
                    b'.' => {
                        tok.bump();

                        Ok(Operator(Spread))
                    },

                    _ => Err(tok.invalid_character())
                }
            },

            _ => Ok(Operator(Accessor))
        }
    }

    // " or '
    const QOT: quote |tok, byte| {
        let start = tok.index;

        tok.bump();

        loop {
            let ch = expect_byte!(tok);

            if ch == byte {
                break;
            }

            if ch == b'\\' {
                expect_byte!(tok);
            }
        }

        let value = Slice::new(start, tok.index);

        Ok(LitString(value))
    }

    // `
    const TPL: template |tok, _| {
        tok.bump();

        let template_kind = try!(tok.read_template_kind());

        Ok(Template(template_kind))
    }
}

mod whitespace {
    const __: bool = false;
    const WH: bool = true;

    pub static TABLE: [bool; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
      __, __, __, __, __, __, __, __, __, WH, WH, __, __, WH, __, __, // 0
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
      WH, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ];
}

mod ident_lookup {
    // Look up table that marks which ASCII characters are allowed in identifiers
    pub const NU: bool = true; // digit
    pub const AL: bool = true; // alphabet
    pub const DO: bool = true; // dollar sign $
    pub const US: bool = true; // underscore
    pub const UN: bool = true; // unicode
    pub const BS: bool = true; // backslash
    pub const __: bool = false;

    pub static TABLE: [bool; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
      __, __, __, __, DO, __, __, __, __, __, __, __, __, __, __, __, // 2
      NU, NU, NU, NU, NU, NU, NU, NU, NU, NU, __, __, __, __, __, __, // 3
      __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 4
      AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, BS, __, __, US, // 5
      __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 6
      AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, __, __, __, __, // 7
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // 8
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // 9
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // A
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // B
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // C
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // D
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // E
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // F
    ];
}

pub struct Tokenizer<'src> {
    /// Flags whether or not a new line was read before the token
    consumed_new_line: bool,

    /// String slice to parse
    source: &'src str,

    /// Current index
    index: usize,

    /// Index of current token in source
    token_start: usize,
}


impl<'src> Tokenizer<'src> {
    #[inline]
    pub fn new(source: &'src str) -> Self {
        Tokenizer {
            consumed_new_line: false,
            source: source,
            index: 0,
            token_start: 0,
        }
    }

    #[inline]
    pub fn get_token(&mut self) -> Result<Token> {
        self.consume_whitespace();

        self.token_start = self.index;

        if self.is_eof() {
            return Ok(EndOfProgram);
        }

        let ch = self.read_byte();

        BYTE_HANDLERS[ch as usize](self, ch)
    }

    /// On top of being called when the opening backtick (`) of a template
    /// literal occurs, this method needs to be used by the parser while
    /// parsing a complex template string expression.
    ///
    /// **Note:** Parser needs to expect a BraceClose token before calling
    /// this method to ensure that the tokenizer state is not corrupted.
    #[inline]
    pub fn read_template_kind(&mut self) -> Result<TemplateKind> {
        let start = self.index;

        loop {
            let ch = expect_byte!(self);

            match ch {
                b'`' => {
                    let quasi = Slice::new(start, self.index - 1);

                    return Ok(TemplateKind::Closed(quasi));
                },
                b'$' => {
                    let ch = expect_byte!(self);

                    if ch != b'{' {
                        continue;
                    }

                    let quasi = Slice::new(start, self.index - 2);

                    return Ok(TemplateKind::Open(quasi));
                },
                b'\\' => {
                    expect_byte!(self);
                },
                _ => {}
            }
        }
    }

    /// Check if Automatic Semicolon Insertion rules can be applied
    #[inline]
    pub fn asi(&self) -> bool {
        self.consumed_new_line
    }

    pub fn invalid_token(&self) -> Error {
        Error::UnexpectedToken {
            start: self.token_start,
            end: self.index,
        }
    }

    fn invalid_character(&self) -> Error {
        if self.is_eof() {
            return Error::UnexpectedEndOfProgram;
        }

        let len = self.source[self.index..]
                      .chars()
                      .next()
                      .expect("Must have a character")
                      .len_utf8();

        Error::UnexpectedToken {
            start: self.index,
            end: self.index + len,
        }
    }

    // Check if we are at the end of the source.
    #[inline]
    fn is_eof(&self) -> bool {
        self.index == self.source.len()
    }

    // Read a byte from the source. Note that this does not increment
    // the index. In few cases (all of them related to number parsing)
    // we want to peek at the byte before doing anything. This will,
    // very very rarely, lead to a situation where the same byte is read
    // twice, but since this operation is using a raw pointer, the cost
    // is virtually irrelevant.
    #[inline]
    fn read_byte(&self) -> u8 {
        unsafe { *self.source.as_ptr().offset(self.index as isize) }
    }

    #[inline]
    fn peek_byte(&self) -> u8 {
        if self.is_eof() {
            return 0;
        }

        self.read_byte()
    }

    // Manually increment the index. Calling `read_byte` and then `bump`
    // is equivalent to consuming a byte on an iterator.
    #[inline]
    fn bump(&mut self) {
        self.index += 1;
    }


    #[inline]
    fn consume_whitespace(&mut self) {
        self.consumed_new_line = false;

        while !self.is_eof() {
            let ch = self.read_byte();

            // if ch <= 0x20 {
            if whitespace::TABLE[ch as usize] {
                if ch == b'\n' {
                    self.consumed_new_line = true;
                }

                self.bump();
                continue;
            }

            return;
        }
    }

    #[inline]
    fn read_binary(&mut self) -> Token {
        let mut value = 0;

        while !self.is_eof() {
            let peek = self.read_byte();
            match peek {
                b'0' => {
                    value <<= 1;
                    self.bump();
                },
                b'1' => {
                    value = (value << 1) + 1;
                    self.bump();
                },
                _ => break
            }
        }

        LitBinary(value)
    }

    #[inline]
    fn consume_label_characters(&mut self) -> Slice {
        // TODO: Reject invalid unicode and escaped unicode character

        let start = self.index;

        self.bump();

        while !self.is_eof() && ident_lookup::TABLE[self.read_byte() as usize] {
            self.bump();
        }

        Slice::new(start, self.index)
    }

    #[inline]
    fn read_octal(&mut self, start: usize) -> Token {
        while !self.is_eof() {
            match self.read_byte() {
                b'0'...b'7' => self.bump(),
                _           => break
            };
        }

        LitNumber(Slice::new(start, self.index))
    }

    #[inline]
    fn read_hexadec(&mut self, start: usize) -> Token {
        while !self.is_eof() {
            match self.read_byte() {
                b'0'...b'9' => (),
                b'a'...b'f' => (),
                b'A'...b'F' => (),
                _           => break
            };

            self.bump();
        }

        LitNumber(Slice::new(start, self.index))
    }

    #[inline]
    fn read_float(&mut self, start: usize) -> Token {
        while !self.is_eof() {
            let ch = self.read_byte();
            match ch {
                b'0'...b'9'  => self.bump(),
                b'e' | b'E'  => {
                    self.bump();
                    return self.read_scientific(start);
                },
                _            => break
            }
        }

        let value = Slice::new(start, self.index);

        LitNumber(value)
    }

    #[inline]
    fn read_scientific(&mut self, start: usize) -> Token {
        if !self.is_eof() {
            match self.read_byte() {
                b'-' | b'+' => self.bump(),
                _           => {}
            }
        }

        while !self.is_eof() {
            let ch = self.read_byte();
            match ch {
                b'0'...b'9' => self.bump(),
                _           => break
            }
        }

        let value = Slice::new(start, self.index);

        LitNumber(value)
    }

    #[inline]
    pub fn read_regular_expression(&mut self) -> Result<Expression> {
        let start = self.index;
        let mut in_class = false;

        loop {
            let ch = expect_byte!(self);
            match ch {
                b'['  => {
                    in_class = true;
                },
                b']'  => {
                    in_class = false;
                },
                b'/'  => {
                    if !in_class {
                        break;
                    }
                },
                b'\\' => {
                    expect_byte!(self);
                },
                b'\n' => {
                    return Err(Error::UnexpectedToken {
                        start: self.index,
                        end: self.index + 1
                    });
                },
                _     => {}
            }
        }

        let pattern = Slice::new(start, self.index - 1);
        let flags_start = self.index;

        while !self.is_eof() {
            let ch = self.peek_byte();
            match ch {
                b'g' | b'i' | b'm' | b'u' | b'y' => {
                    self.bump();
                },
                _                                => {
                    break;
                }
            }
        }

        Ok(Expression::RegEx{
            pattern: pattern,
            flags: Slice::new(flags_start, self.index)
        })
    }
}
