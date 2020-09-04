use crate::error::TypescriptError;
use swc_common::BytePos;
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::input::StringInput, Parser, Syntax, TsConfig};

mod class;
mod enums;
mod func;
mod interface;
mod module;
mod name;
mod param;
mod ty;

pub use class::Class;
pub use enums::Enum;
pub use func::Func;
pub use interface::Interface;
pub use module::TsModule;
pub use name::Name;
pub use param::TsParam;
pub use ty::Ty;

pub fn parse_d_ts(contents: &str) -> Result<Module, TypescriptError> {
    let input = StringInput::new(contents, BytePos(0), BytePos(0));
    let mut parser = Parser::new(
        Syntax::Typescript(TsConfig {
            tsx: false,
            decorators: false,
            dynamic_import: false,
            dts: true,
            no_early_errors: true,
        }),
        input,
        None, // TODO figure out what comments do here?
    );
    Ok(parser.parse_typescript_module()?)
}

impl std::str::FromStr for module::TsModule {
    type Err = TypescriptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(parse_d_ts(s)?))
    }
}
