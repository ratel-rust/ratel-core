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
struct Program<'ast> {
    body: StatementList<'ast>,
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

impl<'ast> SerializeInLoc for Program<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer
    {
        self.in_loc(serializer, "Program", 1, |state| {
            state.serialize_field("body", &self.body)
        })
    }
}

#[cfg(test)]
use serde_json;

#[cfg(test)]
pub fn generate_ast<'ast>(module: &Module) -> serde_json::Value {
    let program = Program {
        body: module.body()
    };
    json!(Loc::new(0, 0, program))
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
              "end": 0,
        });
    }
}
