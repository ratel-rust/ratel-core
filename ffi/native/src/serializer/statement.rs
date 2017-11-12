use serde_json;
use ast;
use serializer::Serializable;
use ast::{Statement, StatementPtr, Ptr, Loc, ObjectMember, DeclarationKind, Declarator, Property};

impl<'ast> Serializable<'ast> for StatementPtr<'ast> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        use self::Statement::*;

        let result = match self.item {
            Error { .. } => panic!("Module contains errors"),
            Empty => return None,
            Expression { expression } => {
                json!({
                    "type": "ExpressionStatement",
                    "expression": expression.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Declaration { kind, declarators } => {
                json!({
                    "type": "VariableDeclaration",
                    "kind": kind.serialize(),
                    "declarations": declarators.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Return { value } => {
                json!({
                    "type": "ReturnStatement",
                    "argument": value.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Break { label } => {
                json!({
                    "type": "BreakStatement",
                    "label": label.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Throw { value } => {
                json!({
                    "type": "ThrowStatement",
                    "argument": value.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            If { test, consequent, alternate } => {
                json!({
                    "type": "IfStatement",
                    "test": test.serialize(),
                    "consequent": consequent.serialize(),
                    "alternate": alternate.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            While { test, body } => {
                json!({
                    "type": "WhileStatement",
                    "test": test.serialize(),
                    "body": body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Do { body, test } => {
                json!({
                    "type": "DoWhileStatement",
                    "body": body.serialize(),
                    "test": test.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            For { init, test, update, body } => {
                json!({
                    "type": "ForStatement",
                    "init": init.serialize(),
                    "test": test.serialize(),
                    "update": update.serialize(),
                    "body": body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            ForIn { left, right, body } => {
               json!({
                    "type": "ForInStatement",
                    "left": left.serialize(),
                    "right": right.serialize(),
                    "body": body.serialize(),
                    "start": self.start,
                    "end": self.end,
               })
            },
            ForOf { left, right, body } => {
               json!({
                    "type": "ForOfStatement",
                    "left": left.serialize(),
                    "right": right.serialize(),
                    "body": body.serialize(),
                    "start": self.start,
                    "end": self.end,
               })
            },
            Try { body, error, handler } => {
                json!({
                    "type": "TryStatement",
                    "block": body.serialize(),
                    "handler": {
                        "type": "CatchClause",
                        "param": error.serialize(),
                        "body": handler.serialize()
                    },
                    "start": self.start,
                    "end": self.end,
                })
            },
            Block { body } => {
                json!({
                    "type": "BlockStatement",
                    "body": body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Labeled { label, body } => {
                json!({
                    "type": "LabeledStatement",
                    "label": label,
                    "body": body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Function { function } => {
                json!({
                    "type": "FunctionDeclaration",
                    "name": function.name.serialize(),
                    "params": function.params.serialize(),
                    "body": function.body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Class { class } => {
                json!({
                    "type": "ClassDeclaration",
                    "id": class.name.serialize(),
                    "superClass": class.extends.serialize(),
                    "body": {
                        "type": "ClassBody",
                        "body": class.body.serialize()
                    },
                    "start": self.start,
                    "end": self.end,
                })
            },
            Continue { label } => {
                json!({
                    "type": "ContinueStatement",
                    "label": label.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Switch { discriminant, cases } => {
                json!({
                    "type": "SwitchStatement",
                    "discriminant": discriminant.serialize(),
                    "cases": cases.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            SwitchCase { test, consequent } => {
                json!({
                    "type": "SwitchCase",
                    "test": test.serialize(),
                    "consequent": consequent.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
        };
        Some(result)
    }
}

impl<'ast> Serializable<'ast> for Loc<ObjectMember<'ast>> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        let result = match self.item {
            ObjectMember::Shorthand(value) => {
                // FIXME
                let key = Ptr::new(&Loc::new(self.start, self.end, value)).serialize();
                json!({
                    "type": "Property",
                    "key": key,
                    "method": false,
                    "shorthand": true,
                    "computed": false,
                    "value": key,
                    "start": self.start,
                    "end": self.end,
                })
            },
            ObjectMember::Value { property, value } => {
                // FIXME
                let key = Ptr::new(&Loc::new(self.start, self.end, property)).serialize();
                let computed = if let Property::Computed(value) = property.item { true } else { false };

                json!({
                    "type": "Property",
                    "key": property.serialize(),
                    "method": false,
                    "shorthand": false,
                    "computed": computed,
                    "value": value.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            ObjectMember::Method { property, params, body } => {
                // FIXME
                let function = ast::Function {
                    name: ast::Name::empty(),
                    params,
                    body
                };
                let value = Ptr::new(&Loc::new(self.start, self.end, ast::Expression::Function { function })).serialize();

                json!({
                    "type": "Property",
                    "key": property.serialize(),
                    "method": true,
                    "shorthand": false,
                    "computed": false,
                    "value": value,
                    "start": self.start,
                    "end": self.end,
                })
            },
        };
        Some(result)
    }
}

impl<'ast> Serializable<'ast> for Loc<Property<'ast>> {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        use self::Property::*;
        let result = match self.item {
            Computed(expr) => return expr.serialize(),
            Literal(value) => {
                let loc = &Loc::new(self.start, self.end, value);
                let value = Ptr::new(loc);
                return value.serialize()
            },
            Binary(value) => json!(value),
        };
        Some(result)
    }
}

impl<'ast> Serializable<'ast> for Loc<Declarator<'ast>> {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        Some(json!({
            "type": "VariableDeclarator",
            "id": self.name.serialize(),
            "init": self.value.serialize(),
            "start": self.start,
            "end": self.end
        }))
    }
}

impl<'ast> Serializable<'ast> for DeclarationKind {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        use self::DeclarationKind::*;
        let result = match *self {
            Const => json!("const"),
            Let => json!("let"),
            Var => json!("var"),
        };
        Some(result)
    }
}
