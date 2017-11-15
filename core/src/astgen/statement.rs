use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde_json;
use ast::{StatementList, Statement, Loc, Expression, ExpressionPtr};
use ast::{Declarator, DeclaratorId};

use super::function::ClassBody;

#[derive(Debug)]
struct CatchClause<'ast> {
    param: ExpressionPtr<'ast>,
    body: StatementList<'ast>,
}

#[derive(Debug)]
pub struct BlockStatement<'ast> {
    pub body: StatementList<'ast>
}

impl<'ast> Serialize for Loc<BlockStatement<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("BlockStatement", 3)?;
        state.serialize_field("type", &"BlockStatement")?;
        state.serialize_field("body", &self.body)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<CatchClause<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("CatchClause", 4)?;
        state.serialize_field("type", &"CatchClause")?;
        state.serialize_field("param", &*self.param)?;
        let body = Loc::new(self.start, self.end, BlockStatement { body: self.body });
        state.serialize_field("body", &body)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<Declarator<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {

        let mut state = serializer.serialize_struct("VariableDeclarator", 4)?;
        state.serialize_field("type", &"VariableDeclarator")?;

        state.serialize_field("id", &Loc::new(self.start, self.end, self.name))?;
        if let Some(value) = self.value {
           state.serialize_field("init", &*value)?;
        } else {
           state.serialize_field("init", &serde_json::Value::Null)?;
        }

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<DeclaratorId<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match self.item {
            DeclaratorId::Identifier(ident) => {
                let value = &Loc::new(self.start, self.end, Expression::Identifier(ident));
                serializer.serialize_some(value)
            },
            DeclaratorId::Pattern(expr) => {
                match expr.item {
                    Expression::Array { body } => {
                        let mut state = serializer.serialize_struct("ArrayPattern", 4)?;
                        state.serialize_field("type", &"ArrayPattern")?;
                        state.serialize_field("elements", &body)?;
                        return state.end();
                    },
                    _ => {
                        panic!("Unimplemented: ParameterKey::Pattern(expr)");
                    }
                }
            }
        }
    }
}

impl<'ast> Serialize for Loc<Statement<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
      use self::Statement::*;

      let mut state = match self.item {
        Error { .. } => panic!("Module contains errors"),
        Empty => {
            let mut state = serializer.serialize_struct("EmptyStatement", 3)?;
            state.serialize_field("type", &"EmptyStatement")?;
            state
        },
        Expression { expression } => {
            let mut state = serializer.serialize_struct("ExpressionStatement", 4)?;
            state.serialize_field("type", &"ExpressionStatement")?;
            state.serialize_field("expression", &*expression)?;
            state
        },
        Declaration { kind, declarators } => {
            let mut state = serializer.serialize_struct("VariableDeclaration", 5)?;
            state.serialize_field("type", &"VariableDeclaration")?;
            state.serialize_field("kind", &kind)?;
            state.serialize_field("declarations", &declarators)?;
            state
        },
        Return { value } => {
            let mut state = serializer.serialize_struct("ReturnStatement", 3)?;
            state.serialize_field("type", &"ReturnStatement")?;
            if let Some(expr) = value {
                state.serialize_field("argument", &*expr)?;
            } else {
                state.serialize_field("argument", &serde_json::Value::Null)?;
            }
            state
        },
        Break { label } => {
            let mut state = serializer.serialize_struct("BreakStatement", 4)?;
            state.serialize_field("type", &"BreakStatement")?;
            if let Some(expr) = label {
                state.serialize_field("label", &*expr)?;
            } else {
                state.serialize_field("label", &serde_json::Value::Null)?;
            }
            state
        },
        Throw { value } => {
            let mut state = serializer.serialize_struct("ThrowStatement", 4)?;
            state.serialize_field("type", &"ThrowStatement")?;
            state.serialize_field("argument", &*value)?;
            state
        },
        If { test, consequent, alternate } => {
            let mut state = serializer.serialize_struct("IfStatement", 6)?;
            state.serialize_field("type", &"IfStatement")?;
            state.serialize_field("test", &*test)?;
            state.serialize_field("consequent", &*consequent)?;
            if let Some(alternate) = alternate {
                state.serialize_field("alternate", &*alternate)?;
            } else {
                state.serialize_field("alternate", &serde_json::Value::Null)?;
            }
            state
        },
        While { test, body } => {
            let mut state = serializer.serialize_struct("WhileStatement", 5)?;
            state.serialize_field("type", &"WhileStatement")?;
            state.serialize_field("test", &*test)?;
            state.serialize_field("body", &*body)?;
            state
        },
        Do { body, test } => {
            let mut state = serializer.serialize_struct("DoWhileStatement", 5)?;
            state.serialize_field("type", &"DoWhileStatement")?;
            state.serialize_field("body", &*body)?;
            state.serialize_field("test", &*test)?;
            state
        },
        For { init, test, update, body } => {
            let mut state = serializer.serialize_struct("ForStatement", 7)?;
            state.serialize_field("type", &"ForStatement")?;

            if let Some(init) = init {
                state.serialize_field("init", &*init)?;
            } else {
                state.serialize_field("init", &serde_json::Value::Null)?;
            }

            if let Some(test) = test {
                state.serialize_field("test", &*test)?;
            } else {
                state.serialize_field("init", &serde_json::Value::Null)?;
            }

            if let Some(update) = update {
                state.serialize_field("update", &*update)?;
            } else {
                state.serialize_field("update", &serde_json::Value::Null)?;
            }

            state.serialize_field("body", &*body)?;
            state
        },
        ForIn { left, right, body } => {
            let mut state = serializer.serialize_struct("ForInStatement", 6)?;
            state.serialize_field("type", &"ForInStatement")?;
            state.serialize_field("left", &*left)?;
            state.serialize_field("right", &*right)?;
            state.serialize_field("body", &*body)?;
            state
        },
        ForOf { left, right, body } => {
            let mut state = serializer.serialize_struct("ForOfStatement", 6)?;
            state.serialize_field("type", &"ForOfStatement")?;
            state.serialize_field("left", &*left)?;
            state.serialize_field("right", &*right)?;
            state.serialize_field("body", &*body)?;
            state
        },
        Try { body, error, handler } => {
            let mut state = serializer.serialize_struct("TryStatement", 5)?;
            state.serialize_field("type", &"TryStatement")?;
            state.serialize_field("block", &Loc::new(self.start, self.end, BlockStatement { body: body }))?;
            let handler = Loc::new(self.start, self.end, CatchClause {
                param: error,
                body: handler
            });

            state.serialize_field("handler", &handler)?;
            state
        },
        Block { body } => {
            let mut state = serializer.serialize_struct("BlockStatement", 4)?;
            state.serialize_field("type", &"BlockStatement")?;
            state.serialize_field("body", &body)?;
            state
        },
        Labeled { label, body } => {
            let mut state = serializer.serialize_struct("LabeledStatement", 5)?;
            state.serialize_field("type", &"LabeledStatement")?;
            state.serialize_field("label", &label)?;
            state.serialize_field("body", &*body)?;
            state
        },
        Function { function } => {
            let mut state = serializer.serialize_struct("FunctionDeclaration", 6)?;
            state.serialize_field("type", &"FunctionDeclaration")?;
            state.serialize_field("name", &Loc::new(self.start, self.end, function.name))?;
            state.serialize_field("params", &function.params)?;

            match function.body.only_element() {
                Some(&Loc { item: Block { .. } , .. }) => {
                    state.serialize_field("body", &function.body)?;
                },
                _ => {
                let body = BlockStatement { body: function.body };
                    state.serialize_field("body", &Loc::new(self.start, self.end, body))?;
                }
            };
            state
        },
        Class { class } => {
            let mut state = serializer.serialize_struct("ClassDeclaration", 6)?;
            state.serialize_field("type", &"ClassDeclaration")?;
            state.serialize_field("id", &Loc::new(self.start, self.end, class.name))?;
            if let Some(extends) = class.extends {
                state.serialize_field("superClass", &*extends)?;
            } else {
                state.serialize_field("superClass", &serde_json::Value::Null)?;
            }
            state.serialize_field("body", &Loc::new(self.start, self.end, ClassBody { body: class.body }))?;
            state
        },
        Continue { label } => {
            let mut state = serializer.serialize_struct("ContinueStatement", 4)?;
            state.serialize_field("type", &"ContinueStatement")?;
            if let Some(label) = label {
                state.serialize_field("label", &*label)?;
            } else  {
                state.serialize_field("label", &serde_json::Value::Null)?;
            }
            state
        },
        Switch { discriminant, cases } => {
            let mut state = serializer.serialize_struct("SwitchStatement", 5)?;
            state.serialize_field("type", &"SwitchStatement")?;
            state.serialize_field("discriminant", &*discriminant)?;
            state.serialize_field("cases", &cases)?;
            state
        },
        SwitchCase { test, consequent } => {
            let mut state = serializer.serialize_struct("SwitchCase", 5)?;
            state.serialize_field("type", &"SwitchCase")?;

            if let Some(test) = test {
                state.serialize_field("test", &*test)?;
            } else {
                state.serialize_field("test", &serde_json::Value::Null)?;
            }

            state.serialize_field("consequent", &consequent)?;
            state
        }
      };

      state.serialize_field("start", &self.start)?;
      state.serialize_field("end", &self.end)?;
      state.end()
    }
}
