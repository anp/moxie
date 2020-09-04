use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};
use swc_ecma_ast::{Class as AstClass, ClassMember, ClassMethod, Constructor, PropName};

use super::{Func, Name, Ty};

pub struct Class {
    ty: Ty,
    constructors: Vec<Func>,
    statics: BTreeMap<Name, Func>,
    methods: BTreeMap<Name, Func>,
}

impl From<AstClass> for Class {
    fn from(class: AstClass) -> Self {
        let mut new = Class {
            ty: Ty::any(), // TODO a real type?
            constructors: Default::default(),
            statics: Default::default(),
            methods: Default::default(),
        };

        // TODO type params
        // TODO super class & type params
        // TODO implemented interfaces

        for member in class.body {
            match member {
                ClassMember::Constructor(ctor) => new.add_constructor(ctor),
                ClassMember::Method(method) => new.add_method(method),
                ClassMember::ClassProp(_) => println!("TODO class properties?"),
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
    pub fn ty(&self) -> &Ty {
        &self.ty
    }

    fn add_constructor(&mut self, ctor: Constructor) {
        self.constructors.push(Func::ctor(&self, ctor.params));
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
}

impl Debug for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_map();

        let ctor_name = Name::from("contructor".to_string());
        for ctor in &self.constructors {
            f.entry(&ctor_name, &ctor);
        }

        f.entries(&self.statics);
        f.entries(&self.methods);
        f.finish()
    }
}

fn prop_name(key: PropName) -> Name {
    match key {
        PropName::Ident(i) => i.sym.to_string().into(),
        PropName::Str(s) => s.value.to_string().into(),
        PropName::Num(n) => n.value.to_string().into(),
        PropName::Computed(c) => todo!("support computed property names: {:#?}", c),
    }
}
