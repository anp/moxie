#![allow(unused)]

use swc_ecma_ast::{TsType, TsTypeAnn};

#[derive(Clone, Debug)]
pub struct Ty {
    // TODO figure out a better repr for not-yet-resolved types
}

impl Ty {
    pub fn any() -> Self {
        Ty {}
    }
}

impl From<TsType> for Ty {
    fn from(ty: TsType) -> Ty {
        match ty {
            TsType::TsKeywordType(kw) => {
                println!("TODO keyword types");
                Ty::any()
            }
            TsType::TsFnOrConstructorType(_) => {
                println!("TODO function or ctor type in annotation");
                Ty::any()
            }
            TsType::TsTypeRef(r) => {
                println!("TODO type reference in annotations");
                Ty::any()
            }
            TsType::TsTypeLit(l) => {
                println!("TODO type literals in annotations");
                Ty::any()
            }
            TsType::TsArrayType(a) => {
                println!("TODO array types in annotations");
                Ty::any()
            }
            TsType::TsTupleType(t) => {
                println!("TODO tuple types in annotations");
                Ty::any()
            }
            TsType::TsUnionOrIntersectionType(u) => {
                println!("TODO union/intersect types in annotations");
                Ty::any()
            }
            TsType::TsOptionalType(o) => todo!("optional types"),
            TsType::TsRestType(r) => todo!("rest types"),
            TsType::TsTypeQuery(q) => todo!("type queries"),
            TsType::TsThisType(_) => todo!("`this`"),
            TsType::TsConditionalType(c) => todo!("conditional types"),
            TsType::TsInferType(i) => todo!("inferred types"),
            TsType::TsParenthesizedType(p) => todo!("paren'sized types"),
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
