use std::ptr;
use std::io::Write;

use grammar::*;
use operator::OperatorKind;

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
            self.write_byte(b'\n');
            for _ in 0..self.dent {
                self.write_bytes(b"    ");
            }
        }
    }

    #[inline]
    pub fn write<T: Code>(&mut self, item: &T) {
        item.to_code(self);
    }

    #[inline]
    pub fn write_byte(&mut self, ch: u8) {
        self.code.push(ch);
    }

    #[inline]
    pub fn write_bytes(&mut self, slice: &[u8]) {
        extend_from_slice(&mut self.code, slice);
    }

    #[inline]
    pub fn write_min(&mut self, slice: &[u8], minslice: &[u8]) {
        if self.minify {
            self.write_bytes(minslice);
        } else {
            self.write_bytes(slice);
        }
    }

    #[inline]
    pub fn write_list<T: Code>(&mut self, items: &Vec<T>) {
        let mut iter = items.iter();

        for item in iter.next() {
            self.write(item);
        }

        for item in iter {
            self.write_min(b", ", b",");
            self.write(item);
        }
    }

    #[inline]
    pub fn write_block<T: Code>(&mut self, items: &Vec<T>) {
        if items.len() == 0 {
            return;
        }

        self.indent();
        for item in items {
            self.new_line();
            self.write(item);
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
                self.write(kind);
                self.write_byte(b' ');
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

impl Code for Ident {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Ident::Insitu(s)     => "INSITU",
            Ident::Static(s)     => s,
            Ident::Inline(ref s) => s.as_str(),
        }.to_code(gen);
    }
}

impl Code for Slice {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        "INSITU".to_code(gen);
    }
}

impl Code for u64 {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        write!(&mut gen.code, "{}", *self).expect("Can't fail on a Vec");
    }
}

impl<T: Code> Code for Box<T> {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write(self.as_ref());
    }
}

impl<'a> Code for &'a str {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        extend_from_slice(&mut gen.code, self.as_bytes());
    }
}

impl<T: Code> Code for Option<T> {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Some(ref value) => value.to_code(gen),
            None            => {}
        }
    }
}

impl Code for OperatorKind {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write_bytes(self.as_str().as_bytes());
    }
}

fn write_quasi(gen: &mut Generator, quasi: Slice) {
    gen.write_byte(b'"');

    gen.write_bytes(b"INSITU");

    // let mut iter = quasi.as_str().bytes();

    // while let Some(byte) = iter.next() {
    //     match byte {
    //         b'\r' => {},
    //         b'\n' => gen.write_bytes(b"\\n"),
    //         b'"'  => gen.write_bytes(b"\\\""),
    //         b'\\' => {
    //             if let Some(follow) = iter.next() {
    //                 match follow {
    //                     b'`'  => gen.write_byte(b'`'),
    //                     b'\n' => {},
    //                     b'\r' => {},
    //                     _     => {
    //                         gen.write_byte(b'\\');
    //                         gen.write_byte(follow);
    //                     }
    //                 }
    //             }
    //         },
    //         _ => gen.write_byte(byte),
    //     }
    // }

    gen.write_byte(b'"');
}

impl Code for Value {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Value::Undefined           => gen.write_min(b"undefined", b"void 0"),
            Value::Null                => gen.write_bytes(b"null"),
            Value::Boolean(val)        => if val { gen.write_min(b"true", b"!0",) } else { gen.write_min(b"false", b"!1") },
            Value::Binary(ref num)     => gen.write(num),
            Value::Number(ref num)     => gen.write(num),
            Value::String(ref string)  => gen.write(string),
            Value::RawQuasi(ref quasi) => write_quasi(gen, *quasi),
        }
    }
}

impl Code for ObjectMember {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            ObjectMember::Shorthand {
                ref key
            } => gen.write(key),

            ObjectMember::Value {
                ref key,
                ref value,
            } => {
                gen.write(key);
                gen.write_min(b": ", b":");
                gen.write(value);
            },

            ObjectMember::Method {
                ref key,
                ref params,
                ref body,
            } => {
                gen.write(key);
                gen.write_byte(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_byte(b'}');
            },
        }
    }
}

impl Code for ObjectKey {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            ObjectKey::Computed (ref expression) => {
                gen.write_byte(b'[');
                gen.write(expression);
                gen.write_byte(b']');
            },

            ObjectKey::Literal (ref slice) => {
                gen.write(slice);
            },

            ObjectKey::Binary (ref num) => {
                gen.write(num)
            }
        }
    }
}

impl Code for Parameter {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write(&self.name);
        if let Some(ref expression) = self.default {
            gen.write_min(b" = ", b"=");
            gen.write(expression);
        }
    }
}

impl Code for Expression {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Expression::Void => {},

            Expression::This => gen.write_bytes(b"this"),

            Expression::Identifier(ref ident) => gen.write(ident),

            Expression::Literal(ref literal)  => gen.write(literal),

            Expression::Template {
                ref tag,
                ref expressions,
                ref quasis,
            } => {
                gen.write(tag);
                gen.write_byte(b'`');

                match quasis.len() {
                    0 => panic!("Must have at least one quasi"),
                    1 => {
                        gen.write(&quasis[0]);
                    },
                    _ => {
                        let last = quasis.len() - 1;
                        let iter = quasis[..last].iter().zip(expressions);

                        for (quasi, expression) in iter {
                            gen.write(quasi);
                            gen.write_min(b"${ ", b"${");
                            gen.write(expression);
                            gen.write_min(b" }", b"}");
                        }

                        gen.write(&quasis[last]);
                    }
                }

                gen.write_byte(b'`');
            },

            Expression::RegEx{ ref pattern, ref flags } => {
                gen.write_byte(b'/');
                gen.write(pattern);
                gen.write_byte(b'/');
                gen.write(flags);
            },

            Expression::Array(ref items) => {
                gen.write_byte(b'[');
                gen.write_list(items);

                // Add dangling comma if the array ends with a spare element
                if let Some(&Expression::Void) = items.iter().rev().next() {
                    gen.write_byte(b',');
                }

                gen.write_byte(b']');
            },

            Expression::Sequence(ref items) => {
                gen.write_byte(b'(');
                gen.write_list(items);
                gen.write_byte(b')');
            },

            Expression::Object(ref members) => {
                if members.len() == 0 {
                    gen.write_bytes(b"{}");
                    return;
                }

                gen.write_byte(b'{');
                gen.indent();

                let mut iter = members.iter();

                for member in iter.next() {
                    gen.new_line();
                    gen.write(member);
                }

                for member in iter {
                    gen.write_byte(b',');
                    gen.new_line();
                    gen.write(member);
                }

                gen.dedent();
                gen.new_line();
                gen.write_byte(b'}');
            },

            Expression::Member {
                ref object,
                ref property,
            } => {
                gen.write(object);
                gen.write_byte(b'.');
                gen.write(property);
            },

            Expression::ComputedMember {
                ref object,
                ref property,
            } => {
                gen.write(object);
                gen.write_byte(b'[');
                gen.write(property);
                gen.write_byte(b']');
            },

            Expression::Call {
                ref callee,
                ref arguments,
            } => {
                gen.write(callee);
                gen.write_byte(b'(');
                gen.write_list(arguments);
                gen.write_byte(b')');
            },

            Expression::Binary {
                ref operator,
                ref left,
                ref right,
                ..
            } => {
                let bp = self.binding_power();
                let spacing = operator.is_word() || !gen.minify;

                if left.binding_power() < bp {
                    gen.write_byte(b'(');
                    gen.write(left);
                    gen.write_byte(b')');
                } else {
                    gen.write(left);
                }

                if spacing {
                    gen.write_byte(b' ');
                }
                gen.write(operator);
                if spacing {
                    gen.write_byte(b' ');
                }

                if right.needs_parens(bp) {
                    gen.write_byte(b'(');
                    gen.write(right);
                    gen.write_byte(b')');
                } else {
                    gen.write(right);
                }
            },

            Expression::Prefix {
                ref operator,
                ref operand,
            } => {
                gen.write(operator);
                if operator.is_word() {
                    gen.write_byte(b' ');
                }
                gen.write(operand);
            },

            Expression::Postfix {
                ref operator,
                ref operand,
            } => {
                gen.write(operand);
                gen.write(operator);
            },

            Expression::Conditional {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                gen.write(test);
                gen.write_min(b" ? ", b"?");
                gen.write(consequent);
                gen.write_min(b" : ", b":");
                gen.write(alternate);
            },

            Expression::ArrowFunction {
                ref params,
                ref body,
            } => {
                if params.len() == 1 {
                    gen.write(&params[0]);
                } else {
                    gen.write_byte(b'(');
                    gen.write_list(params);
                    gen.write_byte(b')');
                }
                gen.write_min(b" => ", b"=>");
                match **body {
                    Statement::Expression {
                        ref value,
                    } => gen.write(value),
                    _ => gen.write(body),
                }
            },

            Expression::Function {
                ref name,
                ref params,
                ref body,
            } => {
                gen.write_bytes(b"function");
                if let Some(ref name) = *name {
                    gen.write_byte(b' ');
                    gen.write(name);
                } else {
                    gen.write_min(b" ", b"");
                }
                gen.write_byte(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_byte(b'}');
            },

            Expression::Class {
                ref name,
                ref extends,
                ref body,
            } => {
                gen.new_line();
                if let &Some(ref name) = name {
                    gen.write_bytes(b"class ");
                    gen.write(name);
                } else {
                    gen.write_bytes(b"class");
                }
                if let &Some(ref super_class) = extends {
                    gen.write_bytes(b" extends ");
                    gen.write(super_class);
                }
                gen.write_min(b" {", b"{");
                gen.write_block(body);
                gen.write_byte(b'}');
                gen.new_line();
            },
        }
    }
}

impl Code for VariableDeclarationKind {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write_bytes(match *self {
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
                gen.write_bytes(b"constructor(");
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_byte(b'}');
            },

            ClassMember::Method {
                is_static,
                ref key,
                ref params,
                ref body,
            } => {
                if is_static {
                    gen.write_bytes(b"static ");
                }
                gen.write(key);
                gen.write_byte(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_byte(b'}');
            },

            ClassMember::Property {
                is_static,
                ref key,
                ref value,
            } => {
                if is_static {
                    gen.write_bytes(b"static ");
                }
                gen.write(key);
                gen.write_min(b" = ", b"=");
                gen.write(value);
                gen.write_byte(b';');
            }
        }
    }
}

impl Code for ClassKey {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            ClassKey::Literal(ref name)  => gen.write(name),

            ClassKey::Number(ref name)   => gen.write(name),

            ClassKey::Binary(ref num)    => gen.write(num),

            ClassKey::Computed(ref expr) => {
                gen.write_byte(b'[');
                gen.write(expr);
                gen.write_byte(b']');
            }
        }
    }
}

impl Code for VariableDeclarator {
    #[inline]
    fn to_code(&self, gen: &mut Generator) {
        gen.write(&self.name);
        if let Some(ref value) = self.value {
            gen.write_min(b" = ", b"=");
            gen.write(value);
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
                gen.write(label);
                gen.write_min(b": ", b":");
                gen.write(body);
            },

            Statement::Block {
                ref body,
            } => {
                gen.write_byte(b'{');
                gen.write_block(body);
                gen.write_byte(b'}');
            },

            Statement::Transparent {
                ref body,
            } => {
                let mut iter = body.iter();

                for statement in iter.next() {
                    gen.write(statement);
                }

                for statement in iter {
                    gen.new_line();
                    gen.write(statement);
                }
            },

            Statement::Empty => {
                gen.write_byte(b';');
            }

            Statement::Expression {
                ref value,
            } => {
                if value.is_allowed_as_bare_statement() {
                    gen.write(value);
                } else {
                    gen.write_byte(b'(');
                    gen.write(value);
                    gen.write_byte(b')');
                }
                gen.write_byte(b';');
            },

            Statement::Return {
                ref value,
            } => {
                gen.write_bytes(b"return");
                if let Some(ref value) = *value {
                    gen.write_byte(b' ');
                    gen.write(value);
                }
                gen.write_byte(b';');
            },

            Statement::Break {
                ref label,
            } => {
                gen.write_bytes(b"break");
                if let Some(ref label) = *label {
                    gen.write_byte(b' ');
                    gen.write(label);
                }
                gen.write_byte(b';');
            },

            Statement::VariableDeclaration {
                ref kind,
                ref declarators,
            } => {
                gen.write(kind);
                gen.write_byte(b' ');
                gen.write_list(declarators);
                gen.write_byte(b';');
            },

            Statement::Function {
                ref name,
                ref params,
                ref body,
            } => {
                gen.new_line();
                gen.write_bytes(b"function ");
                gen.write(name);
                gen.write_byte(b'(');
                gen.write_list(params);
                gen.write_min(b") {", b"){");
                gen.write_block(body);
                gen.write_byte(b'}');
                gen.new_line();
            },

            Statement::Class {
                ref name,
                ref extends,
                ref body,
            } => {
                gen.new_line();
                gen.write_bytes(b"class ");
                gen.write(name);
                if let &Some(ref super_class) = extends {
                    gen.write_bytes(b" extends ");
                    gen.write(super_class);
                }
                gen.write_min(b" {", b"{");
                gen.write_block(body);
                gen.write_byte(b'}');
                gen.new_line();
            },

            Statement::If {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                gen.write_min(b"if (", b"if(");
                gen.write(test);
                gen.write_min(b") ", b")");
                gen.write(consequent);

                if let Some(ref alternate) = *alternate {
                    gen.write_bytes(b" else ");
                    gen.write(alternate);
                }
            },

            Statement::While {
                ref test,
                ref body,
            } => {
                gen.write_min(b"while (", b"while(");
                gen.write(test);
                gen.write_min(b") ", b")");
                gen.write(body);
            },

            Statement::Do {
                ref test,
                ref body
            } => {
                gen.write_bytes(b"do ");
                gen.write(body);
                gen.write_bytes(b"while (");
                gen.write(test);
                gen.write_byte(b')');
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
                gen.write(test);
                gen.write_min(b"; ", b";");
                gen.write(update);
                gen.write_min(b") ", b")");
                gen.write(body);
            },

            Statement::ForIn {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_min(b"for (", b"for(");
                gen.write_declaration_or_expression(left);
                gen.write_bytes(b" in ");
                gen.write(right);
                gen.write_min(b") ", b")");
                gen.write(body);
            },

            Statement::ForOf {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_min(b"for (", b"for(");
                gen.write_declaration_or_expression(left);
                gen.write_bytes(b" of ");
                gen.write(right);
                gen.write_min(b") ", b")");
                gen.write(body);
            },

            Statement::Throw {
                ref value,
            } => {
                gen.write_bytes(b"throw ");
                gen.write(value);
                gen.write_byte(b';');
            },

            Statement::Try {
                ref body,
                ref error,
                ref handler,
            } => {
                gen.write_min(b"try ", b"try");
                gen.write(body);
                gen.write_min(b" catch (", b"catch(");
                gen.write(error);
                gen.write_min(b") ", b")");
                gen.write(handler);
            }
        }
    }
}

pub fn generate_code(program: &Program, minify: bool) -> String {
    let mut gen = Generator::new(minify);

    for statement in &program.body {
        gen.write(statement);
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

