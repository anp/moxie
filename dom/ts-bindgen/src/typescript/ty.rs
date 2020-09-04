#![allow(unused)]

use std::fmt::{Debug, Formatter, Result as FmtResult};
use swc_ecma_ast::{TsEntityName, TsKeywordTypeKind, TsType, TsTypeAnn};

use super::Name;

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
            TsType::TsFnOrConstructorType(_) => {
                println!("TODO function or ctor type in annotation");
                Ty::Any
            }
            TsType::TsTypeRef(r) => Ty::Named(match r.type_name {
                TsEntityName::Ident(i) => i.sym.to_string().into(),
                TsEntityName::TsQualifiedName(n) => todo!("qualified type references"),
            }),
            TsType::TsTypeLit(l) => {
                println!("TODO type literals in annotations");
                Ty::Any
            }
            TsType::TsArrayType(a) => Ty::Array(Box::new((*a.elem_type).into())),
            TsType::TsTupleType(t) => {
                Ty::Tuple(t.elem_types.into_iter().map(|t| t.ty.into()).collect())
            }
            TsType::TsUnionOrIntersectionType(u) => {
                println!("TODO union/intersect types in annotations");
                Ty::Any
            }
            TsType::TsParenthesizedType(p) => {
                println!("TODO paren'sized types");
                Ty::Any
            }
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
        }
    }
}
