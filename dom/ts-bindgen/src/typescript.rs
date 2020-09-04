use crate::{error::TypescriptError, wasm::WasmBindgenImport};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    str::FromStr,
};
use swc_common::BytePos;
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::input::StringInput, Parser, Syntax, TsConfig};

pub fn parse_d_ts(contents: &str) -> Result<Module, TypescriptError> {
    let input = StringInput::new(contents, BytePos(0), BytePos(0));
    let mut parser = Parser::new(
        Syntax::Typescript(TsConfig {
            tsx: false,
            decorators: false,
            dynamic_import: false,
            dts: true,
            no_early_errors: true,
        }),
        input,
        None, // TODO figure out what comments do here?
    );
    Ok(parser.parse_typescript_module()?)
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Name(String);

impl From<String> for Name {
    fn from(s: String) -> Self {
        Name(s.to_string())
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

pub struct TsModule {
    enums: Vec<Enum>,
    classes: Vec<Class>,
    children: BTreeMap<Name, TsModule>,
}

impl Debug for TsModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut module = f.debug_set();

        module.entries(&self.enums);
        module.entries(&self.classes);
        module.entries(&self.children);

        module.finish()
    }
}

impl FromStr for TsModule {
    type Err = TypescriptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(parse_d_ts(s)?))
    }
}

impl From<Module> for TsModule {
    fn from(module: Module) -> Self {
        let mut new = TsModule::blank();
        new.add_module_contents(module.body);
        new
    }
}

impl TsModule {
    fn blank() -> Self {
        Self {
            children: Default::default(),
            classes: Default::default(),
            enums: Default::default(),
        }
    }

    fn add_module_contents(&mut self, contents: Vec<ModuleItem>) {
        for item in contents {
            match item {
                ModuleItem::ModuleDecl(decl) => match decl {
                    ModuleDecl::Import(import) => self.add_import(import),
                    ModuleDecl::ExportDecl(export) => self.add_decl(export.decl),
                    ModuleDecl::ExportNamed(named) => self.add_named_export(named),
                    ModuleDecl::ExportDefaultDecl(default_decl) => {
                        self.add_default_export(default_decl)
                    }
                    ModuleDecl::ExportDefaultExpr(expr) => self.add_default_export_expr(expr),
                    ModuleDecl::ExportAll(export_all) => self.add_export_all(export_all),

                    ModuleDecl::TsImportEquals(ts_import) => self.add_import_equals(ts_import),
                    ModuleDecl::TsExportAssignment(ts_export) => self.add_export_assign(ts_export),
                    ModuleDecl::TsNamespaceExport(ts_ns_export) => {
                        self.add_namespace_export(ts_ns_export)
                    }
                },
                ModuleItem::Stmt(stmt) => self.add_stmt(stmt),
            }
        }
    }

    fn add_import(&mut self, _import: ImportDecl) {
        println!("TODO support imports");
    }

    pub fn add_named_export(&mut self, _named: NamedExport) {
        println!("TODO support re-exports");
    }

    pub fn add_default_export(&mut self, _default_decl: ExportDefaultDecl) {
        println!("TODO export default decl");
    }

    pub fn add_default_export_expr(&mut self, _default_expr: ExportDefaultExpr) {
        println!("TODO export default expr");
    }

    pub fn add_export_all(&mut self, _export_all: ExportAll) {
        println!("TODO export all");
    }

    pub fn add_import_equals(&mut self, _ts_import: TsImportEqualsDecl) {
        println!("TODO ts import");
    }

    pub fn add_export_assign(&mut self, _ts_export: TsExportAssignment) {
        println!("TODO export assignment");
    }

    pub fn add_namespace_export(&mut self, _ts_ns_export: TsNamespaceExportDecl) {
        println!("TODO typescript namespace export");
    }

    fn add_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Decl(decl) => self.add_decl(decl),
            _ => {
                println!("TODO non-decl statements");
            }
        }
    }

    fn add_decl(&mut self, decl: Decl) {
        match decl {
            Decl::Class(class) => self.add_class(class),
            Decl::TsInterface(interface) => self.add_interface(interface),
            Decl::TsTypeAlias(alias) => self.add_alias(alias),
            Decl::TsEnum(ts_enum) => self.add_enum(ts_enum),
            Decl::TsModule(TsModuleDecl { id, body, .. }) => self.add_ts_module(id, body),
            Decl::Fn(fun) => self.add_function(fun),
            Decl::Var(var) => self.add_var(var),
        }
    }

    fn add_var(&mut self, var: VarDecl) {
        for decl in var.decls {
            let name = match decl.name {
                Pat::Ident(n) => Name::from(n.sym.to_string()),
                other => {
                    todo!("i guess implement support for assignments to {:?}", other);
                }
            };

            if let Some(init) = decl.init {
                todo!("i guess support initializing {:?} {:?} {:?}", &var.kind, name, init);
            }
        }
    }

    fn add_function(&mut self, _fun: FnDecl) {
        println!("TODO functions");
    }

    fn add_class(&mut self, class: ClassDecl) {
        self.classes.push(class.into());
    }

    fn add_interface(&mut self, _interface: TsInterfaceDecl) {
        println!("TODO interfaces");
    }

    fn add_alias(&mut self, _alias: TsTypeAliasDecl) {
        println!("TODO ts aliases");
    }

    fn add_enum(&mut self, decl: TsEnumDecl) {
        self.enums.push(decl.into());
    }

    fn add_ts_module(&mut self, id: TsModuleName, body: Option<TsNamespaceBody>) {
        let new_child = self.children.entry(module_name(&id)).or_insert_with(TsModule::blank);

        match body {
            Some(TsNamespaceBody::TsModuleBlock(block)) => {
                new_child.add_module_contents(block.body);
            }
            Some(TsNamespaceBody::TsNamespaceDecl(TsNamespaceDecl { id, body, .. })) => {
                new_child.add_ts_module(TsModuleName::Ident(id), Some(*body));
            }
            None => (),
        }
    }
}

struct Class {
    name: Name,
    constructors: Vec<Ctor>,
    methods: Vec<Method>,
}

impl From<ClassDecl> for Class {
    fn from(class: ClassDecl) -> Self {
        let mut new = Class {
            name: class.ident.sym.to_string().into(),
            constructors: Default::default(),
            methods: Default::default(),
        };

        // TODO type params
        // TODO super class & type params
        // TODO implemented interfaces

        for member in class.class.body {
            match member {
                ClassMember::Constructor(ctor) => new.add_constructor(ctor),
                ClassMember::Method(method) => new.add_method(method),
                ClassMember::ClassProp(_) => {
                    println!("TODO figure out if we care about class properties")
                }
                ClassMember::PrivateMethod(_) => {
                    println!("TODO figure out if we care about private methods")
                }
                ClassMember::PrivateProp(_) => {
                    println!("TODO figure out if we care about private properties")
                }
                ClassMember::TsIndexSignature(_) => println!("TODO figure out ts index signatures"),
                ClassMember::Empty(_) => (),
            }
        }

        new
    }
}

impl Class {
    fn add_constructor(&mut self, ctor: Constructor) {
        self.constructors.push(Ctor {
            for_class: self.name.clone(),
            params: ctor
                .params
                .into_iter()
                .map(|param| match param {
                    ParamOrTsParamProp::Param(p) => p.into(),
                    ParamOrTsParamProp::TsParamProp(t) => {
                        todo!("support ts 'param props' from swc ast {:#?}", t);
                    }
                })
                .collect(),
        });
    }

    fn add_method(&mut self, method: ClassMethod) {
        self.methods.push(method.into());
    }
}

impl Debug for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut f = f.debug_struct(&self.name.0);

        for ctor in &self.constructors {
            f.field("constructor", &ctor); // TODO better way to structure these?
        }

        for method in &self.methods {
            f.field("method", &method); // TODO better way to structure these?
        }

        f.finish()
    }
}

struct Ctor {
    for_class: Name,
    params: Vec<TsParam>,
}

impl Debug for Ctor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let new = format!("new {}", &self.for_class);
        let mut f = f.debug_tuple(&new);

        for p in &self.params {
            f.field(p);
        }

        f.finish()
    }
}

struct Method {
    name: Name,
    params: Vec<TsParam>,
    returns: Option<Ty>,
}

impl From<ClassMethod> for Method {
    fn from(method: ClassMethod) -> Self {
        let name = prop_name(method.key);

        let mut params = vec![];
        // TODO iterate over params

        let returns = None; // TODO

        Self { name, params, returns }
    }
}

impl Debug for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut tup = f.debug_tuple(&self.name.0);
        for p in &self.params {
            tup.field(p);
        }
        tup.finish()?;

        if let Some(ret) = &self.returns { write!(f, " -> {:?}", ret) } else { Ok(()) }
    }
}

struct TsParam {
    name: Name,
    optional: bool,
    ty: Ty,
}

impl From<Param> for TsParam {
    fn from(param: Param) -> TsParam {
        match param.pat {
            Pat::Ident(i) => {
                let name = i.sym.to_string().into();
                let ty = i.type_ann.into();
                Self { name, ty, optional: i.optional }
            }
            other => todo!("other parameter types like {:#?}", other),
        }
    }
}

impl Debug for TsParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let optional = if self.optional { "?" } else { "" };
        write!(f, "{}{}: {:?}", &self.name, optional, &self.ty)
    }
}

#[derive(Debug)]
struct Ty {
    // TODO figure out a repr for not-yet-resolved types
}

impl From<TsType> for Ty {
    fn from(_ty: TsType) -> Ty {
        // TODO ...stuff
        Ty {}
    }
}

impl From<Option<TsTypeAnn>> for Ty {
    fn from(ann: Option<TsTypeAnn>) -> Ty {
        if let Some(ann) = ann {
            (*ann.type_ann).into()
        } else {
            Ty {} // TODO make an Any/universal type i guess?
        }
    }
}

#[derive(Debug)]
struct Enum {
    name: Name,
    members: Vec<Name>,
}

impl From<TsEnumDecl> for Enum {
    fn from(decl: TsEnumDecl) -> Self {
        let name = decl.id.sym.to_string().into();
        let members = decl
            .members
            .into_iter()
            .map(|member| {
                let member_name = member_name(&member.id);
                if member.init.is_some() {
                    println!("TODO enum member init {:?}.{:?}", &name, &member_name);
                }
                member_name
            })
            .collect();
        Enum { name, members }
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

fn module_name(id: &TsModuleName) -> Name {
    match id {
        TsModuleName::Ident(i) => &i.sym,
        TsModuleName::Str(s) => &s.value,
    }
    .to_string()
    .into()
}

fn prop_name(key: PropName) -> Name {
    match key {
        PropName::Ident(i) => i.sym.to_string().into(),
        PropName::Str(s) => s.value.to_string().into(),
        PropName::Num(n) => n.value.to_string().into(),
        PropName::Computed(c) => todo!("support computed property names: {:#?}", c),
    }
}

impl TsModule {
    pub fn import_with_wasm_bindgen(&self) -> Result<WasmBindgenImport, TypescriptError> {
        todo!("{:#?}", self)
    }
}
