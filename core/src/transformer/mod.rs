mod scope;
mod settings;

use ast::{List, Loc, Statement};

pub use self::settings::Settings;

pub fn transform<'ast>(list: &mut List<'ast, Loc<Statement<'ast>>>, settings: Settings) {

}
