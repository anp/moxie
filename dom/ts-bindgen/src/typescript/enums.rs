use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::*;

use super::Name;

pub struct Enum {
    name: Name,
    members: Vec<Name>,
}

impl From<TsEnumDecl> for Enum {
    fn from(decl: TsEnumDecl) -> Self {
        let name = decl.id.sym.to_string().into();
        let members = decl
            .members
            .into_iter()
            .map(|member| {
                let member_name = member_name(&member.id);
                if member.init.is_some() {
                    println!("TODO enum member init {:?}.{:?}", &name, &member_name);
                }
                member_name
            })
            .collect();
        Enum { name, members }
    }
}

impl Debug for Enum {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut tup = f.debug_tuple(&self.name.as_ref());
        for member in &self.members {
            tup.field(member);
        }
        tup.finish()
    }
}

fn member_name(id: &TsEnumMemberId) -> Name {
    match id {
        TsEnumMemberId::Ident(i) => &i.sym,
        TsEnumMemberId::Str(s) => &s.value,
    }
    .to_string()
    .into()
}
