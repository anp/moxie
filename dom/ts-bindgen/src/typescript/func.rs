use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::{Function, ParamOrTsParamProp};

use super::{Name, TsParam, Ty};

pub struct Func {
    is_generator: bool,
    is_async: bool,
    params: Vec<TsParam>,
    returns: Option<Ty>,
}

impl Func {
    pub fn ctor(name: &Name, params: Vec<ParamOrTsParamProp>) -> Self {
        Self {
            is_async: false,
            is_generator: false,
            params: params
                .into_iter()
                .map(|param| match param {
                    ParamOrTsParamProp::Param(p) => p.into(),
                    ParamOrTsParamProp::TsParamProp(t) => {
                        todo!("support ts 'param props' from swc ast {:#?}", t);
                    }
                })
                .collect(),
            returns: Some(Ty::Named(name.clone())),
        }
    }
}

impl From<Function> for Func {
    fn from(function: Function) -> Self {
        Self {
            is_async: function.is_async,
            is_generator: function.is_generator,
            params: function.params.into_iter().map(From::from).collect(),
            returns: function.return_type.map(From::from),
        }
    }
}

impl Debug for Func {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let asyncness = if self.is_async { "async " } else { "" };
        let genny = if self.is_generator { "*" } else { "" };
        let prelude = format!("{}function{} ", asyncness, genny);

        if self.params.is_empty() {
            write!(f, "{}()", prelude)?;
        } else {
            let mut tup = f.debug_tuple(&prelude);
            for p in &self.params {
                tup.field(p);
            }
            tup.finish()?;
        }

        if let Some(ret) = &self.returns { write!(f, " -> {:?}", ret) } else { Ok(()) }
    }
}
