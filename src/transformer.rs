use grammar::*;
use grammar::ClassMember::*;
use grammar::OperatorType::*;

pub struct Settings {
    pub transform_block_scope: bool,
    pub transform_arrow: bool,
    pub transform_object: bool,
    pub transform_exponentation: bool,
    pub transform_class_properties: bool,
}

#[inline(always)]
fn bind_this(function: Expression) -> Expression {
    Expression::call(Expression::member(function, "bind"), vec![Expression::This])
}

impl Settings {
    pub fn target_es5() -> Settings {
        let mut settings = Settings::target_es2015();

        settings.transform_block_scope = true;
        settings.transform_arrow = true;
        settings.transform_object = true;

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
        }
    }
}

/// The `Transformable` trait provides an interface for instances of grammar
/// to alter the AST, either by mutating self, or by returning a new node.
///
/// NOTE: Returning `None` means no changes are necessary!
trait Transformable {
    fn transform(&mut self, _: &Settings) {}

    fn contains_this(&self) -> bool {
        false
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
                    Statement::Expression { ref value } => vec![
                        Statement::Return {
                            value: Some(value.clone())
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

                let (mut computed, literal): (Vec<ObjectMember>, Vec<ObjectMember>)
                = members.drain(..).partition(|member| {
                    match *member {
                        ObjectMember::Computed { .. } => true,
                        _                             => false,
                    }
                });

                if computed.is_empty() {
                    *members = literal;
                    return;
                }

                let mut body = Vec::new();

                body.push(Statement::VariableDeclaration {
                    kind: VariableDeclarationKind::Var,
                    declarators: vec![
                        VariableDeclarator {
                            name: "___".to_string(),
                            value: Some(Expression::Object(literal)),
                        }
                    ]
                });

                for member in computed.drain(..) {
                    if let ObjectMember::Computed { key, value } = member {
                        body.push(Statement::Expression {
                            value: Expression::Binary {
                                left: Box::new(Expression::Member {
                                    object: Box::new(Expression::ident("___")),
                                    property: Box::new(
                                        MemberKey::Computed(key)
                                    )
                                }),
                                operator: Assign,
                                right: Box::new(value),
                            }
                        });
                    }
                }

                body.push(Statement::Return {
                    value: Some(Expression::ident("___"))
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
                ref mut left,
                ref mut operator,
                ref mut right,
            } => {
                left.transform(settings);
                right.transform(settings);

                if !settings.transform_exponentation {
                    return;
                }

                match *operator {
                    Exponent => Expression::call(
                        Expression::member(Expression::ident("Math"), "pow"),
                        vec![*left.clone(), *right.clone()]
                    ),

                    ExponentAssign => {
                        *operator = Assign;
                        *right = Box::new(Expression::call(
                            Expression::member(Expression::ident("Math"), "pow"),
                            vec![*left.clone(), *right.clone()]
                        ));
                        return;
                    },

                    _ => return,
                }
            }

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

impl Transformable for ObjectMember {
    fn transform(&mut self, settings: &Settings) {
        *self = match *self {

            ObjectMember::Shorthand {
                ref key,
            } => {
                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Literal {
                    key: key.clone(),
                    value: Expression::Identifier(key.clone()),
                }
            },

            ObjectMember::Literal {
                ref mut value,
                ..
            } => {
                value.transform(settings);
                return;
            },

            ObjectMember::Computed {
                ref mut key,
                ref mut value,
            } => {
                key.transform(settings);
                value.transform(settings);
                return;
            },

            ObjectMember::Method {
                ref mut name,
                ref mut params,
                ref mut body,
            } => {
                body.transform(settings);
                params.transform(settings);

                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Literal {
                    key: name.clone(),
                    value: Expression::Function {
                        name: Some(name.clone()),
                        params: params.split_off(0),
                        body: body.split_off(0),
                    }
                }
            },

            ObjectMember::ComputedMethod {
                ref mut name,
                ref mut params,
                ref mut body,
            } => {
                body.transform(settings);
                params.transform(settings);

                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Computed {
                    key: name.clone(),
                    value: Expression::Function {
                        name: None,
                        params: params.split_off(0),
                        body: body.split_off(0),
                    }
                }
            },
        }
    }

    fn contains_this(&self) -> bool {
        match *self {
            ObjectMember::Literal {
                ref value,
                ..
            } => value.contains_this(),

            ObjectMember::Computed {
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
    fn transform(&mut self, settings: &Settings) {
        match self.value {
            Some(ref mut expression) => expression.transform(settings),
            _                        => {},
        }
    }

    fn contains_this(&self) -> bool {
        match self.value {
            Some(ref expression) => expression.contains_this(),
            _                    => false,
        }
    }
}

fn add_props_to_body(body: &mut Vec<Statement>, props: &mut Vec<ClassMember>) {
    // for prop in props {
    //     match prop {
    //         ClassMember::Property {
    //             ref is_static,
    //             ref name,
    //         } => {

    //         }
    //     }
    // }
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
                if let Some(ref mut alternate) = *alternate {
                    alternate.transform(settings);
                }
                return;
            },

            Statement::Class {
                ref mut body,
                ..
            } => {
                body.transform(settings);

                if !settings.transform_class_properties {
                    return;
                }

                let (mut props, mut methods): (Vec<ClassMember>, Vec<ClassMember>)
                = body.drain(..).partition(|member| {
                    match *member {
                        ClassMember::Property { .. } => true,
                        _                            => false,
                    }
                });

                if !props.is_empty() {
                    // copy the props into the class constructor
                    for method in methods.iter_mut() {
                        match *method {
                            ClassMember::Constructor {
                                ref mut body,
                                ..
                            } => {
                                add_props_to_body(body, &mut props);
                                break
                            }
                            _ => continue
                        }
                    }
                }

                *body = methods;

                return;
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
    fn transform(&mut self, settings: &Settings) {
        for item in self.iter_mut() {
            item.transform(settings);
        }
    }

    fn contains_this(&self) -> bool {
        for item in self {
            if item.contains_this() {
                return true;
            }
        }
        return false;
    }
}

pub fn transform(program: &mut Program, settings: Settings) {
    program.body.transform(&settings);
}
