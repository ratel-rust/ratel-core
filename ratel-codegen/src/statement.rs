use ratel::ast::{Statement, Declarator, DeclarationKind};
use ratel::ast::statement::*;

use {ToCode, Generator};


impl<'ast, G: Generator> ToCode<G> for Statement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ratel::ast::Statement::*;

        match *self {
            Empty => {},
            Expression(ref expression) => {
                if expression.is_allowed_as_bare_statement() {
                    gen.write(expression);
                } else {
                    gen.write_byte(b'(');
                    gen.write(expression);
                    gen.write_byte(b')');
                }
                gen.write_byte(b';');
            },
            Declaration(ref declaration) => {
                gen.write(declaration);
                gen.write_byte(b';');
            },
            Return(ref return_statement) => gen.write(return_statement),
            Break(ref break_statement)   => gen.write(break_statement),
            Throw(ref throw)             => gen.write(throw),
            If(ref if_statement)         => gen.write(if_statement),
            While(ref while_statement)   => gen.write(while_statement),
            Do(ref do_statement)         => gen.write(do_statement),
            For(ref for_statement)       => gen.write(for_statement),
            ForIn(ref for_in)            => gen.write(for_in),
            ForOf(ref for_of)            => gen.write(for_of),
            Try(ref try)                 => gen.write(try),
            Labeled(ref labeled)         => gen.write(labeled),
            Block(ref block)             => gen.write(block),
            Function(ref function)       => gen.write(function),
            Class(ref class)             => gen.write(class),
            Continue(ref cont)           => gen.write(cont),
            Switch(ref switch)           => gen.write(switch)
        }
    }
}

impl<G: Generator> ToCode<G> for DeclarationKind {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ratel::ast::DeclarationKind::*;

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
        gen.write(&self.id);

        if let Some(ref init) = self.init {
            gen.write_pretty(b' ');
            gen.write_byte(b'=');
            gen.write_pretty(b' ');
            gen.write(init);
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for DeclarationStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.kind);
        gen.write_list(&self.declarators);
    }
}

impl<'ast, G: Generator> ToCode<G> for ReturnStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match self.value {
            Some(ref value) => {
                gen.write_bytes(b"return ");
                gen.write(value);
                gen.write_byte(b';');
            },
            None => gen.write_bytes(b"return;")
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for BreakStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match self.label {
            Some(ref label) => {
                gen.write_bytes(b"break ");
                gen.write(label);
                gen.write_byte(b';');
            },
            None => gen.write_bytes(b"break;")
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for ThrowStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"throw ");
        gen.write(&self.value);
        gen.write_byte(b';');
    }
}

impl<'ast, G: Generator> ToCode<G> for IfStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"if");
        gen.write_pretty(b' ');
        gen.write_byte(b'(');
        gen.write(&self.test);
        gen.write_byte(b')');
        gen.write_pretty(b' ');
        gen.write(&self.consequent);

        if let Some(ref alternate) = self.alternate {
            if self.consequent.is_block() {
                gen.write_pretty(b' ');
            } else {
                gen.write_byte(b' ');
            }
            gen.write_bytes(b"else");
            if alternate.is_block() {
                gen.write_pretty(b' ');
            } else {
                gen.write_byte(b' ');
            }
            gen.write(alternate);
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for WhileStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"while");
        gen.write_pretty(b' ');
        gen.write_byte(b'(');
        gen.write(&self.test);
        gen.write_byte(b')');
        gen.write_pretty(b' ');
        gen.write(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for DoStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"do");
        if self.body.is_block() {
            gen.write_pretty(b' ');
        } else {
            gen.write_byte(b' ');
        }
        gen.write(&self.body);
        gen.write_bytes(b"while");
        gen.write_pretty(b' ');
        gen.write_byte(b'(');
        gen.write(&self.test);
        gen.write_byte(b')');
    }
}

impl<'ast, G: Generator> ToCode<G> for ForInit<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match *self {
            ForInit::Declaration(ref declaration) => gen.write(declaration),
            ForInit::Expression(ref expression) => gen.write(expression),
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for ForStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"for");
        gen.write_pretty(b' ');
        gen.write_byte(b'(');
        gen.write(&self.init);
        gen.write_byte(b';');
        gen.write_pretty(b' ');
        gen.write(&self.test);
        gen.write_byte(b';');
        gen.write_pretty(b' ');
        gen.write(&self.update);
        gen.write_byte(b')');
        gen.write_pretty(b' ');
        gen.write(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for ForInStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"for");
        gen.write_pretty(b' ');
        gen.write_byte(b'(');
        gen.write(&self.left);
        gen.write_bytes(b" in ");
        gen.write(&self.right);
        gen.write_byte(b')');
        gen.write_pretty(b' ');
        gen.write(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for ForOfStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"for");
        gen.write_pretty(b' ');
        gen.write_byte(b'(');
        gen.write(&self.left);
        gen.write_bytes(b" of ");
        gen.write(&self.right);
        gen.write_byte(b')');
        gen.write_pretty(b' ');
        gen.write(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for TryStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"try");
        gen.write_pretty(b' ');
        gen.write(&self.block);
        if let Some(ref handler) = self.handler {
            gen.write_pretty(b' ');
            gen.write_bytes(b"catch");
            gen.write_pretty(b' ');
            gen.write_byte(b'(');
            gen.write(&handler.param);
            gen.write_byte(b')');
            gen.write_pretty(b' ');
            gen.write(&handler.body);
        }
        if let Some(ref finalizer) = self.finalizer {
            gen.write_pretty(b' ');
            gen.write(finalizer);
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for LabeledStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.label);
        gen.write_byte(b':');
        gen.write_pretty(b' ');
        gen.write(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for ContinueStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match self.label {
            Some(ref label) => {
                gen.write_bytes(b"continue ");
                gen.write(label);
                gen.write_byte(b';');
            },
            None => gen.write_bytes(b"continue;")
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for SwitchStatement<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"switch");
        gen.write_pretty(b' ');
        gen.write_byte(b'(');
        gen.write(&self.discriminant);
        gen.write_byte(b')');
        gen.write(&self.cases);
    }
}

impl<'ast, G: Generator> ToCode<G> for SwitchCase<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match self.test {
            Some(ref test) => {
                gen.write_bytes(b"case ");
                gen.write(test);
                gen.write_byte(b':');
            }
            None => gen.write_bytes(b"default:")
        }
        gen.write_block(&self.consequent);
    }
}

#[cfg(test)]
mod test {
    use assert_min;

    #[test]
    fn block_statement() {
        assert_min("{}", "{}");
        assert_min("{foo;}", "{foo;}");
    }

    #[test]
    fn labeled_statement() {
        assert_min("foo: {}", "foo:{}");
        assert_min("foo: bar;", "foo:bar;");
    }

    #[test]
    fn function_statement() {
        assert_min("function foo() {}", "function foo(){}");
    }

    #[test]
    fn declaration_statement() {
        assert_min("var foo;", "var foo;");
        assert_min("let foo;", "let foo;");
        assert_min("const foo;", "const foo;");
        assert_min("var foo = 10;", "var foo=10;");
        assert_min("let foo = 10;", "let foo=10;");
        assert_min("const foo = 10;", "const foo=10;");
        assert_min("var foo, bar;", "var foo,bar;");
        assert_min("let foo, bar;", "let foo,bar;");
        assert_min("const foo, bar;", "const foo,bar;");
        assert_min("var foo = 10, bar = 20;", "var foo=10,bar=20;");
        assert_min("let foo = 10, bar = 20;", "let foo=10,bar=20;");
        assert_min("const foo = 10, bar = 20;", "const foo=10,bar=20;");
        assert_min("const a = {...foo};", "const a={...foo};");
    }

    #[test]
    fn if_statement() {
        assert_min("if (true) foo;", "if(true)foo;");
        assert_min("if (true) { foo; }", "if(true){foo;}");
        assert_min("if (true) foo; else bar;", "if(true)foo; else bar;");
        assert_min("if (true) { foo; } else { bar; }", "if(true){foo;}else{bar;}");
        assert_min("if (true) foo; else { bar; }", "if(true)foo; else{bar;}");
        assert_min("if (true) { foo; } else bar;", "if(true){foo;}else bar;");
    }

    #[test]
    fn while_statement() {
        assert_min("while (true) foo;", "while(true)foo;");
        assert_min("while (true) { foo; }", "while(true){foo;}");
    }

    #[test]
    fn do_statement() {
        assert_min("do { foo; } while (true)", "do{foo;}while(true)");
        assert_min("do foo; while (true)", "do foo;while(true)");
    }

    #[test]
    fn for_statement() {
        assert_min("for (var i = 0; i < 10; i++) {}", "for(var i=0;i<10;i++){}");
        assert_min("for (i = 0; i < 10; i++) {}", "for(i=0;i<10;i++){}");
        assert_min("for (;;) {}", "for(;;){}");
        assert_min("for (foo in bar){}", "for(foo in bar){}");
        assert_min("for (let foo in bar){}", "for(let foo in bar){}");
        assert_min("for (foo of bar){}", "for(foo of bar){}");
        assert_min("for (let foo of bar){}", "for(let foo of bar){}");
    }
}
