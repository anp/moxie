//! Serve the project directory from a local HTTP server.
//!
//! ```cargo
//! [package]
//! edition = "2018"
//!
//! [dependencies]
//! futures-preview = { version = "0.3.0-alpha.17", features = [ "async-await", "compat", "nightly" ] }
//! gumdrop = "0.6"
//! http = "0.1"
//! hyper = "0.12"
//! hyper-staticfile = "0.3"
//! pretty_env_logger = "0.3"
//! runtime = "0.3.0-alpha.6"
//! runtime-tokio = "0.3.0-alpha.5"
//! sfz = "*"
//! tracing = { version = "0.1", features = [ "log" ] }
//! ```
#![feature(async_await)]

use {
    futures::{compat::Future01CompatExt, TryFutureExt},
    gumdrop::Options,
    http::Request,
    hyper::Body,
    hyper_staticfile::{Static, StaticFuture},
    std::io::Error,
    std::{net::IpAddr, path::Path},
    tracing::*,
};

#[derive(Debug, Options)]
struct Config {
    help: bool,
    #[options(default = "::1")]
    addr: IpAddr,
    #[options(default = "8000")]
    port: u16,
}

#[runtime::main(runtime_tokio::Tokio)]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Warn)
        .filter_module(module_path!(), log::LevelFilter::Debug)
        .init();
    debug!("logging init'd");

    let scripts_path = std::env::var("CARGO_SCRIPT_BASE_PATH").unwrap();
    let root_path = Path::new(&scripts_path).parent().unwrap().to_path_buf();

    let config = Config::parse_args_default_or_exit();

    let addr = (config.addr, config.port).into();
    let make_service = hyper::service::make_service_fn(move |_| {
        let svc = MainService::new(&root_path);
        Box::pin(async { Ok::<_, hyper::Error>(svc) }).compat()
    });
    let server = hyper::Server::bind(&addr).serve(make_service);
    info!("server running on http://{}/", addr);
    server.compat().await.unwrap();
}

struct MainService {
    static_: Static,
}

impl MainService {
    fn new(project_root: &Path) -> MainService {
        MainService {
            static_: Static::new(project_root),
        }
    }
}

impl hyper::service::Service for MainService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = StaticFuture<Body>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.static_.serve(req)
    }
}
