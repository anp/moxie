//! Serve the project directory from a local HTTP server.
//!
//! ```cargo
//! [package]
//! edition = "2018"
//!
//! [dependencies]
//! actix = "0.8"
//! actix-files = "0.1"
//! actix-service = "0.4"
//! actix-web = "1"
//! actix-web-actors = "1"
//! futures-preview = { version = "0.3.0-alpha.17", features = [ "async-await", "compat", "nightly" ] }
//! gumdrop = "0.6"
//! parking_lot = "0.9"
//! pretty_env_logger = "0.3"
//! tracing = { version = "0.1", features = [ "log" ] }
//! ```
#![feature(async_await)]

use {
    actix::prelude::*,
    actix_service::ServiceExt,
    actix_web::{middleware, web, App, HttpServer},
    actix_web_actors::ws,
    gumdrop::Options,
    std::{
        net::IpAddr,
        path::{Path, PathBuf},
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

    let change_socket = |req, stream: web::Payload| {
        ws::start(
            ChangeWatchSession {
                last_heartbeat: Instant::now(),
            },
            &req,
            stream,
        )
    };

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/ch-ch-ch-changes").route(web::get().to(change_socket)))
            .service(actix_files::Files::new("/", &root_path).show_files_listing())
    })
    .bind((config.addr, config.port))
    .unwrap()
    .run()
    .unwrap();
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
