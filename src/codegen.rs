use std::ptr;

use grammar::*;
use grammar::OperatorType::*;

/// The `Generator` is a wrapper around an owned `String` that's used to
/// stringify the AST. There is a bunch of useful methods here to manage
/// things like indentation and automatically producing minified code.
struct Generator {
    pub minify: bool,
    code: Vec<u8>,
    dent: u16,
}

impl Generator {
    pub fn new(minify: bool) -> Self {
        Generator {
            minify: minify,
            code: Vec::with_capacity(128),
            dent: 0,
        }
    }

    #[inline]
    pub fn new_line(&mut self) {
        if !self.minify {
            self.write_char(b'\n');
            for _ in 0..self.dent {
                self.write(b"    ");
            }
        }
    }

    #[inline]
    pub fn write(&mut self, slice: &[u8]) {
        extend_from_slice(&mut self.code, slice);
    }

    #[inline]
    pub fn write_string(&mut self, text: &str) {
        extend_from_slice(&mut self.code, text.as_bytes());
    }

    #[inline]
    pub fn write_min(&mut self, slice: &[u8], minslice: &[u8]) {
        if self.minify {
            self.write(minslice);
        } else {
            self.write(slice);
        }
    }

    #[inline]
    pub fn write_char(&mut self, ch: u8) {
        self.code.push(ch);
    }

    #[inline]
    pub fn write_list<T: Code>(&mut self, items: &Vec<T>) {
        let mut first = true;
        for item in items {
            if first {
                first = false;
            } else {
                self.write_min(b", ", b",");
            }
            item.to_code(self);
        }
    }

    #[inline]
    pub fn write_block<T: Code>(&mut self, items: &Vec<T>) {
        self.indent();
        for item in items {
            self.new_line();
            item.to_code(self);
        }
        self.dedent();
        self.new_line();
    }

    pub fn write_declaration_or_expression(&mut self, statement: &Statement) {
        match *statement {
            Statement::VariableDeclaration {
                ref kind,
                ref declarators,
            } => {
                kind.to_code(self);
                self.write_char(b' ');
                self.write_list(declarators);
            },

            Statement::Expression {
                ref value,
            } => {
                value.to_code(self);
            },

            _ => panic!("Invalid AST structure!"),
        }
    }

    #[inline]
    pub fn indent(&mut self) {
        self.dent += 1;
    }

    #[inline]
    pub fn dedent(&mut self) {
        self.dent -= 1;
    }

    #[inline]
    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

/// The `Code` trait provides an interface to pieces of grammar, that allows
/// to efficiently write characters and string slices to the code `Generator`.
trait Code {
    fn to_code(&self, gen: &mut Generator);
}

impl Code for String {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write_string(self);
    }
}

impl<T: Code> Code for Option<T> {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Some(ref value) => value.to_code(gen),
            None            => {}
        }
    }
}

impl Code for OperatorType {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write(match *self {
            FatArrow         => b"=>",
            Accessor         => b".",
            New              => b"new",
            Increment        => b"++",
            Decrement        => b"--",
            LogicalNot       => b"!",
            BitwiseNot       => b"~",
            Typeof           => b"typeof",
            Void             => b"void",
            Delete           => b"delete",
            Multiplication   => b"*",
            Division         => b"/",
            Remainder        => b"%",
            Exponent         => b"**",
            Addition         => b"+",
            Substraction     => b"-",
            BitShiftLeft     => b"<<",
            BitShiftRight    => b">>",
            UBitShiftRight   => b">>>",
            Lesser           => b"<",
            LesserEquals     => b"<=",
            Greater          => b">",
            GreaterEquals    => b">=",
            Instanceof       => b"instanceof",
            In               => b"in",
            StrictEquality   => b"===",
            StrictInequality => b"!==",
            Equality         => b"==",
            Inequality       => b"!=",
            BitwiseAnd       => b"&",
            BitwiseXor       => b"^",
            BitwiseOr        => b"|",
            LogicalAnd       => b"&&",
            LogicalOr        => b"||",
            Conditional      => b"?",
            Assign           => b"=",
            AddAssign        => b"+=",
            SubstractAssign  => b"-=",
            ExponentAssign   => b"**=",
            MultiplyAssign   => b"*=",
            DivideAssign     => b"/=",
            RemainderAssign  => b"%=",
            BSLAssign        => b"<<=",
            BSRAssign        => b">>=",
            UBSRAssign       => b">>>=",
            BitAndAssign     => b"&=",
            BitXorAssign     => b"^=",
            BitOrAssign      => b"|=",
            Spread           => b"...",
        });
    }
}

impl Code for LiteralValue {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            LiteralUndefined          => gen.write_min(b"undefined", b"void 0"),
            LiteralNull               => gen.write(b"null"),
            LiteralTrue               => gen.write_min(b"true", b"!0",),
            LiteralFalse              => gen.write_min(b"false", b"!1"),
            LiteralInteger(ref num)   => gen.write(&num.to_string().as_bytes()),
            LiteralFloat(ref num)     => gen.write(&num.to_string().as_bytes()),
            LiteralString(ref string) => gen.write_string(string),
        }
    }
}

impl Code for ObjectMember {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            ObjectMember::Shorthand {
                ref key
            } => gen.write_string(key),

            ObjectMember::Literal {
                ref key,
                ref value,
            } => {
                gen.write_string(key);
                gen.write_min(b": ", b":");
                value.to_code(gen);
            },

            ObjectMember::Computed {
                ref key,
                ref value,
            } => {
                gen.write_char(b'[');
                key.to_code(gen);
                gen.write_min(b"]: ", b"]:");
                value.to_code(gen);
            },

            ObjectMember::Method {
                ref name,
                ref params,
                ref body,
            } => {
                gen.write_string(name);
                gen.write_char(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_char(b'}');
            },

            ObjectMember::ComputedMethod {
                ref name,
                ref params,
                ref body,
            } => {
                gen.write_char(b'[');
                name.to_code(gen);
                gen.write(b"](");
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_char(b'}');
            },
        }
    }
}

impl Code for MemberKey {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            MemberKey::Literal(ref string) => {
                gen.write_char(b'.');
                gen.write_string(string);
            },
            MemberKey::Computed(ref expr)  => {
                gen.write_char(b'[');
                expr.to_code(gen);
                gen.write_char(b']');
            },
        }
    }
}

impl Code for Parameter {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        let Parameter { ref name } = *self;
        gen.write_string(name);
    }
}

impl Code for Expression {
    fn to_code(&self, gen: &mut Generator) {
        match *self {

            Expression::This => gen.write(b"this"),

            Expression::Identifier(ref ident) => gen.write_string(ident),

            Expression::Literal(ref literal)  => literal.to_code(gen),

            Expression::Array(ref items) => {
                gen.write_char(b'[');
                gen.write_list(items);
                gen.write_char(b']');
            },

            Expression::Sequence(ref items) => {
                gen.write_char(b'(');
                gen.write_list(items);
                gen.write_char(b')');
            },

            Expression::Object(ref members) => {
                gen.write_char(b'{');
                gen.indent();
                let mut first = true;
                for member in members {
                    if first {
                        first = false;
                    } else {
                        gen.write_char(b',');
                    }
                    gen.new_line();
                    member.to_code(gen);
                }
                gen.dedent();
                gen.new_line();
                gen.write_char(b'}');
            },

            Expression::Member {
                ref object,
                ref property,
            } => {
                object.to_code(gen);
                property.to_code(gen);
            },

            Expression::Call {
                ref callee,
                ref arguments,
            } => {
                callee.to_code(gen);
                gen.write_char(b'(');
                gen.write_list(arguments);
                gen.write_char(b')');
            },

            Expression::Binary {
                ref left,
                ref operator,
                ref right,
            } => {
                if left.binding_power() < self.binding_power() {
                    gen.write_char(b'(');
                    left.to_code(gen);
                    gen.write_char(b')');
                } else {
                    left.to_code(gen);
                }
                gen.write_min(b" ", b"");
                operator.to_code(gen);
                gen.write_min(b" ", b"");
                right.to_code(gen);
            },

            Expression::Prefix {
                ref operator,
                ref operand,
            } => {
                operator.to_code(gen);
                operand.to_code(gen);
            },

            Expression::Postfix {
                ref operator,
                ref operand,
            } => {
                operand.to_code(gen);
                operator.to_code(gen);
            },

            Expression::Conditional {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                test.to_code(gen);
                gen.write_min(b" ? ", b"?");
                consequent.to_code(gen);
                gen.write_min(b" : ", b":");
                alternate.to_code(gen);
            },

            Expression::ArrowFunction {
                ref params,
                ref body,
            } => {
                if params.len() == 1 {
                    params[0].to_code(gen);
                } else {
                    gen.write_char(b'(');
                    gen.write_list(params);
                    gen.write_char(b')');
                }
                gen.write_min(b" => ", b"=>");
                match **body {
                    Statement::Expression {
                        ref value,
                    } => value.to_code(gen),
                    _ => body.to_code(gen),
                }
            },

            Expression::Function {
                ref name,
                ref params,
                ref body,
            } => {
                gen.write(b"function");
                if let Some(ref name) = *name {
                    gen.write_char(b' ');
                    gen.write_string(name);
                } else {
                    gen.write_min(b" ", b"");
                }
                gen.write_char(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_char(b'}');
            },

            // _ => gen.write_char('💀'),
        }
    }
}

impl Code for VariableDeclarationKind {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write(match *self {
            VariableDeclarationKind::Var   => b"var",
            VariableDeclarationKind::Let   => b"let",
            VariableDeclarationKind::Const => b"const",
        })
    }
}

impl Code for ClassMember {
    fn to_code(&self, gen: &mut Generator) {
        match *self {

            ClassMember::Constructor {
                ref params,
                ref body,
            } => {
                gen.write(b"constructor(");
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_char(b'}');
            },

            ClassMember::Method {
                is_static,
                ref name,
                ref params,
                ref body,
            } => {
                if is_static {
                    gen.write(b"static ");
                }
                gen.write_string(name);
                gen.write_char(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_char(b'}');
            },

            ClassMember::Property {
                is_static,
                ref name,
                ref value,
            } => {
                if is_static {
                    gen.write(b"static ");
                }
                gen.write_string(name);
                gen.write_min(b" = ", b"=");
                value.to_code(gen);
                gen.write_char(b';');
            }
        }
    }
}

impl Code for VariableDeclarator {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write_string(&self.name);
        if self.value.is_some() {
            gen.write_min(b" = ", b"=");
            self.value.to_code(gen);
        }
    }
}

impl Code for Statement {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Statement::Labeled {
                ref label,
                ref body,
            } => {
                gen.write_string(label);
                gen.write_min(b": ", b":");
                body.to_code(gen);
            },

            Statement::Block {
                ref body,
            } => {
                gen.write_char(b'{');
                gen.write_block(body);
                gen.write_char(b'}');
            },

            Statement::Expression {
                ref value,
            } => {
                value.to_code(gen);
                gen.write_char(b';');
            },

            Statement::Return {
                ref value,
            } => {
                gen.write(b"return");
                if let Some(ref value) = *value {
                    gen.write_char(b' ');
                    value.to_code(gen);
                }
                gen.write_char(b';');
            },

            Statement::Break {
                ref label,
            } => {
                gen.write(b"break");
                if let Some(ref label) = *label {
                    gen.write_char(b' ');
                    gen.write_string(label);
                }
                gen.write_char(b';');
            },

            Statement::VariableDeclaration {
                ref kind,
                ref declarators,
            } => {
                kind.to_code(gen);
                gen.write_char(b' ');
                gen.write_list(declarators);
                gen.write_char(b';');
            },

            Statement::Function {
                ref name,
                ref params,
                ref body,
            } => {
                gen.new_line();
                gen.write(b"function ");
                gen.write_string(name);
                gen.write_char(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_char(b'}');
                gen.new_line();
            },

            Statement::If {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                gen.write_min(b"if (", b"if(");
                test.to_code(gen);
                gen.write_min(b") ", b")");
                consequent.to_code(gen);

                if let Some(ref alternate) = *alternate {
                    gen.write(b" else ");
                    alternate.to_code(gen);
                };
            },

            Statement::While {
                ref test,
                ref body,
            } => {
                gen.write_min(b"while (", b"while(");
                test.to_code(gen);
                gen.write_min(b") ", b")");
                body.to_code(gen);
            },

            Statement::For {
                ref init,
                ref test,
                ref update,
                ref body,
            } => {
                gen.write_min(b"for (", b"for(");
                if let Some(ref init) = *init {
                    gen.write_declaration_or_expression(init);
                }
                gen.write_min(b"; ", b";");
                test.to_code(gen);
                gen.write_min(b"; ", b";");
                update.to_code(gen);
                gen.write_min(b") ", b")");
                body.to_code(gen);
            },

            Statement::ForIn {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_min(b"for (", b"for(");
                gen.write_declaration_or_expression(left);
                gen.write(b" in ");
                right.to_code(gen);
                gen.write_min(b") ", b")");
                body.to_code(gen);
            },

            Statement::ForOf {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_min(b"for (", b"for(");
                gen.write_declaration_or_expression(left);
                gen.write(b" of ");
                right.to_code(gen);
                gen.write_min(b") ", b")");
                body.to_code(gen);
            },

            Statement::Class {
                ref name,
                ref extends,
                ref body,
            } => {
                gen.new_line();
                gen.write(b"class ");
                gen.write_string(name);
                if let &Some(ref super_class) = extends {
                    gen.write(b" extends ");
                    gen.write_string(super_class);
                }
                gen.write_min(b" {", b"{");
                gen.write_block(body);
                gen.write_char(b'}');
                gen.new_line();
            },
        }
    }
}

pub fn generate_code(program: Program, minify: bool) -> String {
    let mut gen = Generator::new(minify);

    for statement in program.body {
        statement.to_code(&mut gen);
        gen.new_line();
    }

    gen.consume()
}


// From: https://github.com/dtolnay/fastwrite/blob/master/src/lib.rs#L68
//
// LLVM is not able to lower `Vec::extend_from_slice` into a memcpy, so this
// helps eke out that last bit of performance.
#[inline]
fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ptr::copy_nonoverlapping(
            src.as_ptr(),
            dst.as_mut_ptr().offset(dst_len as isize),
            src_len);
    }
}

