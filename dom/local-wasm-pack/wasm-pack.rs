use clap::Parser;

fn main() {
    pretty_env_logger::formatted_timed_builder().init();
    let args = wasm_pack::Cli::parse();
    wasm_pack::PBAR.set_log_level(args.log_level);
    wasm_pack::PBAR.set_quiet(args.quiet);
    wasm_pack::command::run_wasm_pack(args.cmd).unwrap();
}
