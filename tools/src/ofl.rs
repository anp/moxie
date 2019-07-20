#![feature(async_await)]

use {failure::Error, gumdrop::Options, tracing::*};

mod serve;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), Error> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = Args::parse_args_default_or_exit();

    match args.command.expect("a subcommand is required") {
        Command::Serve(serve_opts) => {
            serve_opts.run().await?;
        }
    }

    Ok(())
}

#[derive(Debug, Options)]
struct Args {
    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "watch the provided project for changes and serve artifacts")]
    Serve(serve::ServeOpts),
}
