use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::TsInterfaceDecl;

use super::Name;

pub struct Interface {
    name: Name,
}

impl From<TsInterfaceDecl> for Interface {
    fn from(decl: TsInterfaceDecl) -> Self {
        let name = Name::from(decl.id.sym.to_string());

        // TODO

        Self { name }
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_struct(self.name.as_ref());

        // TODO

        f.finish()
    }
}
