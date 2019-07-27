//! Serve the project directory from a local HTTP server.
//!
//! ```cargo
//! [package]
//! edition = "2018"
//!
//! [dependencies]
//! actix = "0.8"
//! actix-files = "0.1"
//! actix-web = "1"
//! actix-web-actors = "1"
//! crossbeam = "0.7"
//! futures01 = { version = "0.1", package = "futures" }
//! futures = { package = "futures-preview", version = "0.3.0-alpha.17", features = [ "async-await", "compat", "nightly" ] }
//! gumdrop = "0.6"
//! notify = "5.0.0-pre.1"
//! parking_lot = "0.9"
//! pretty_env_logger = "0.3"
//! tracing = { version = "0.1", features = [ "log" ] }
//! ```
#![feature(async_await)]

use {
    actix::prelude::*,
    actix_web::{
        dev::{Service, ServiceRequest, ServiceResponse, Transform},
        middleware, web, App, Error, HttpServer,
    },
    actix_web_actors::ws,
    futures::{compat::Compat, future::BoxFuture},
    // crossbeam::channel::unbounded as chan,
    futures01::Async,
    gumdrop::Options,
    // notify::Watcher,
    // parking_lot::Mutex,
    std::{
        net::IpAddr,
        path::{Path, PathBuf},
        // sync::Arc,
        time::{Duration, Instant},
    },
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

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Warn)
        .filter_module(module_path!(), log::LevelFilter::Debug)
        .init();
    debug!("logging init'd");

    let scripts_path = std::env::var("CARGO_SCRIPT_BASE_PATH").unwrap();
    let root_path = Path::new(&scripts_path).parent().unwrap().to_path_buf();
    let config = Config::parse_args_default_or_exit();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/ch-ch-ch-changes").route(web::get().to(
                |req, stream: web::Payload| {
                    ws::start(
                        ChangeWatchSession {
                            last_heartbeat: Instant::now(),
                        },
                        &req,
                        stream,
                    )
                },
            )))
            .wrap(FilesWatcher::new(&root_path))
            .default_service(actix_files::Files::new("/", &root_path).show_files_listing())
    })
    .bind((config.addr, config.port))
    .unwrap()
    .run()
    .unwrap();
}

struct FilesWatcher {
    path: PathBuf,
    paths_of_interest: Vec<PathBuf>,
    watcher: notify::RecommendedWatcher,
    sessions: Vec<actix::Addr<ChangeWatchSession>>,
    joiner: Option<()>,
}

impl FilesWatcher {
    fn new(root_path: &Path) -> Self {
        unimplemented!()

        // let joiner = self.joiner.as_mut();

        // if joiner.is_none() {
        //     let (tx, _rx) = chan();

        //     std::thread::spawn(|| {
        //         let watcher = Arc::new(Mutex::new(
        //             notify::watcher(tx, std::time::Duration::from_millis(500)).unwrap(),
        //         ));
        //         watcher
        //             .lock()
        //             .watch(root, notify::RecursiveMode::Recursive)
        //             .unwrap();
        //     });
        // }

        // self.joiner.as_ref().unwrap().0.clone()
    }
}

impl<S> Transform<S> for FilesWatcher {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = WatchHandle;
    type InitError = ();
    type Future = Compat<BoxFuture<'static, Result<Self::Transform, Self::InitError>>>;
    fn new_transform(&self, service: S) -> Self::Future {
        unimplemented!()
    }
}

struct WatchHandle {
    request: ServiceRequest,
}

impl WatchHandle {
    fn add_session(&self, session: Addr<ChangeWatchSession>) {
        unimplemented!()
    }

    fn watch_path(&self, path: PathBuf) {
        unimplemented!()
    }

    fn start_watching(&self, request: &ServiceRequest) {
        unimplemented!()
    }
}

impl Service for WatchHandle {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Compat<BoxFuture<'static, Result<Self::Response, Self::Error>>>;
    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        unimplemented!()
    }
    fn call(&mut self, req: Self::Request) -> Self::Future {
        unimplemented!()
    }
}

struct ChangeWatchSession {
    last_heartbeat: Instant,
}

impl ChangeWatchSession {
    fn tick_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ChangeWatchSession {
    fn handle(&mut self, msg: ws::Message, cx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                self.tick_heartbeat();
                cx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.tick_heartbeat();
            }
            ws::Message::Close(_) => {
                cx.stop();
            }
            ws::Message::Nop => (),
            ws::Message::Text(text) => {
                self.tick_heartbeat();
                debug!("ignoring text ws message {:?}", text);
            }
            ws::Message::Binary(_bin) => {
                self.tick_heartbeat();
                debug!("ignoring binary ws message");
            }
        }
    }
}

impl Actor for ChangeWatchSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, cx: &mut Self::Context) {
        cx.run_interval(Duration::from_secs(3), |session, cx| {
            if Instant::now().duration_since(session.last_heartbeat) > Duration::from_secs(10) {
                info!("ws change event client timed out, disconnecting");
                cx.stop();
                return;
            }
            cx.ping("");
        });
    }
}
