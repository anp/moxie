use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};
use swc_ecma_ast::{Function, ParamOrTsParamProp, TsConstructorType, TsFnType};

use super::{Name, TsParam, Ty, TyParam};

#[derive(Clone)]
pub struct Func {
    is_ctor: bool,
    is_generator: bool,
    is_async: bool,
    ty_params: BTreeMap<Name, TyParam>,
    params: Vec<TsParam>,
    returns: Option<Ty>,
}

impl Func {
    pub fn ctor(name: &Name, params: Vec<ParamOrTsParamProp>) -> Self {
        Self {
            is_ctor: true,
            is_async: false,
            is_generator: false,
            ty_params: Default::default(), // constructors only have the class' ty params
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
            is_ctor: false,
            is_async: function.is_async,
            is_generator: function.is_generator,
            ty_params: TyParam::make_map(function.type_params),
            params: function.params.into_iter().map(From::from).collect(),
            returns: function.return_type.map(From::from),
        }
    }
}

impl From<TsFnType> for Func {
    fn from(function: TsFnType) -> Self {
        Self {
            is_ctor: false,
            is_generator: false,
            is_async: false,
            ty_params: TyParam::make_map(function.type_params),
            returns: Some(function.type_ann.into()),
            params: function.params.into_iter().map(From::from).collect(),
        }
    }
}

impl From<TsConstructorType> for Func {
    fn from(function: TsConstructorType) -> Self {
        Self {
            is_ctor: true,
            is_generator: false,
            is_async: false,
            ty_params: TyParam::make_map(function.type_params),
            returns: Some(function.type_ann.into()),
            params: function.params.into_iter().map(From::from).collect(),
        }
    }
}

impl Debug for Func {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let fun = if self.is_ctor { "new" } else { "function" };
        let asyncness = if self.is_async { "async " } else { "" };
        let genny = if self.is_generator { "*" } else { "" };
        let prelude = format!("{}{}{} ", asyncness, fun, genny);

        if self.params.is_empty() {
            write!(f, "{}()", prelude)?;
        } else {
            let mut tup = f.debug_tuple(&prelude);
            for p in &self.params {
                tup.field(p);
            }
            tup.finish()?;
        }

        if let Some(ret) = &self.returns { write!(f, ": {:?}", ret) } else { Ok(()) }
    }
}
