use swc_ecma_ast::*;

#[derive(Debug)]
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
            TsType::TsThisType(_) => {
                println!("TODO handle `this` in type annotation??");
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
            TsType::TsTypeQuery(q) => {
                println!("TODO type queries in annotations");
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
            TsType::TsOptionalType(o) => {
                println!("TODO optional types in annotations");
                Ty::any()
            }
            TsType::TsRestType(r) => {
                println!("TODO rest types in annotations");
                Ty::any()
            }
            TsType::TsUnionOrIntersectionType(u) => {
                println!("TODO union/intersect types in annotations");
                Ty::any()
            }
            TsType::TsConditionalType(c) => {
                println!("TODO conditional types in annotations");
                Ty::any()
            }
            TsType::TsInferType(i) => {
                println!("TODO inferred types in annotations");
                Ty::any()
            }
            TsType::TsParenthesizedType(p) => {
                println!("TODO paren'sized types in annotations");
                Ty::any()
            }
            TsType::TsTypeOperator(o) => {
                println!("TODO type operators in annotations");
                Ty::any()
            }
            TsType::TsIndexedAccessType(i) => {
                println!("TODO indexed access types in annotations");
                Ty::any()
            }
            TsType::TsMappedType(m) => {
                println!("TODO mapped types in annotations");
                Ty::any()
            }
            TsType::TsLitType(l) => {
                println!("TODO literal types in annotations");
                Ty::any()
            }
            TsType::TsTypePredicate(p) => {
                println!("TODO predicates in annotations");
                Ty::any()
            }
            TsType::TsImportType(i) => {
                println!("TODO import types in annotations");
                Ty::any()
            }
        }
    }
}

impl From<TsTypeAnn> for Ty {
    fn from(ann: TsTypeAnn) -> Ty {
        (*ann.type_ann).into()
    }
}
