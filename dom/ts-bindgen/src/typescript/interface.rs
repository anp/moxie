#![allow(unused)]

use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::TsInterfaceBody;

pub struct Interface {}

impl From<TsInterfaceBody> for Interface {
    fn from(decl: TsInterfaceBody) -> Self {
        // TODO

        Self {}
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Ok(())
    }
}
