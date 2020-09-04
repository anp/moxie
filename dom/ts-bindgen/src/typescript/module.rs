use crate::{error::TypescriptError, wasm::WasmBindgenImport};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};
use swc_ecma_ast::{
    Decl, Ident, Module, ModuleDecl, ModuleItem, Stmt, TsModuleDecl, TsModuleName, TsNamespaceBody,
    TsNamespaceDecl,
};

use super::{class::Class, enums::Enum, func::Func, interface::Interface, name::Name, ty::Ty};

pub struct TsModule {
    variables: BTreeMap<Name, Ty>,
    aliases: BTreeMap<Name, Ty>,
    enums: BTreeMap<Name, Enum>,
    classes: BTreeMap<Name, Class>,
    interfaces: BTreeMap<Name, Interface>,
    functions: BTreeMap<Name, Func>,
    children: BTreeMap<Name, TsModule>,
}

impl Debug for TsModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_map()
            .entries(&self.variables)
            .entries(&self.enums)
            .entries(&self.classes)
            .entries(&self.interfaces)
            .entries(&self.functions)
            .entries(&self.children)
            .finish()
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
            aliases: Default::default(),
            children: Default::default(),
            classes: Default::default(),
            interfaces: Default::default(),
            enums: Default::default(),
            functions: Default::default(),
            variables: Default::default(),
        }
    }

    fn add_module_contents(&mut self, contents: Vec<ModuleItem>) {
        for item in contents {
            match item {
                ModuleItem::ModuleDecl(decl) => match decl {
                    ModuleDecl::Import(_) => todo!("support imports"),
                    ModuleDecl::ExportNamed(_) => todo!("support re-exports"),
                    ModuleDecl::ExportDefaultDecl(_) => todo!("export default decl"),
                    ModuleDecl::ExportDefaultExpr(_) => todo!("export default expr"),
                    ModuleDecl::ExportAll(_) => todo!("export all"),
                    ModuleDecl::TsImportEquals(_) => todo!("ts import"),
                    ModuleDecl::TsExportAssignment(_) => todo!("export assignment"),
                    ModuleDecl::TsNamespaceExport(_) => todo!("typescript namespace export"),
                    ModuleDecl::ExportDecl(export) => self.add_decl(export.decl),
                },
                ModuleItem::Stmt(Stmt::Decl(decl)) => self.add_decl(decl),
                ModuleItem::Stmt(s) => todo!("support non-decl statements? {:#?}", s),
            }
        }
    }

    fn add_decl(&mut self, decl: Decl) {
        match decl {
            Decl::Class(class) => {
                self.classes.insert(class.ident.sym.to_string().into(), class.into());
            }
            Decl::TsInterface(interface) => {
                self.interfaces.insert(interface.id.sym.to_string().into(), interface.body.into());
            }
            Decl::TsTypeAlias(alias) => {
                self.aliases.insert(alias.id.sym.to_string().into(), From::from(*alias.type_ann));
            }
            Decl::TsEnum(decl) => {
                self.enums.insert(decl.id.sym.to_string().into(), decl.members.into());
            }
            Decl::Fn(fun) => {
                self.functions.insert(fun.ident.sym.to_string().into(), fun.function.into());
            }
            Decl::Var(var) => {
                for decl in var.decls {
                    let Ident { sym, type_ann, .. } = decl.name.expect_ident();
                    let (name, ty) =
                        (Name::from(sym.to_string()), type_ann.map(Ty::from).unwrap_or(Ty::Any));
                    self.variables.insert(name, ty);
                }
            }
            Decl::TsModule(TsModuleDecl { id, body, .. }) => self.add_ts_module(id, body),
        }
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

    pub fn import_with_wasm_bindgen(&self) -> Result<WasmBindgenImport, TypescriptError> {
        todo!("{:?}", self)
    }
}

fn module_name(id: &TsModuleName) -> Name {
    match id {
        TsModuleName::Ident(i) => &i.sym,
        TsModuleName::Str(s) => &s.value,
    }
    .to_string()
    .into()
}
