use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::{TsEnumMember, TsEnumMemberId};

use super::Name;

pub struct Enum {
    members: Vec<Name>,
}

impl From<Vec<TsEnumMember>> for Enum {
    fn from(decls: Vec<TsEnumMember>) -> Self {
        Enum { members: decls.into_iter().map(|member| member_name(&member.id)).collect() }
    }
}

impl Debug for Enum {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_set().entries(&self.members).finish()
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
