use actix::prelude::*;
use actix_web::{
    dev::{MessageBody, Service, ServiceRequest, ServiceResponse, Transform},
    http::uri::Uri,
    middleware, web, App, HttpServer,
};
use actix_web_actors::ws;
use crossbeam::channel::{select, unbounded as chan, Receiver, Sender};
use failure::Error;
use futures::{compat::Compat, future::Ready, TryFutureExt};
use futures01::{Async, Future as OldFuture};
use gumdrop::Options;
use notify::Watcher;
use session::{ChangeWatchSession, Changed};
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    sync::Arc,
    thread::JoinHandle,
};
use tracing::*;

mod session;

#[derive(Debug, Options)]
pub struct ServerOpts {
    #[options(default = "::1")]
    addr: IpAddr,
    #[options(default = "8000")]
    port: u16,
}

impl Default for ServerOpts {
    fn default() -> Self {
        Self {
            // this is so annoying. different cli parsing option maybe?
            addr: "::1".parse().unwrap(),
            port: 8000,
        }
    }
}

impl ServerOpts {
    pub fn run_server(self, root_path: PathBuf) -> Result<(), Error> {
        let (session_tx, session_rx) = chan();
        let watcher = Arc::new(FilesWatcher::new(&root_path, session_rx));

        let root_url = format!("http://[{}]:{}/index.html", &self.addr, &self.port);
        std::thread::spawn(move || {
            opener::open(&root_url).unwrap();
        });

        HttpServer::new(move || {
            let session_tx = session_tx.clone();
            let watcher_middleware = watcher.clone();
            App::new()
                .wrap(middleware::Logger::default())
                .service(web::resource("/ch-ch-ch-changes").route(web::get().to(
                    move |req, stream: web::Payload| {
                        let session_tx = session_tx.clone();
                        let session = ChangeWatchSession::new(session_tx);
                        ws::start(session, &req, stream)
                    },
                )))
                .wrap(watcher_middleware)
                .default_service(actix_files::Files::new("/", &root_path).show_files_listing())
        })
        .bind((self.addr, self.port))
        .unwrap()
        .run()
        .unwrap();
        Ok(())
    }
}

#[allow(clippy::drop_copy, clippy::zero_ptr)] // wtf crossbeam
fn pump_channels(
    root: PathBuf,
    (uri_rx, session_rx): (Receiver<Uri>, Receiver<Addr<ChangeWatchSession>>),
) {
    let (event_tx, event_rx) = chan();
    let mut sessions = Vec::new();

    let mut watcher = notify::watcher(event_tx, std::time::Duration::from_millis(500)).unwrap();

    loop {
        select! {
            recv(uri_rx) -> new_uri => watch_uri(&mut watcher, &root, new_uri),
            recv(session_rx) -> new_session => {
                match new_session {
                    Ok(new_session) => sessions.push(new_session),
                    Err(what) => warn!({ ?what }, "error on session channel"),
                }
            },
            recv(event_rx) -> event => consume_fs_event(&sessions, event),
        }
    }
}

fn consume_fs_event(
    sessions: &[Addr<ChangeWatchSession>],
    event: Result<Result<notify::event::Event, notify::Error>, crossbeam::channel::RecvError>,
) {
    let event = match event {
        Ok(Ok(event)) => event,
        problem => {
            warn!({ ?problem }, "problem receiving fs event");
            return;
        }
    };
    if let Some(path) = event.paths.iter().next() {
        let changed = path.display().to_string();

        info!("file change detected at {}", &changed);

        for session in sessions {
            session.do_send(Changed(changed.clone()));
        }
    }
}

fn watch_uri(
    watcher: &mut notify::RecommendedWatcher,
    root: &Path,
    new_uri: Result<http::Uri, crossbeam::channel::RecvError>,
) {
    let new_uri = match new_uri {
        Ok(new_uri) => new_uri,
        Err(what) => {
            warn!({ ?what }, "error on uri channel");
            return;
        }
    };
    let mut new_path = root.to_path_buf();
    for part in new_uri.path().split('/') {
        new_path.push(part);
    }

    let displayed = new_path.display().to_string();

    if let Err(why) = watcher.watch(&displayed, notify::RecursiveMode::NonRecursive) {
        warn!("couldn't add a watch for {} because {:?}", &displayed, why);
    } else {
        debug!("added/refreshed watch for {}", &displayed);
    }
}

struct FilesWatcher {
    uri_tx: Sender<Uri>,
    joiner: Option<JoinHandle<()>>,
}

impl FilesWatcher {
    fn new(root_path: &Path, session_rx: Receiver<Addr<ChangeWatchSession>>) -> Self {
        let (uri_tx, uri_rx) = chan();
        let root = root_path.to_owned();
        let joiner = Some(std::thread::spawn(|| pump_channels(root, (uri_rx, session_rx))));

        Self { uri_tx, joiner }
    }
}

impl Drop for FilesWatcher {
    fn drop(&mut self) {
        self.joiner.take().unwrap().join().unwrap();
    }
}

impl<S, B> Transform<S> for FilesWatcher
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
{
    type Error = S::Error;
    type Future = Compat<Ready<Result<Self::Transform, Self::InitError>>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Transform = WatchHandle<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ok(WatchHandle { service, uri_tx: self.uri_tx.clone() }).compat()
    }
}

struct WatchHandle<S> {
    service: S,
    uri_tx: Sender<Uri>,
}

impl<S, B> Service for WatchHandle<S>
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
{
    type Error = S::Error;
    type Future = Box<
        dyn OldFuture<
                Item = <<S as Service>::Future as OldFuture>::Item,
                Error = <<S as Service>::Future as OldFuture>::Error,
            > + 'static,
    >;
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;

    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let result = self.service.call(req);
        let uri_tx = self.uri_tx.clone();
        let mapped = result.map(move |response| {
            if response.status().is_success() {
                uri_tx
                    .send(response.request().uri().clone())
                    .unwrap_or_else(|_| warn!("wasn't able to send a new uri to watch"));
            }
            response
        });
        Box::new(mapped)
    }
}
