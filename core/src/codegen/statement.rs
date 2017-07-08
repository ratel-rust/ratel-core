use ast::{Statement, Declarator, DeclarationKind};
use codegen::{ToCode, Generator};

impl<G: Generator> ToCode<G> for DeclarationKind {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::DeclarationKind::*;

        match *self {
            Var   => gen.write_bytes(b"var "),
            Let   => gen.write_bytes(b"let "),
            Const => gen.write_bytes(b"const "),
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for Declarator<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.name);

        if let Some(ref value) = self.value {
            gen.write_pretty(b' ');
            gen.write_byte(b'=');
            gen.write_pretty(b' ');
            gen.write(value);
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for Statement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::Statement::*;

        match *self {
            Error => panic!("Module contains errors"),
            Empty => {},
            Expression {
                ref expression
            } => {
                if expression.is_allowed_as_bare_statement() {
                    gen.write(expression);
                } else {
                    gen.write_byte(b'(');
                    gen.write(expression);
                    gen.write_byte(b')');
                }
                gen.write_byte(b';');
            },
            Declaration {
                ref kind,
                ref declarators,
            } => {
                gen.write(kind);
                gen.write_list(declarators);
                gen.write_byte(b';');
            },
            Return {
                ref value,
            } => {
                match *value {
                    Some(ref value) => {
                        gen.write_bytes(b"return ");
                        gen.write(value);
                        gen.write_byte(b';');
                    },
                    None => gen.write_bytes(b"return;")
                }
            },
            Break {
                ref label,
            } => {
                match *label {
                    Some(ref label) => {
                        gen.write_bytes(b"break ");
                        gen.write(label);
                        gen.write_byte(b';');
                    },
                    None => gen.write_bytes(b"break;")
                }
            },
            Throw {
                ref value,
            } => {
                gen.write_bytes(b"throw ");
                gen.write(value);
                gen.write_byte(b';');
            },
            If {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                gen.write_bytes(b"if");
                gen.write_pretty(b' ');
                gen.write_byte(b'(');
                gen.write(test);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write(consequent);

                if let Some(ref alternate) = *alternate {
                    match consequent.is_block() {
                        true  => gen.write_pretty(b' '),
                        false => gen.write_byte(b' '),
                    }
                    gen.write_bytes(b"else");
                    match alternate.is_block() {
                        true  => gen.write_pretty(b' '),
                        false => gen.write_byte(b' '),
                    }
                    gen.write(alternate);
                }
            },
            While {
                ref test,
                ref body,
            } => {
                gen.write_bytes(b"while");
                gen.write_pretty(b' ');
                gen.write_byte(b'(');
                gen.write(test);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write(body);
            },
            Do {
                ref test,
                ref body,
            } => {
                gen.write_bytes(b"do ");
                gen.write(body);
                gen.write_bytes(b"while");
                gen.write_pretty(b' ');
                gen.write_byte(b'(');
                gen.write(test);
                gen.write_byte(b')');
            },
            For {
                ref init,
                ref test,
                ref update,
                ref body,
            } => {
                gen.write_bytes(b"for");
                gen.write_pretty(b' ');
                gen.write_byte(b'(');
                if let Some(ref init) = *init {
                    gen.write_declaration_or_expression(init);
                }
                gen.write_byte(b';');
                gen.write_pretty(b' ');
                if let Some(ref test) = *test {
                    gen.write(test);
                }
                gen.write_byte(b';');
                gen.write_pretty(b' ');
                if let Some(ref update) = *update {
                    gen.write(update);
                }
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write(body);
            },
            ForIn {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_bytes(b"for");
                gen.write_pretty(b' ');
                gen.write_byte(b'(');
                gen.write_declaration_or_expression(left);
                gen.write_bytes(b" in ");
                gen.write(right);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write(body);
            },
            ForOf {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_bytes(b"for");
                gen.write_pretty(b' ');
                gen.write_byte(b'(');
                gen.write_declaration_or_expression(left);
                gen.write_bytes(b" of ");
                gen.write(right);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write(body);
            },
            Try {
                ref body,
                ref error,
                ref handler,
            } => {
                gen.write_bytes(b"try");
                gen.write_pretty(b' ');
                gen.write_byte(b'{');
                gen.write_block(body);
                gen.write_byte(b'}');
                gen.write_pretty(b' ');
                gen.write_bytes(b"catch");
                gen.write_pretty(b' ');
                gen.write_byte(b'(');
                gen.write(error);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write_byte(b'{');
                gen.write_block(handler);
                gen.write_byte(b'}');
            },
            Block {
                ref body,
            } => {
                gen.write_byte(b'{');
                gen.write_block(body);
                gen.write_byte(b'}');
            },
            Function {
                ref function,
            } => {
                gen.write(function);
                gen.new_line();
            },
            Class {
                ref class,
            } => {
                gen.write(class);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use codegen::assert_parse;

    #[test]
    fn function_statement() {
        assert_parse("function foo() {}", "function foo(){}");
        assert_parse("function foo(a) {}", "function foo(a){}");
        assert_parse("function foo(a, b, c) {}", "function foo(a,b,c){}");
        assert_parse("function foo(bar) { return 10; }", "function foo(bar){return 10;}");
    }

    #[test]
    fn declaration_statement() {
        assert_parse("var foo;", "var foo;");
        assert_parse("let foo;", "let foo;");
        assert_parse("const foo;", "const foo;");
        assert_parse("var foo = 10;", "var foo=10;");
        assert_parse("let foo = 10;", "let foo=10;");
        assert_parse("const foo = 10;", "const foo=10;");
        assert_parse("var foo, bar;", "var foo,bar;");
        assert_parse("let foo, bar;", "let foo,bar;");
        assert_parse("const foo, bar;", "const foo,bar;");
        assert_parse("var foo = 10, bar = 20;", "var foo=10,bar=20;");
        assert_parse("let foo = 10, bar = 20;", "let foo=10,bar=20;");
        assert_parse("const foo = 10, bar = 20;", "const foo=10,bar=20;");
    }

    #[test]
    fn if_statement() {
        assert_parse("if (true) foo;", "if(true)foo;");
        assert_parse("if (true) { foo; }", "if(true){foo;}");
        assert_parse("if (true) foo; else bar;", "if(true)foo; else bar;");
        assert_parse("if (true) { foo; } else { bar; }", "if(true){foo;}else{bar;}");
        assert_parse("if (true) foo; else { bar; }", "if(true)foo; else{bar;}");
        assert_parse("if (true) { foo; } else bar;", "if(true){foo;}else bar;");
    }

    #[test]
    fn while_statement() {
        assert_parse("while (true) foo;", "while(true)foo;");
        assert_parse("while (true) { foo; }", "while(true){foo;}");
    }
}
