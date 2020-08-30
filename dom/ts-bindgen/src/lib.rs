use std::{
    env,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs, io,
    path::Path,
    str::FromStr,
};
use swc_common::BytePos;
use swc_ecma_ast::{Decl, Module, ModuleDecl, ModuleItem, Stmt};
use swc_ecma_parser::{
    error::Error as SwcError, lexer::input::StringInput, Parser, Syntax, TsConfig,
};
use thiserror::Error;

/// Parses the typescript definitions at `input_path` relative to
/// `CARGO_MANIFEST_DIR`, converts them to `wasm-bindgen` import statements, and
/// writes those to `output_path` relative to `OUT_DIR`.
///
/// In `build.rs`:
///
/// ```ignore
/// fn main() {
///     ts_bindgen::d_ts_buildscript("relative/path/to/index.d.ts", "index.d.rs").unwrap();
/// }
/// ```
///
/// In `lib.rs`:
///
/// ```ignore
/// include!(concat!(env!("OUT_DIR"), "/index.d.rs"));
/// ```
///
/// For other uses see [`Definitions`] and [`WasmBindgenImports`].
pub fn d_ts_buildscript(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), BindingError> {
    let crate_dir = env::var_os("CARGO_MANIFEST_DIR").expect("must be run from a build script");
    let input_path = Path::new(&crate_dir).join(input_path.as_ref());

    let out_dir = env::var_os("OUT_DIR").expect("must be run from a build script");
    let output_path = Path::new(&out_dir).join(output_path.as_ref());

    println!("cargo:rerun-if-changed={}", input_path.display());
    let input = fs::read_to_string(input_path).map_err(BindingError::ReadInputFile)?;

    let defs: Definitions = input.parse()?;
    let imports = defs.import_with_wasm_bindgen()?;
    let contents = imports.to_string();

    fs::write(output_path, contents).map_err(BindingError::WriteOutFile)?;
    Ok(())
}

#[derive(Debug)]
pub struct Definitions {
    defs: Vec<Defn>,
}

impl FromStr for Definitions {
    type Err = BindingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = StringInput::new(s, BytePos(0), BytePos(0));
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
        let module = parser.parse_typescript_module()?;
        Self::visit(module)
    }
}

impl Definitions {
    pub fn visit(module: Module) -> Result<Definitions, BindingError> {
        let mut defs = vec![];
        for item in module.body {
            match item {
                ModuleItem::ModuleDecl(decl) => match decl {
                    ModuleDecl::Import(import) => println!("imports?"),
                    ModuleDecl::ExportDecl(decl) => println!("export decl"),
                    ModuleDecl::ExportNamed(named) => println!("export named"),
                    ModuleDecl::ExportDefaultDecl(default_decl) => println!("export default decl"),
                    ModuleDecl::ExportDefaultExpr(default_expr) => println!("export default expr"),
                    ModuleDecl::ExportAll(export_all) => println!("export all"),
                    ModuleDecl::TsImportEquals(ts_import) => println!("ts import ="),
                    ModuleDecl::TsExportAssignment(ts_export) => println!("ts export assignment"),
                    ModuleDecl::TsNamespaceExport(ts_ns_export) => println!("ts ns export"),
                },
                ModuleItem::Stmt(Stmt::Decl(decl)) => {
                    println!("decl");
                }
                ModuleItem::Stmt(stmt) => {
                    println!("skipping statement TODO what to do with these? {:#?}", stmt);
                }
            }
        }
        Ok(Self { defs })
    }

    pub fn import_with_wasm_bindgen(&self) -> Result<WasmBindgenImports, BindingError> {
        Ok(WasmBindgenImports {})
    }
}

#[derive(Debug)]
enum Defn {
    // TODO populate these!
    TypeAlias,
    Interface,
    Class,
    Enum,
    Import,
    Namespace,
    Module,
    Function,
}

pub struct WasmBindgenImports {
    // TODO
}

impl Display for WasmBindgenImports {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        // write!(f, "")
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum BindingError {
    #[error("reading input failed")]
    ReadInputFile(io::Error),

    #[error("parsing")]
    ParseInputFile { source: ParseError },

    #[error("writing output failed")]
    WriteOutFile(io::Error),
}

impl From<SwcError> for BindingError {
    fn from(e: SwcError) -> Self {
        BindingError::ParseInputFile { source: e.into() }
    }
}

#[derive(Error)]
pub struct ParseError {
    e: SwcError,
}

impl ParseError {
    fn to_stderr(&self) {
        use swc_common::errors::{ColorConfig, EmitterWriter, Handler, HandlerFlags};
        let emitter = EmitterWriter::stderr(
            ColorConfig::Auto,
            None,  // source maps
            false, // short_message
            true,  // teach
        );
        let handler = Handler::with_emitter_and_flags(Box::new(emitter), HandlerFlags {
            can_emit_warnings: true,
            treat_err_as_bug: true,
            dont_buffer_diagnostics: true,
            report_delayed_bugs: false,
            external_macro_backtrace: false, // lol
        });
        self.e.clone().into_diagnostic(&handler).emit();
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.to_stderr();
        write!(f, "ParseError(see stderr)")
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.to_stderr();
        write!(f, "Parsing error (see stderr for details)")
    }
}

impl From<SwcError> for ParseError {
    fn from(e: SwcError) -> Self {
        ParseError { e }
    }
}
