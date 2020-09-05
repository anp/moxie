use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};
use swc_ecma_ast::{ClassDecl, ClassMember, ClassMethod, ClassProp, Constructor, Expr, PropName};

use super::{Func, Name, Ty};

pub struct Class {
    constructors: Vec<Func>,
    properties: BTreeMap<Name, Ty>,
    statics: BTreeMap<Name, Func>,
    methods: BTreeMap<Name, Func>,
}

impl From<ClassDecl> for Class {
    fn from(class: ClassDecl) -> Self {
        let mut new = Class {
            constructors: Default::default(),
            properties: Default::default(),
            statics: Default::default(),
            methods: Default::default(),
        };

        let name = Name::from(class.ident.sym.to_string());

        // TODO type params
        // TODO super class & type params
        // TODO implemented interfaces

        for member in class.class.body {
            match member {
                ClassMember::Constructor(ctor) => new.add_constructor(&name, ctor),
                ClassMember::Method(method) => new.add_method(method),
                ClassMember::ClassProp(prop) => new.add_property(prop),
                ClassMember::PrivateMethod(_) => todo!("private methods"),
                ClassMember::PrivateProp(_) => todo!("private properties"),
                ClassMember::TsIndexSignature(_) => todo!("ts index signatures"),
                ClassMember::Empty(_) => (),
            }
        }

        new
    }
}

impl Class {
    fn add_constructor(&mut self, name: &Name, ctor: Constructor) {
        self.constructors.push(Func::ctor(name, ctor.params));
    }

    fn add_method(&mut self, method: ClassMethod) {
        let name = prop_name(method.key);
        let func = Func::from(method.function);

        if method.is_static {
            self.statics.insert(name, func);
        } else {
            self.methods.insert(name, func);
        }
    }

    fn add_property(&mut self, prop: ClassProp) {
        let name = match *prop.key {
            Expr::Ident(i) => i.sym.to_string().into(),
            other => panic!("only ident exprs supported for property keys, found: {:?}", other),
        };
        self.properties.insert(name, prop.type_ann.map(Ty::from).unwrap_or(Ty::Any));
    }
}

impl Debug for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_map();
        f.entries(&self.properties);

        let ctor_name = Name::from("contructor".to_string());
        for ctor in &self.constructors {
            f.entry(&ctor_name, &ctor);
        }

        f.entries(&self.statics).entries(&self.methods).finish()
    }
}

fn prop_name(key: PropName) -> Name {
    match key {
        PropName::Ident(i) => i.sym.to_string().into(),
        PropName::Str(s) => s.value.to_string().into(),
        PropName::Num(n) => n.value.to_string().into(),
        PropName::Computed(c) => todo!("computed property names: {:#?}", c),
    }
}
