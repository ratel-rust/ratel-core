#[macro_use]
mod macros;
mod statement;
mod expression;
mod function;
mod value;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde_json;
use ast::{StatementList, Ptr, Loc, List};
use module::Module;

struct Program<'ast> {
    body: StatementList<'ast>,
}

impl<'ast> Serialize for Loc<Program<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
      let mut state = serializer.serialize_struct("Program", 4)?;
      state.serialize_field("type", &"Program")?;
      state.serialize_field("body", &self.body)?;
      state.serialize_field("start", &self.start)?;
      state.serialize_field("end", &self.end)?;
      state.end()
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

impl<'ast, T: Serialize> Serialize for Ptr<'ast, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_some(&*self)
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
                    "end": 4,
                    "expression": {
                        "type": "ThisExpression",
                        "end": 4,
                        "start": 0
                    },
                    "start": 0
                }
              ],
              "end": 0,
              "start": 0
        });
    }
}
