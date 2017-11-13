use serde_json;
use module::Module;
use ast;
use ast::{List, StatementList, ExpressionList, ParameterList,IdentifierPtr};

mod expression;
mod statement;
mod function;

pub trait Serializable<'ast> {
    fn serialize(&self) -> Option<serde_json::Value>;
}

impl<'ast, T: 'ast + Serializable<'ast>> Serializable<'ast> for List<'ast, T> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        let mut result: Vec<Option<serde_json::Value>> = vec![];
        for statement in self.iter() {
            let entry = statement.serialize();
            if entry.is_some() {
                result.push(entry);
            }
        }
        Some(json!(result))
    }
}

impl<'ast, T: 'ast + Serializable<'ast>> Serializable<'ast> for Option<T> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        if let Some(ref value) = *self {
            value.serialize()
        } else {
            None
        }
    }
}

impl<'ast> Serializable<'ast> for ParameterList<'ast> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        let mut result: Vec<Option<serde_json::Value>> = vec![];
        for expression in self.ptr_iter() {
            let entry = expression.serialize();
            if entry.is_some() {
                result.push(entry);
            }
        }
        Some(json!(result))
    }
}

impl<'ast> Serializable<'ast> for StatementList<'ast> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        let mut result: Vec<Option<serde_json::Value>> = vec![];
        for statement in self.ptr_iter() {
            let entry = statement.serialize();
            if entry.is_some() {
                result.push(entry);
            }
        }
        Some(json!(result))
    }
}

impl<'ast> Serializable<'ast> for ExpressionList<'ast> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        let mut result: Vec<Option<serde_json::Value>> = vec![];
        for expression in self.ptr_iter() {
            let entry = expression.serialize();
            if entry.is_some() {
                result.push(entry);
            }
        }
        Some(json!(result))
    }
}

impl<'ast> Serializable<'ast> for IdentifierPtr<'ast> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        Some(json!({
            "type": "Identifier",
            "name": self.item,
            "start": self.start,
            "end": self.end,
        }))
    }
}

pub fn serialize<'ast>(module: &Module) -> Option<serde_json::Value> {
    let body = module.body();
    Some(json!({
        "type": "Program",
        "body": body.serialize(),
        "start": 0,
        "end": 0
    }))
}
