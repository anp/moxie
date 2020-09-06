use quote::ToTokens;
use std::{
    env, fs,
    io::{prelude::*, Result as IoResult},
    path::Path,
    process::{Command, Stdio},
};

pub mod error;
pub mod typescript;
pub mod wasm;

use error::BindingError;
use typescript::TsModule;

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
/// For other uses see [`typescript::TsModule`] and TODO add back type for
/// wasm-bindgen.
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
    let contents = make_bindings(&input)?;
    fs::write(output_path, contents).map_err(BindingError::WriteOutFile)?;
    Ok(())
}

/// Parses `input` as a typescript definitions module and generates a module of
/// Rust bindings to it, returning the generated Rust code as a string.
pub fn make_bindings(input: &str) -> Result<String, BindingError> {
    let defs: TsModule = input.parse()?;
    let imports = defs.import_with_wasm_bindgen()?;
    let output = imports.to_token_stream().to_string();
    if let Ok(formatted) = rustfmt(&output) { Ok(formatted) } else { Ok(output) }
}

fn rustfmt(code: &str) -> IoResult<String> {
    let mut cmd = Command::new("rustfmt").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
    let mut stdin = cmd.stdin.take().unwrap();
    stdin.write_all(code.as_bytes())?;

    let output = cmd.wait_with_output()?;
    if !output.status.success() {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "rustfmt command failed"))
    } else {
        Ok(String::from_utf8(output.stdout).expect("rustfmt always returns utf8"))
    }
}
