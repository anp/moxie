#![allow(unused)]

use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::{
    TsEntityName, TsFnOrConstructorType, TsKeywordTypeKind, TsType, TsTypeAnn, TsTypeElement,
    TsUnionOrIntersectionType,
};

use super::{Func, Name};

#[derive(Clone)]
pub enum Ty {
    Any,
    Unknown,
    Number,
    Object,
    Boolean,
    BigInt,
    String,
    Symbol,
    Void,
    Undefined,
    Null,
    Never,
    Array(Box<Ty>),
    Tuple(Vec<Ty>),
    Named(Name),
    Fn(Box<Func>),
    Ctor(Box<Func>),
    Lit(Vec<Ty>),
    Union(Vec<Ty>),
    Intersection(Vec<Ty>),
}

impl From<TsType> for Ty {
    fn from(ty: TsType) -> Ty {
        match ty {
            TsType::TsKeywordType(kw) => match kw.kind {
                TsKeywordTypeKind::TsAnyKeyword => Ty::Any,
                TsKeywordTypeKind::TsUnknownKeyword => Ty::Unknown,
                TsKeywordTypeKind::TsNumberKeyword => Ty::Number,
                TsKeywordTypeKind::TsObjectKeyword => Ty::Object,
                TsKeywordTypeKind::TsBooleanKeyword => Ty::Boolean,
                TsKeywordTypeKind::TsBigIntKeyword => Ty::BigInt,
                TsKeywordTypeKind::TsStringKeyword => Ty::String,
                TsKeywordTypeKind::TsSymbolKeyword => Ty::Symbol,
                TsKeywordTypeKind::TsVoidKeyword => Ty::Void,
                TsKeywordTypeKind::TsUndefinedKeyword => Ty::Undefined,
                TsKeywordTypeKind::TsNullKeyword => Ty::Null,
                TsKeywordTypeKind::TsNeverKeyword => Ty::Never,
            },
            TsType::TsTypeRef(r) => Ty::Named(r.type_name.into()),
            TsType::TsArrayType(a) => Ty::Array(Box::new((*a.elem_type).into())),
            TsType::TsTupleType(t) => {
                Ty::Tuple(t.elem_types.into_iter().map(|t| t.ty.into()).collect())
            }
            TsType::TsFnOrConstructorType(fn_or_ctor) => match fn_or_ctor {
                TsFnOrConstructorType::TsFnType(func) => Ty::Fn(Box::new(func.into())),
                TsFnOrConstructorType::TsConstructorType(ctor) => Ty::Ctor(Box::new(ctor.into())),
            },
            TsType::TsTypeLit(l) => Ty::Lit(l.members.into_iter().map(Ty::from).collect()),
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(u)) => {
                Ty::Union(u.types.into_iter().map(|t| Ty::from(*t)).collect())
            }
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsIntersectionType(i)) => {
                Ty::Intersection(i.types.into_iter().map(|t| Ty::from(*t)).collect())
            }
            TsType::TsParenthesizedType(p) => Ty::from(*p.type_ann),
            TsType::TsOptionalType(o) => todo!("optional types"),
            TsType::TsRestType(r) => todo!("rest types"),
            TsType::TsTypeQuery(q) => todo!("type queries"),
            TsType::TsThisType(_) => todo!("`this`"),
            TsType::TsConditionalType(c) => todo!("conditional types"),
            TsType::TsInferType(i) => todo!("inferred types"),
            TsType::TsTypeOperator(o) => todo!("type operators"),
            TsType::TsIndexedAccessType(i) => todo!("indexed access types"),
            TsType::TsMappedType(m) => todo!("mapped types"),
            TsType::TsLitType(l) => todo!("literal types"),
            TsType::TsTypePredicate(p) => todo!("predicates"),
            TsType::TsImportType(i) => todo!("import types"),
        }
    }
}

impl From<TsTypeElement> for Ty {
    fn from(elem: TsTypeElement) -> Self {
        println!("TODO type element");
        match elem {
            TsTypeElement::TsPropertySignature(p) => Ty::Any,
            TsTypeElement::TsCallSignatureDecl(c) => Ty::Any,
            TsTypeElement::TsConstructSignatureDecl(c) => Ty::Any,
            TsTypeElement::TsMethodSignature(m) => Ty::Any,
            TsTypeElement::TsIndexSignature(i) => Ty::Any,
        }
    }
}

impl From<TsTypeAnn> for Ty {
    fn from(ann: TsTypeAnn) -> Ty {
        (*ann.type_ann).into()
    }
}

impl Debug for Ty {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Ty::Any => write!(f, "Any"),
            Ty::Unknown => write!(f, "Unknown"),
            Ty::Number => write!(f, "Number"),
            Ty::Object => write!(f, "Object"),
            Ty::Boolean => write!(f, "Boolean"),
            Ty::BigInt => write!(f, "BigInt"),
            Ty::String => write!(f, "String"),
            Ty::Symbol => write!(f, "Symbol"),
            Ty::Void => write!(f, "Void"),
            Ty::Undefined => write!(f, "undefined"),
            Ty::Null => write!(f, "null"),
            Ty::Never => write!(f, "!"),
            Ty::Array(elem) => write!(f, "Array<{:?}>", elem),
            Ty::Tuple(tup) => {
                let mut f = f.debug_tuple("");
                for ty in tup {
                    f.field(ty);
                }
                f.finish()
            }
            Ty::Named(name) => write!(f, "{}", name),
            Ty::Fn(fun) => write!(f, "{:?}", fun),
            Ty::Ctor(ctor) => write!(f, "new {:?}", ctor),
            Ty::Lit(members) => f.debug_set().entries(members).finish(),
            Ty::Union(members) => {
                let mut tup = f.debug_tuple("∪");
                for m in members {
                    tup.field(m);
                }
                tup.finish()
            }
            Ty::Intersection(members) => {
                let mut tup = f.debug_tuple("∩");
                for m in members {
                    tup.field(m);
                }
                tup.finish()
            }
        }
    }
}
