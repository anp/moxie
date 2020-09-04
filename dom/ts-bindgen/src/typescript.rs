use crate::{error::TypescriptError, wasm::WasmBindgenImport};
use std::{collections::BTreeMap, fmt::Debug, str::FromStr};
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

#[derive(Debug)]
pub struct TsModule {
    children: BTreeMap<String, TsModule>,
    classes: BTreeMap<String, Class>,
    enums: BTreeMap<String, Enum>,
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
                Pat::Ident(n) => n.sym.to_string(),
                other => {
                    todo!("i guess implement support for assignments to {:?}", other);
                }
            };

            if let Some(init) = decl.init {
                todo!("i guess support initializing {:?} {} {:?}", &var.kind, name, init);
            }
        }
    }

    fn add_function(&mut self, _fun: FnDecl) {
        println!("TODO functions");
    }

    fn add_class(&mut self, class: ClassDecl) {
        let class: Class = class.into();
        let name = class.name.clone();
        self.classes.insert(name, class);
    }

    fn add_interface(&mut self, _interface: TsInterfaceDecl) {
        println!("TODO interfaces");
    }

    fn add_alias(&mut self, _alias: TsTypeAliasDecl) {
        println!("TODO ts aliases");
    }

    fn add_enum(&mut self, decl: TsEnumDecl) {
        let new: Enum = decl.into();
        let name = new.name.clone();
        self.enums.insert(name, new);
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

#[derive(Debug)]
struct Class {
    name: String,
    constructor: Option<()>,
    properties: BTreeMap<String, ()>,
    methods: BTreeMap<String, ()>,
}

impl From<ClassDecl> for Class {
    fn from(class: ClassDecl) -> Self {
        let mut new = Class {
            name: class.ident.sym.to_string(),
            constructor: None,
            properties: Default::default(),
            methods: Default::default(),
        };

        // TODO type params
        // TODO super class & type params
        // TODO implemented interfaces

        for member in class.class.body {
            match member {
                ClassMember::Constructor(ctor) => new.set_constructor(ctor),
                ClassMember::Method(method) => new.add_method(method),
                ClassMember::ClassProp(prop) => new.add_property(prop),
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
    fn set_constructor(&mut self, ctor: Constructor) {
        println!("TODO constructors");
    }

    fn add_method(&mut self, method: ClassMethod) {
        println!("TODO class methods");
    }

    fn add_property(&mut self, prop: ClassProp) {
        println!("TODO class properties");
    }
}

#[derive(Debug)]
struct Enum {
    name: String,
    members: Vec<String>,
}

impl From<TsEnumDecl> for Enum {
    fn from(decl: TsEnumDecl) -> Self {
        let name = decl.id.sym.to_string();
        let members = decl
            .members
            .into_iter()
            .map(|member| {
                let member_name = member_name(&member.id);
                if member.init.is_some() {
                    println!("TODO enum member init {}.{}", &name, &member_name);
                }
                member_name
            })
            .collect();
        Enum { name, members }
    }
}

fn member_name(id: &TsEnumMemberId) -> String {
    match id {
        TsEnumMemberId::Ident(i) => &i.sym,
        TsEnumMemberId::Str(s) => &s.value,
    }
    .to_string()
}

fn module_name(id: &TsModuleName) -> String {
    match id {
        TsModuleName::Ident(i) => &i.sym,
        TsModuleName::Str(s) => &s.value,
    }
    .to_string()
}

impl TsModule {
    pub fn import_with_wasm_bindgen(&self) -> Result<WasmBindgenImport, TypescriptError> {
        todo!("{:#?}", self)
    }
}
