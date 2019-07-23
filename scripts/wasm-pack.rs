//! ```cargo
//! [package]
//! edition = "2018"
//!
//! [dependencies]
//! pretty_env_logger = "0.3"
//! structopt = "0.2"
//! wasm-pack = "0.8"
//! ```

use structopt::StructOpt;

fn main() {
    pretty_env_logger::formatted_timed_builder().init();
    wasm_pack::command::run_wasm_pack(wasm_pack::Cli::from_args().cmd).unwrap();
}