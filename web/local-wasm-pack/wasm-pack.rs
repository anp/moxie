use structopt::StructOpt;

fn main() {
    pretty_env_logger::formatted_timed_builder().init();
    wasm_pack::command::run_wasm_pack(wasm_pack::Cli::from_args().cmd).unwrap();
}
