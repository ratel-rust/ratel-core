mod token;

pub use lexer::token::*;

use lexer::token::Token::*;
use lexer::ReservedKind::*;
use lexer::TemplateKind;

use std::str;
use ast::Value;
use ast::OperatorKind::*;
use ast::VariableDeclarationKind::*;
use error::{ Error, Result };

/// Helper macro for declaring byte-handler functions with correlating constants.
/// This becomes handy due to a lookup table present below.
macro_rules! define_handlers {
    { $(const $static_name:ident: $name:ident |$lex:pat, $byte:pat| $code:block)* } => {
        $(
            fn $name<'src>($lex: &mut Lexer<'src>, $byte: u8) -> Result<Token<'src>> $code

            const $static_name: for<'src> fn(&mut Lexer<'src>, u8) -> Result<Token<'src>> = $name;
        )*
    }
}

macro_rules! expect_byte {
    ($lex:ident) => ({
        if $lex.is_eof() {
            return Err(Error::UnexpectedEndOfProgram);
        }

        let byte = $lex.read_byte();
        $lex.bump();

        byte
    })
}

macro_rules! unwind_loop {
    ($iteration:expr) => ({
        $iteration
        $iteration
        $iteration
        $iteration
        $iteration

        loop {
            $iteration
            $iteration
            $iteration
            $iteration
            $iteration
        }
    })
}

/// Lookup table mapping any incoming byte to a handler function defined below.
static BYTE_HANDLERS: [for<'src> fn(&mut Lexer<'src>, u8) -> Result<Token<'src>>; 256] = [
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
    const ___: invalid_byte |lex, _| {
        Err(lex.invalid_character())
    }

    // ;
    const SEM: semicolon |lex, _| {
        lex.bump();

        Ok(Semicolon)
    }

    // :
    const COL: colon |lex, _| {
        lex.bump();

        Ok(Colon)
    }

    // ,
    const COM: comma |lex, _| {
        lex.bump();

        Ok(Comma)
    }

    // (
    const PNO: paren_open |lex, _| {
        lex.bump();

        Ok(ParenOpen)
    }

    // )
    const PNC: paren_close |lex, _| {
        lex.bump();

        Ok(ParenClose)
    }

    // [
    const BTO: bracket_open |lex, _| {
        lex.bump();

        Ok(BracketOpen)
    }

    // ]
    const BTC: bracket_close |lex, _| {
        lex.bump();

        Ok(BracketClose)
    }

    // {
    const BEO: brace_open |lex, _| {
        lex.bump();

        Ok(BraceOpen)
    }

    // }
    const BEC: brace_close |lex, _| {
        lex.bump();

        Ok(BraceClose)
    }

    // =
    const EQL: equal_sign |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'=' => {
                lex.bump();

                match lex.peek_byte() {
                    b'=' => {
                        lex.bump();

                        StrictEquality
                    },

                    _ => Equality
                }
            },

            b'>' => {
                lex.bump();

                FatArrow
            },

            _ => Assign
        };

        Ok(Operator(op))
    }

    // !
    const EXL: exclamation_mark |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'=' => {
                lex.bump();

                match lex.peek_byte() {
                    b'=' => {
                        lex.bump();

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
    const LSS: less_sign |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'<' => {
                lex.bump();

                match lex.peek_byte() {
                    b'=' => {
                        lex.bump();

                        BSLAssign
                    },

                    _ => BitShiftLeft
                }
            },

            b'=' => {
                lex.bump();

                LesserEquals
            },

            _ => Lesser
        };

        Ok(Operator(op))
    }

    // >
    const MOR: more_sign |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'>' => {
                lex.bump();

                match lex.peek_byte() {
                    b'>' => {
                        lex.bump();

                        match lex.peek_byte() {
                            b'=' => {
                                lex.bump();

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
                lex.bump();

                GreaterEquals
            },

            _ => Greater
        };

        Ok(Operator(op))
    }

    // ?
    const QST: question_mark |lex, _| {
        lex.bump();

        Ok(Operator(Conditional))
    }

    // ~
    const TLD: tilde |lex, _| {
        lex.bump();

        Ok(Operator(BitwiseNot))
    }

    // ^
    const CRT: caret |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'=' => {
                lex.bump();

                BitXorAssign
            },

            _ => BitwiseXor
        };

        Ok(Operator(op))
    }

    // &
    const AMP: ampersand |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'&' => {
                lex.bump();

                LogicalAnd
            },

            b'=' => {
                lex.bump();

                BitAndAssign
            },

            _ => BitwiseAnd
        };

        Ok(Operator(op))
    }

    // |
    const PIP: pipe |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'|' => {
                lex.bump();

                LogicalOr
            },

            b'=' => {
                lex.bump();

                BitOrAssign
            },

            _ => BitwiseOr
        };

        Ok(Operator(op))
    }

    // +
    const PLS: plus_sign |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'+' => {
                lex.bump();

                Increment
            },

            b'=' => {
                lex.bump();

                AddAssign
            },

            _ => Addition
        };

        Ok(Operator(op))
    }

    // -
    const MIN: minus_sign |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'-' => {
                lex.bump();

                Decrement
            },

            b'=' => {
                lex.bump();

                SubstractAssign
            },

            _ => Substraction
        };

        Ok(Operator(op))
    }

    // *
    const ATR: asterisk |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'*' => {
                lex.bump();

                match lex.peek_byte() {
                    b'=' => {
                        lex.bump();

                        ExponentAssign
                    },

                    _ => Exponent
                }
            },

            b'=' => {
                lex.bump();

                MultiplyAssign
            },

            _ => Multiplication
        };

        Ok(Operator(op))
    }

    // /
    const SLH: slash |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            // regular comment
            b'/' => {
                lex.bump();

                // Keep consuming bytes until new line or end of source
                unwind_loop!({
                    if lex.is_eof() || lex.read_byte() == b'\n' {
                        return lex.get_token();
                    }
                    lex.bump();
                });
            },

            // block comment
            b'*' => {
                lex.bump();

                // Keep consuming bytes until */ happens in a row
                unwind_loop!({
                    if expect_byte!(lex) == b'*' && expect_byte!(lex) == b'/' {
                        return lex.get_token();
                    }
                });
            },

            b'=' => {
                lex.bump();

                DivideAssign
            }

            _ => Division
        };

        Ok(Operator(op))
    }

    // %
    const PRC: percent |lex, _| {
        lex.bump();

        let op = match lex.peek_byte() {
            b'=' => {
                lex.bump();

                RemainderAssign
            },

            _ => Remainder
        };

        Ok(Operator(op))
    }

    // Non-keyword Identifier: starting with a letter, _ or $
    const IDT: identifier |lex, _| {
        Ok(Identifier(lex.consume_label_characters()))
    }

    // Identifier or keyword starting with a letter `b`
    const L_B: label_b |lex, _| {
        Ok(match lex.consume_label_characters() {
            "break"      => Break,
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `c`
    const L_C: label_c |lex, _| {
        Ok(match lex.consume_label_characters() {
            "const"      => Declaration(Const),
            "case"       => Case,
            "class"      => Class,
            "catch"      => Catch,
            "continue"   => Continue,
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `d`
    const L_D: label_d |lex, _| {
        Ok(match lex.consume_label_characters() {
            "delete"     => Operator(Delete),
            "do"         => Do,
            "debugger"   => Debugger,
            "default"    => Default,
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `e`
    const L_E: label_e |lex, _| {
        Ok(match lex.consume_label_characters() {
            "else"       => Else,
            "export"     => Export,
            "extends"    => Extends,
            "enum"       => Reserved(Enum),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `f`
    const L_F: label_f |lex, _| {
        Ok(match lex.consume_label_characters() {
            "finally"    => Finally,
            "for"        => For,
            "function"   => Function,
            "false"      => Literal(Value::False),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `i`
    const L_I: label_i |lex, _| {
        Ok(match lex.consume_label_characters() {
            "in"         => Operator(In),
            "instanceof" => Operator(Instanceof),
            "if"         => If,
            "import"     => Import,
            "implements" => Reserved(Implements),
            "interface"  => Reserved(Interface),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `l`
    const L_L: label_l |lex, _| {
        Ok(match lex.consume_label_characters() {
            "let"        => Declaration(Let),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `n`
    const L_N: label_n |lex, _| {
        Ok(match lex.consume_label_characters() {
            "new"        => Operator(New),
            "null"       => Literal(Value::Null),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `p`
    const L_P: label_p |lex, _| {
        Ok(match lex.consume_label_characters() {
            "package"    => Reserved(Package),
            "protected"  => Reserved(Protected),
            "private"    => Reserved(Private),
            "public"     => Reserved(Public),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `r`
    const L_R: label_r |lex, _| {
        Ok(match lex.consume_label_characters() {
            "return"     => Return,
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `s`
    const L_S: label_s |lex, _| {
        Ok(match lex.consume_label_characters() {
            "super"      => Super,
            "switch"     => Switch,
            "static"     => Static,
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `t`
    const L_T: label_t |lex, _| {
        Ok(match lex.consume_label_characters() {
            "typeof"     => Operator(Typeof),
            "this"       => This,
            "throw"      => Throw,
            "try"        => Try,
            "true"       => Literal(Value::True),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `u`
    const L_U: label_u |lex, _| {
        Ok(match lex.consume_label_characters() {
            "undefined"  => Literal(Value::Undefined),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `v`
    const L_V: label_v |lex, _| {
        Ok(match lex.consume_label_characters() {
            "void"       => Operator(Void),
            "var"        => Declaration(Var),
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `w`
    const L_W: label_w |lex, _| {
        Ok(match lex.consume_label_characters() {
            "while"      => While,
            "with"       => With,
            slice        => Identifier(slice),
        })
    }

    // Identifier or keyword starting with a letter `y`
    const L_Y: label_y |lex, _| {
        Ok(match lex.consume_label_characters() {
            "yield"      => Yield,
            slice        => Identifier(slice),
        })
    }

    // Unicode character
    const UNI: unicode |lex, _| {
        let start = lex.index;

        let first = lex.source[start..].chars().next().expect("Has to have one");

        if !first.is_alphanumeric() {
            return Err(Error::UnexpectedToken {
                start: start,
                end: start + 1
            });
        }

        // `consume_label_characters` bumps one at the beginning,
        // so we subtract it here.
        lex.index += first.len_utf8() - 1;

        lex.consume_label_characters();

        let ident = lex.slice_from(start);

        Ok(Identifier(ident))
    }

    // 0
    const ZER: zero |lex, _| {
        let start = lex.index;

        lex.bump();

        match lex.peek_byte() {
            b'b' | b'B' => {
                lex.bump();

                return Ok(lex.read_binary());
            },

            b'o' | b'O' => {
                lex.bump();

                return Ok(lex.read_octal(start));
            },

            b'x' | b'X' => {
                lex.bump();

                return Ok(lex.read_hexadec(start));
            },

            _ => {}
        }

        while !lex.is_eof() {
            match lex.read_byte() {
                b'0'...b'9' => {
                    lex.bump();
                },
                b'.' => {
                    lex.bump();

                    return Ok(lex.read_float(start));
                },
                b'e' | b'E' => {
                    lex.bump();
                    return Ok(lex.read_scientific(start));
                }
                _ => break,
            }
        }

        let value = lex.slice_from(start);

        Ok(Literal(Value::Number(value)))
    }

    // 1 to 9
    const DIG: digit |lex, _| {
        let start = lex.index;

        lex.bump();

        while !lex.is_eof() {
            match lex.read_byte() {
                b'0'...b'9' => {
                    lex.bump();
                },
                b'.' => {
                    lex.bump();

                    return Ok(lex.read_float(start));
                },
                b'e' | b'E' => {
                    lex.bump();
                    return Ok(lex.read_scientific(start));
                },
                _ => break,
            }
        }

        let value = lex.slice_from(start);

        Ok(Literal(Value::Number(value)))
    }

    // .
    const PRD: period |lex, _| {
        let start = lex.index;

        lex.bump();

        match lex.peek_byte() {
            b'0'...b'9' => {
                lex.bump();

                Ok(lex.read_float(start))
            },

            b'.' => {
                lex.bump();

                match lex.peek_byte() {
                    b'.' => {
                        lex.bump();

                        Ok(Operator(Spread))
                    },

                    _ => Err(lex.invalid_character())
                }
            },

            _ => Ok(Operator(Accessor))
        }
    }

    // " or '
    const QOT: quote |lex, byte| {
        let start = lex.index;

        lex.bump();

        loop {
            let ch = expect_byte!(lex);

            if ch == byte {
                break;
            }

            if ch == b'\\' {
                expect_byte!(lex);
            }
        }

        let value = lex.slice_from(start);

        Ok(Literal(Value::String(value)))
    }

    // `
    const TPL: template |lex, _| {
        lex.bump();

        let template_kind = try!(lex.read_template_kind());

        Ok(Template(template_kind))
    }
}

pub struct Lexer<'src> {
    /// Flags whether or not a new line was read before the token
    consumed_new_line: bool,

    /// String slice to parse
    source: &'src str,

    /// ptr
    ptr: *const u8,

    /// Current index
    index: usize,

    /// Index of current token in source
    token_start: usize,
}


impl<'src> Lexer<'src> {
    #[inline]
    pub fn new(source: &'src str) -> Self {
        Lexer {
            consumed_new_line: false,
            source: source,
            ptr: source.as_ptr(),
            index: 0,
            token_start: 0,
        }
    }

    #[inline(always)]
    pub fn get_token(&mut self) -> Result<Token<'src>> {
        self.consumed_new_line = false;

        unwind_loop!({
            if self.is_eof() {
                return Ok(EndOfProgram);
            }

            let ch = self.read_byte();

            if ch > 0x20 {
                self.token_start = self.index;

                return unsafe { BYTE_HANDLERS.get_unchecked(ch as usize)(self, ch) };
            }

            if ch == b'\n' {
                self.consumed_new_line = true;
            }

            self.bump();
        })
    }

    #[inline(always)]
    pub fn loc(&self) -> (usize, usize) {
        (self.token_start, self.index)
    }

    #[inline(always)]
    pub fn loc_start(&self) -> usize {
        self.token_start
    }

    #[inline(always)]
    pub fn loc_end(&self) -> usize {
        self.index
    }

    /// On top of being called when the opening backtick (`) of a template
    /// literal occurs, this method needs to be used by the parser while
    /// parsing a complex template string expression.
    ///
    /// **Note:** Parser needs to expect a BraceClose token before calling
    /// this method to ensure that the tokenizer state is not corrupted.
    #[inline]
    pub fn read_template_kind(&mut self) -> Result<TemplateKind<'src>> {
        let start = self.index;

        loop {
            let ch = expect_byte!(self);

            match ch {
                b'`' => {
                    let quasi = &self.source[start..self.index - 1];

                    return Ok(TemplateKind::Closed(quasi));
                },
                b'$' => {
                    let ch = expect_byte!(self);

                    if ch != b'{' {
                        continue;
                    }

                    let quasi = &self.source[start..self.index - 2];

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
        unsafe { *self.ptr.offset(self.index as isize) }
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
    fn read_binary(&mut self) -> Token<'src> {
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

        Literal(Value::Binary(value))
    }

    #[inline(always)]
    fn consume_label_characters(&mut self) -> &'src str {
        // Look up table that marks which ASCII characters are allowed in identifiers
        const NU: bool = true; // digit
        const AL: bool = true; // alphabet
        const DO: bool = true; // dollar sign $
        const US: bool = true; // underscore
        const UN: bool = true; // unicode
        const BS: bool = true; // backslash
        const __: bool = false;

        static TABLE: [bool; 256] = [
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

        let start = self.index;

        if self.index + 8 < self.source.len() {
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
            self.bump();
            if !TABLE[self.read_byte() as usize] {
                return self.slice_from(start);
            }
        }

        self.bump();

        while !self.is_eof() && TABLE[self.read_byte() as usize] {
            self.bump();
        }

        self.slice_from(start)
    }

    #[inline(always)]
    fn slice_from(&self, start: usize) -> &'src str {
        unsafe { self.source.slice_unchecked(start, self.index) }
    }

    #[inline]
    fn read_octal(&mut self, start: usize) -> Token<'src> {
        while !self.is_eof() {
            match self.read_byte() {
                b'0'...b'7' => self.bump(),
                _           => break
            };
        }

        let value = self.slice_from(start);

        Literal(Value::Number(value))
    }

    #[inline]
    fn read_hexadec(&mut self, start: usize) -> Token<'src> {
        while !self.is_eof() {
            match self.read_byte() {
                b'0'...b'9' => (),
                b'a'...b'f' => (),
                b'A'...b'F' => (),
                _           => break
            };

            self.bump();
        }

        let value = self.slice_from(start);

        Literal(Value::Number(value))
    }

    #[inline]
    fn read_float(&mut self, start: usize) -> Token<'src> {
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

        let value = self.slice_from(start);

        Literal(Value::Number(value))
    }

    #[inline]
    fn read_scientific(&mut self, start: usize) -> Token<'src> {
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

        let value = self.slice_from(start);

        Literal(Value::Number(value))
    }

    // #[inline]
    // pub fn read_regular_expression(&mut self) -> Result<Expression> {
    //     let start = self.index;
    //     let mut in_class = false;

    //     loop {
    //         let ch = expect_byte!(self);
    //         match ch {
    //             b'['  => {
    //                 in_class = true;
    //             },
    //             b']'  => {
    //                 in_class = false;
    //             },
    //             b'/'  => {
    //                 if !in_class {
    //                     break;
    //                 }
    //             },
    //             b'\\' => {
    //                 expect_byte!(self);
    //             },
    //             b'\n' => {
    //                 return Err(Error::UnexpectedToken {
    //                     start: self.index,
    //                     end: self.index + 1
    //                 });
    //             },
    //             _     => {}
    //         }
    //     }

    //     let pattern = Slice(start, self.index - 1);
    //     let flags_start = self.index;

    //     while !self.is_eof() {
    //         let ch = self.peek_byte();
    //         match ch {
    //             b'g' | b'i' | b'm' | b'u' | b'y' => {
    //                 self.bump();
    //             },
    //             _                                => {
    //                 break;
    //             }
    //         }
    //     }

    //     Ok(Expression::RegEx{
    //         pattern: pattern,
    //         flags: Slice(flags_start, self.index)
    //     })
    // }
}
