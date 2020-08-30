use crate::{error::BindingError, wasm::WasmBindgenImport};
use std::{convert::TryFrom, fmt::Debug, str::FromStr};
use swc_common::BytePos;
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::input::StringInput, Parser, Syntax, TsConfig};

pub fn parse_d_ts(contents: &str) -> Result<Module, BindingError> {
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

#[derive(Debug)]
pub struct TsModule {}

impl FromStr for TsModule {
    type Err = BindingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(parse_d_ts(s)?)
    }
}

impl TryFrom<Module> for TsModule {
    type Error = BindingError;

    fn try_from(_module: Module) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TsModule {
    pub fn import_with_wasm_bindgen(&self) -> Result<WasmBindgenImport, BindingError> {
        todo!()
    }
}
