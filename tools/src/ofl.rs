#![feature(async_await)]

use {failure::Error, gumdrop::Options, std::path::PathBuf, tracing::*, workspace::Workspace};

mod serve;
mod workspace;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), Error> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = Args::parse_args_default_or_exit();

    let workspace = Workspace::new(args.workspace)?;
    match args.command.expect("a subcommand is required") {
        Command::Serve(serve_opts) => {
            serve_opts.run(workspace).await?;
        }
    }

    Ok(())
}

#[derive(Debug, Options)]
struct Args {
    #[options(help = "print help message")]
    help: bool,
    #[options(
        help = "a path within the workspace, preferably the root",
        default_expr = "std::env::current_dir().unwrap()"
    )]
    workspace: PathBuf,
    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "watch the provided project for changes and serve artifacts")]
    Serve(serve::ServeOpts),
}
