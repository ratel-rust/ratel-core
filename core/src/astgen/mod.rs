#[macro_use]
mod macros;
mod statement;
mod expression;
mod function;
mod value;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde_json;
use ast::{StatementList, Node, Loc, List};
use module::Module;

struct Program<'ast> {
    body: StatementList<'ast>,
}

pub trait SerializeInLoc {
    #[inline]
    fn in_loc<S, F>(&self, serializer: S, name: &'static str, length: usize, build: F) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer, F: FnOnce(&mut S::SerializeStruct) -> Result<(), S::Error>
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
        where S: Serializer
    {
        let mut state = self.item.serialize(serializer)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> SerializeInLoc for Program<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer
    {
        self.in_loc(serializer, "Program", 1, |state| {
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast, T: 'ast> Serialize for List<'ast, T>
    where T: Serialize
    {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'ast, T: Serialize> Serialize for Node<'ast, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        T::serialize(&**self, serializer)
    }
}

pub fn generate_ast<'ast>(module: &Module) -> serde_json::Value {
    let body = module.body();
    json!(Loc::new(0, 0, Program { body }))
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::{parse};
    use astgen::generate_ast;

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
              "end": 0,
        });
    }
}
