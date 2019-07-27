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
//! futures = { package = "futures-preview", version = "0.3.0-alpha.17", features = [ "async-await", "compat", "nightly" ] }
//! futures01 = { version = "0.1", package = "futures" }
//! gumdrop = "0.6"
//! notify = "5.0.0-pre.1"
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
    crossbeam::channel::{select, unbounded as chan, Receiver, Sender},
    futures::{
        compat::Compat,
        future::{BoxFuture, Ready},
        TryFutureExt,
    },
    futures01::Async,
    gumdrop::Options,
    std::{
        collections::BTreeSet,
        fmt::Debug,
        net::IpAddr,
        path::{Path, PathBuf},
        sync::Arc,
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
    let watcher = Arc::new(FilesWatcher::new(&root_path));

    HttpServer::new(move || {
        let watcher_middleware = watcher.clone();
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
            .wrap(watcher_middleware)
            .default_service(actix_files::Files::new("/", &root_path).show_files_listing())
    })
    .bind((config.addr, config.port))
    .unwrap()
    .run()
    .unwrap();
}

struct FilesWatcher {
    path: PathBuf,
    paths_of_interest: BTreeSet<PathBuf>,
    path_tx: Sender<PathBuf>,
    event_rx: Receiver<Result<notify::event::Event, notify::Error>>,
    sessions: Vec<actix::Addr<ChangeWatchSession>>,
}

impl FilesWatcher {
    fn new(root_path: &Path) -> Self {
        let (path_tx, path_rx) = chan();
        let (event_tx, event_rx) = chan();
        let remote_event_rx = event_rx.clone();
        std::thread::spawn(move || {
            let watcher = notify::watcher(event_tx, std::time::Duration::from_millis(500)).unwrap();
            let path_rx = path_rx;
            let event_rx = remote_event_rx;
            let mut paths_of_interest = BTreeSet::new();
            loop {
                select! {
                    recv(event_rx) -> event => {
                        let event = event.expect("filesystem events should be live");
                        println!("not multiplexing events just yet");
                    },
                    recv(path_rx) -> new_path => {
                        paths_of_interest.insert(new_path.expect("path events should be live"));
                    },
                }
            }
        });

        Self {
            path: root_path.to_owned(),
            event_rx,
            path_tx,
            paths_of_interest: Default::default(),
            sessions: Default::default(),
        }
    }
}

impl<S> Transform<S> for FilesWatcher
where
    S: Service,
    S::Request: Debug,
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Transform = WatchHandle<S>;
    type InitError = ();
    type Future = Compat<Ready<Result<Self::Transform, Self::InitError>>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ok(WatchHandle { service }).compat()
    }
}

struct WatchHandle<S> {
    service: S,
}

impl<S> Service for WatchHandle<S>
where
    S: Service,
    S::Request: Debug,
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        // TODO inspect the request to see whether we want to add it to watch paths
        debug!("watch handle serving request {:#?}", &req);
        self.service.call(req)
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
