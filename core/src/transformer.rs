use std::mem;

use grammar::*;
use grammar::ClassMember::*;
use operator::OperatorKind::*;

pub struct Settings {
    pub transform_block_scope: bool,
    pub transform_arrow: bool,
    pub transform_object: bool,
    pub transform_exponentation: bool,
    pub transform_class_properties: bool,
    pub transform_class: bool,
    pub transform_template_strings: bool,
}

trait Take {
    fn take(&mut self) -> Self;
}

impl<T> Take for Vec<T> {
    #[inline]
    fn take(&mut self) -> Self {
        mem::replace(self, Vec::new())
    }
}

impl Take for Expression {
    #[inline]
    fn take(&mut self) -> Self {
        mem::replace(self, Expression::This)
    }
}

impl Take for ObjectKey {
    #[inline]
    fn take(&mut self) -> Self {
        mem::replace(self, ObjectKey::Binary(0))
    }
}

#[inline]
fn bind_this(function: Expression) -> Expression {
    Expression::call(Expression::member(function, "bind"), vec![Expression::This])
}

impl Settings {
    pub fn target_es5() -> Settings {
        let mut settings = Settings::target_es2015();

        settings.transform_block_scope = true;
        settings.transform_arrow = true;
        settings.transform_object = true;
        settings.transform_class = true;
        settings.transform_template_strings = true;

        settings
    }

    pub fn target_es2015() -> Settings {
        let mut settings = Settings::no_transform();

        settings.transform_exponentation = true;
        settings.transform_class_properties = true;

        settings
    }

    pub fn no_transform() -> Settings {
        Settings {
            transform_block_scope: false,
            transform_arrow: false,
            transform_object: false,
            transform_exponentation: false,
            transform_class_properties: false,
            transform_class: false,
            transform_template_strings: false,
        }
    }
}

/// The `Transformable` trait provides an interface for instances of grammar
/// to alter the AST.
trait Transformable {
    #[inline]
    fn transform(&mut self, _: &Settings) {}

    #[inline]
    fn contains_this(&self) -> bool {
        false
    }
}

impl<T: Transformable> Transformable for Option<T> {
    #[inline]
    fn transform(&mut self, settings: &Settings) {
        if let Some(ref mut value) = *self {
            value.transform(settings);
        }
    }

    #[inline]
    fn contains_this(&self) -> bool {
        match *self {
            Some(ref value) => value.contains_this(),
            _               => false,
        }
    }
}

impl<T: Transformable> Transformable for Box<T> {
    #[inline]
    fn transform(&mut self, settings: &Settings) {
        self.as_mut().transform(settings)
    }

    #[inline]
    fn contains_this(&self) -> bool {
        self.as_ref().contains_this()
    }
}

impl Transformable for Parameter {}

impl Transformable for Expression {
    fn transform(&mut self, settings: &Settings) {
        *self = match *self {
            Expression::ArrowFunction {
                ref mut params,
                ref mut body,
            } => {
                params.transform(settings);
                body.transform(settings);

                // transformation flag check
                if !settings.transform_arrow {
                    return;
                }

                let body = match **body {
                    Statement::Block { ref mut body }   => body.split_off(0),
                    Statement::Expression { ref mut value } => vec![
                        Statement::Return {
                            value: Some(value.take())
                        }
                    ],
                    ref statement => {
                        panic!("Invalid arrow function body {:#?}", statement);
                    }
                };

                let bind = body.contains_this();

                let function = Expression::Function {
                    name: None,
                    params: params.split_off(0),
                    body: body,
                };

                if bind {
                    bind_this(function)
                } else {
                    function
                }
            },

            Expression::Array(ref mut items) => {
                items.transform(settings);
                return;
            },

            Expression::Object(ref mut members) => {
                members.transform(settings);

                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                let mut computed = partition_vec(members, |member| {
                    match member {
                        &ObjectMember::Value {
                            key: ObjectKey::Computed(_),
                            ..
                        } => false,
                        &ObjectMember::Method {
                            key: ObjectKey::Computed(_),
                            ..
                        } => false,
                        _ => true,
                    }
                });

                if computed.is_empty() {
                    return;
                }

                let literal = members.take();

                let mut body = Vec::with_capacity(computed.len() + 2);

                body.push(Statement::VariableDeclaration {
                    kind: VariableDeclarationKind::Var,
                    declarators: vec![
                        VariableDeclarator {
                            name: "___".into(),
                            value: Some(Expression::Object(literal)),
                        }
                    ]
                });

                for member in computed.drain(..) {
                    body.push(match member {
                        ObjectMember::Value {
                            key: ObjectKey::Computed(key),
                            value,
                        } => {
                            Expression::binary(
                                Expression::ComputedMember {
                                    object: Box::new("___".into()),
                                    property: Box::new(key),
                                },
                                Assign,
                                value
                            ).into()
                        }

                        ObjectMember::Method {
                            key: ObjectKey::Computed(key),
                            params,
                            body,
                        } => {
                            Expression::binary(
                                Expression::ComputedMember {
                                    object: Box::new("___".into()),
                                    property: Box::new(key),
                                },
                                Assign,
                                Expression::Function {
                                    name: None,
                                    params: params,
                                    body: body
                                }
                            ).into()
                        }

                        _ => unreachable!()
                    });
                }

                body.push(Statement::Return {
                    value: Some("___".into())
                });

                Expression::call(Expression::Function {
                    name: None,
                    params: Vec::new(),
                    body: body,
                }, Vec::new())
            },

            Expression::Call {
                ref mut callee,
                ref mut arguments,
            } => {
                callee.transform(settings);
                arguments.transform(settings);
                return;
            },

            Expression::Binary {
                ref mut operator,
                ref mut left,
                ref mut right,
                ..
            } => {
                left.transform(settings);
                right.transform(settings);

                if !settings.transform_exponentation {
                    return;
                }

                match *operator {
                    Exponent => Expression::call(
                        Expression::member("Math", "pow"),
                        vec![left.take(), right.take()]
                    ),

                    ExponentAssign => {
                        *operator = Assign;
                        *right = Box::new(Expression::call(
                            Expression::member("Math", "pow"),
                            vec![left.take(), right.take()]
                        ));
                        return;
                    },

                    _ => return,
                }
            },

            Expression::Template {
                ref mut tag,
                ref mut expressions,
                ref mut quasis,
            } => {
                tag.transform(settings);
                expressions.transform(settings);

                if !settings.transform_template_strings {
                    return;
                }

                if let Some(tag) = tag.take() {
                    // Tagged template

                    let mut arguments = Vec::with_capacity(expressions.len() + 1);

                    arguments.push(Expression::Array(
                        quasis.drain(..)
                              .map(|quasi| Expression::Literal(Value::RawQuasi(quasi)))
                              .collect()
                    ));

                    arguments.extend(expressions.drain(..));

                    Expression::Call {
                        callee: tag,
                        arguments: arguments,
                    }
                } else {
                    // Not tagged template

                    let mut quasis = quasis.drain(..);

                    let mut left = Expression::Literal(
                        Value::RawQuasi(quasis.next().expect("Must have first quasi"))
                    );

                    let iter = quasis.zip(expressions.drain(..));

                    for (quasi, expression) in iter {
                        left = Expression::binary(
                            left,
                            Addition,
                            expression.parenthesize()
                        );

                        if quasi.len() == 0 {
                            continue;
                        }

                        left = Expression::binary(
                            left,
                            Addition,
                            Expression::Literal(
                                Value::RawQuasi(quasi)
                            )
                        );
                    }

                    left
                }
            },

            _ => return,
        }
    }

    fn contains_this(&self) -> bool {
        match *self {
            Expression::This => true,

            Expression::Array(ref items) => items.contains_this(),

            Expression::Object(ref members) => members.contains_this(),

            Expression::Member {
                ref object,
                ..
            } => object.contains_this(),

            Expression::Call {
                ref callee,
                ref arguments,
            } => callee.contains_this() || arguments.contains_this(),

            Expression::Binary {
                ref left,
                ref right,
                ..
            } => left.contains_this() || right.contains_this(),

            _ => false,
        }
    }
}

impl Transformable for ObjectKey {
    #[inline]
    fn transform(&mut self, settings: &Settings) {
        match *self {
            ObjectKey::Computed(ref mut expression) => {
                expression.transform(settings);
            },
            _ => {}
        }
    }

    #[inline]
    fn contains_this(&self) -> bool {
        match *self {
            ObjectKey::Computed(ref expression) => {
                expression.contains_this()
            },
            _ => false
        }
    }
}

impl Transformable for ObjectMember {
    fn transform(&mut self, settings: &Settings) {
        *self = match *self {

            ObjectMember::Shorthand {
                ref mut key,
            } => {
                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Value {
                    key: ObjectKey::Literal(*key),
                    value: Expression::Identifier(*key),
                }
            },

            ObjectMember::Value {
                ref mut key,
                ref mut value,
            } => {
                key.transform(settings);
                value.transform(settings);
                return;
            },

            ObjectMember::Method {
                ref mut key,
                ref mut params,
                ref mut body,
            } => {
                key.transform(settings);
                body.transform(settings);
                params.transform(settings);

                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Value {
                    key: key.take(),
                    value: Expression::Function {
                        name: None,
                        params: params.take(),
                        body: body.take(),
                    }
                }
            },
        }
    }

    fn contains_this(&self) -> bool {
        match *self {

            ObjectMember::Value {
                ref key,
                ref value,
            } => key.contains_this() || value.contains_this(),

            _ => false,
        }
    }
}

impl Transformable for ClassMember {
    fn transform(&mut self, settings: &Settings) {
        match *self {
            Constructor {
                ref mut params,
                ref mut body,
            } => {
                params.transform(settings);
                body.transform(settings);
            },

            Method {
                ref mut params,
                ref mut body,
                ..
            } => {
                params.transform(settings);
                body.transform(settings);
            },

            Property {
                ref mut value,
                ..
            } => {
                value.transform(settings);
            }
        }
    }
}

impl Transformable for VariableDeclarator {
    #[inline]
    fn transform(&mut self, settings: &Settings) {
        self.value.transform(settings);
    }

    #[inline]
    fn contains_this(&self) -> bool {
        self.value.contains_this()
    }
}

fn add_props_to_body(body: &mut Vec<Statement>, mut props: Vec<ClassMember>) {
    body.reserve(props.len());

    for prop in props.iter_mut().rev() {
        if let &mut ClassMember::Property {
            // ref is_static,
            ref mut key,
            ref mut value,
            ..
        } = prop {
            body.insert(
                0,
                Expression::binary(
                    class_key_to_member(Expression::This, key),
                    Assign,
                    value.take(),
                ).into()
            );
        }
    }
}

#[inline]
fn class_key_to_member(object: Expression, key: &mut ClassKey) -> Expression {
    let expr = match *key {
        ClassKey::Literal(ref name) => {
            return Expression::member(object, *name);
        },

        ClassKey::Computed(ref mut expr) => expr.take(),

        ClassKey::Number(ref num) => {
            Expression::Literal(Value::Number(*num))
        },

        ClassKey::Binary(ref num) => {
            Expression::Literal(Value::Binary(*num))
        }
    };

    Expression::ComputedMember {
        object: Box::new(object),
        property: Box::new(expr),
    }
}

impl Transformable for Statement {
    fn transform(&mut self, settings: &Settings) {
        *self = match *self {
            Statement::Block {
                ref mut body,
            } => {
                body.transform(settings);
                return;
            },

            Statement::Labeled {
                ref mut body,
                ..
            } => {
                body.transform(settings);
                return;
            },

            Statement::VariableDeclaration {
                ref mut kind,
                ref mut declarators,
            } => {
                declarators.transform(settings);

                // transformation flag check
                if !settings.transform_block_scope {
                    return;
                }

                *kind = VariableDeclarationKind::Var;
                return;
            },

            Statement::Expression {
                ref mut value,
            } => {
                value.transform(settings);
                return;
            },

            Statement::If {
                ref mut test,
                ref mut consequent,
                ref mut alternate,
                ..
            } => {
                test.transform(settings);
                consequent.transform(settings);
                alternate.transform(settings);
                return;
            },

            Statement::Class(ClassDefinition {
                ref name,
                ref mut body,
                ..
            }) => {
                body.transform(settings);

                if !settings.transform_class_properties
                && !settings.transform_class {
                    return;
                }

                let prop_count = body.iter().filter(|member| match **member {
                    ClassMember::Property { .. } => true,
                    _                            => false,
                }).count();

                if prop_count == 0 && !settings.transform_class {
                    return;
                }

                let mut constructor = None;
                let mut methods = Vec::with_capacity(body.len());
                let mut props = Vec::with_capacity(prop_count);

                for member in body.drain(..) {
                    match member {
                        ClassMember::Property {
                            ..
                        } => props.push(member),

                        ClassMember::Constructor {
                            params,
                            body,
                        } => constructor = Some((params, body)),

                        _ => methods.push(member),
                    }
                }

                let (cnst_params, mut cnst_body) = constructor.unwrap_or_else(|| {
                    (Vec::new(), Vec::new())
                });

                add_props_to_body(&mut cnst_body, props);

                // if !settings.transform_class {
                    methods.insert(0, ClassMember::Constructor {
                        params: cnst_params,
                        body: cnst_body,
                    });

                    *body = methods;

                    return;
                // }

                // let constructor = Statement::Function {
                //     name: name,
                //     params: cnst_params,
                //     body: cnst_body,
                // };

                // if methods.len() > 0 {
                //     let mut body = Vec::with_capacity(methods.len() + 1);

                //     body.push(constructor);

                //     for method in methods.iter_mut() {
                //         if let &mut ClassMember::Method {
                //             key: ref mut method_key,
                //             params: ref mut method_params,
                //             body: ref mut method_body,
                //             ref is_static,
                //         } = method {
                //             let reference = if *is_static {
                //                 Expression::Identifier(*name)
                //             } else {
                //                 Expression::member(name, "prototype")
                //             };

                //             body.push(
                //                 Expression::binary(
                //                     class_key_to_member(reference, method_key),
                //                     Assign,
                //                     Expression::Function {
                //                         name: None,
                //                         params: method_params.take(),
                //                         body: method_body.take(),
                //                     },
                //                 ).into()
                //             );
                //         }
                //     }

                //     Statement::Transparent {
                //         body: body
                //     }
                // } else {
                //     constructor
                // }
            }

            _ => return,
        }
    }

    fn contains_this(&self) -> bool {
        match *self {
            Statement::VariableDeclaration {
                ref declarators,
                ..
            } => declarators.contains_this(),

            Statement::Expression {
                ref value,
            } => value.contains_this(),

            Statement::Return {
                value: Some(ref expression)
            } => expression.contains_this(),

            _ => false,
        }
    }
}

impl<T: Transformable> Transformable for Vec<T> {
    #[inline]
    fn transform(&mut self, settings: &Settings) {
        for item in self.iter_mut() {
            item.transform(settings);
        }
    }

    #[inline]
    fn contains_this(&self) -> bool {
        for item in self {
            if item.contains_this() {
                return true;
            }
        }
        return false;
    }
}

#[inline]
fn partition_vec<T, F: Fn(&T) -> bool>(source: &mut Vec<T>, f: F) -> Vec<T> {
    let mut other = Vec::new();
    let indexes = 0 .. source.len();

    for index in indexes.rev() {
        unsafe {
            if !f(source.get_unchecked(index)) {
                let item = source.remove(index);
                other.push(item)
            }
        }
    }

    other
}

pub fn transform(program: &mut Program, settings: Settings) {
    program.body.transform(&settings);
}
