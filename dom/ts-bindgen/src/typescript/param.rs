use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::{Param, Pat, TsFnParam};

use super::{name::Name, ty::Ty};

#[derive(Clone)]
pub struct TsParam {
    name: Name,
    optional: bool,
    rest: bool,
    ty: Ty,
}

impl From<Param> for TsParam {
    fn from(param: Param) -> Self {
        match param.pat {
            Pat::Ident(i) => {
                let name = i.sym.to_string().into();
                let ty = i.type_ann.map(Ty::from).unwrap_or(Ty::Any);
                Self { name, ty, rest: false, optional: i.optional }
            }
            Pat::Rest(r) => {
                let name = r.arg.expect_ident().sym.to_string().into();
                let ty = r.type_ann.map(Ty::from).unwrap_or(Ty::Any);
                Self { name, ty, rest: true, optional: false }
            }
            other => todo!("other parameter types like {:#?}", other),
        }
    }
}

impl From<TsFnParam> for TsParam {
    fn from(param: TsFnParam) -> Self {
        match param {
            TsFnParam::Ident(i) => {
                let name = i.sym.to_string().into();
                let ty = i.type_ann.map(Ty::from).unwrap_or(Ty::Any);
                TsParam { name, ty, rest: false, optional: i.optional }
            }
            TsFnParam::Rest(r) => {
                let name = r.arg.expect_ident().sym.to_string().into();
                let ty = r.type_ann.map(Ty::from).unwrap_or(Ty::Any);
                TsParam { name, ty, rest: true, optional: false }
            }
            TsFnParam::Array(_) => todo!("typescript array function parameters"),
            TsFnParam::Object(_) => todo!("typescript object function parameters"),
        }
    }
}

impl Debug for TsParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let rest = if self.rest { "..." } else { "" };
        let optional = if self.optional { "?" } else { "" };
        write!(f, "{}{}{}: {:?}", rest, &self.name, optional, &self.ty)
    }
}
