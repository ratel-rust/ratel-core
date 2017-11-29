#[macro_use]
mod macros;
mod statement;
mod expression;
mod function;
mod value;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use ast::{Loc, Node, NodeList, Statement, StatementList};
use module::Module;

#[derive(Debug)]
pub struct Program<'ast> {
    pub body: StatementList<'ast>,
}

pub trait SerializeInLoc {
    #[inline]
    fn in_loc<S, F>(&self, serializer: S, name: &'static str, length: usize, build: F) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
        F: FnOnce(&mut S::SerializeStruct) -> Result<(), S::Error>
    {
        let mut state = serializer.serialize_struct(name, length + 3)?;
        state.serialize_field("type", name)?;
        build(&mut state).map(move |_| state)
    }

    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where S: Serializer;
}

impl<'ast, T: SerializeInLoc> Serialize for Loc<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = self.item.serialize(serializer)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast, T: SerializeInLoc> Serialize for Node<'ast, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        Loc::<T>::serialize(&*self, serializer)
    }
}

impl<'ast> Serialize for Program<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut start = 0;
        let mut end = 0;
        let mut iter = self.body.iter().peekable();

        if let Some(node) = iter.next() {
            start = node.start;
            end = node.end;
            while let Some(node) = iter.next() {
                if iter.peek().is_none() {
                    end = node.end;
                }
            }
        }

        let name = "Program";
        let mut state = serializer.serialize_struct(name, 4)?;
        state.serialize_field("type", &name)?;
        state.serialize_field("body", &self.body)?;
        state.serialize_field("start", &start)?;
        state.serialize_field("end", &end)?;
        state.end()
    }
}

#[cfg(test)]
use serde_json;

#[cfg(test)]
pub fn generate_ast<'ast>(module: &Module) -> serde_json::Value {
    json!(Program {
        body: module.body()
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::{parse};

    #[test]
    fn test_generate_ast() {
        expect_parse!("this;", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ThisExpression",
                        "start": 0,
                        "end": 4,
                    },
                    "start": 0,
                    "end": 4,
                }
              ],
              "start": 0,
              "end": 4,
        });
    }
}
