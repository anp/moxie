use {
    actix::prelude::*,
    actix_web::{
        dev::{MessageBody, Service, ServiceRequest, ServiceResponse, Transform},
        http::uri::Uri,
        middleware, web, App, HttpServer,
    },
    actix_web_actors::ws,
    crossbeam::channel::{select, unbounded as chan, Receiver, Sender},
    failure::Error,
    futures::{compat::Compat, future::Ready, TryFutureExt},
    futures01::{Async, Future as OldFuture},
    gumdrop::Options,
    notify::Watcher,
    std::{
        net::IpAddr,
        path::{Path, PathBuf},
        sync::Arc,
        time::{Duration, Instant},
    },
    tracing::*,
};

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

        self.spawn_opener();

        HttpServer::new(move || {
            let session_tx = session_tx.clone();
            let watcher_middleware = watcher.clone();
            App::new()
                .wrap(middleware::Logger::default())
                .service(web::resource("/ch-ch-ch-changes").route(web::get().to(
                    move |req, stream: web::Payload| {
                        let session_tx = session_tx.clone();
                        ws::start(
                            ChangeWatchSession {
                                last_heartbeat: Instant::now(),
                                session_tx,
                            },
                            &req,
                            stream,
                        )
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

    fn spawn_opener(&self) {
        let root_url = format!("http://[{}]:{}/index.html", &self.addr, &self.port);
        std::thread::spawn(move || {
            opener::open(&root_url).unwrap();
        });
    }
}

fn pump_channels(
    root: PathBuf,
    (uri_rx, session_rx): (Receiver<Uri>, Receiver<Addr<ChangeWatchSession>>),
) {
    let (event_tx, event_rx) = chan();
    let mut sessions = Vec::new();

    let mut watcher = notify::watcher(event_tx, std::time::Duration::from_millis(500)).unwrap();

    loop {
        select! {
            recv(uri_rx) -> new_uri => {
                let mut new_path = root.clone();
                let new_uri = new_uri.expect("path events should be live");

                for part in new_uri.path().split("/") {
                    new_path.push(part);
                }

                let displayed = new_path.display().to_string();

                if let Err(why) = watcher.watch(&displayed, notify::RecursiveMode::NonRecursive) {
                    warn!("couldn't add a watch for {} because {:?}", &displayed, why);
                } else {
                    debug!("added/refreshed watch for {}", &displayed);
                }
            },
            recv(session_rx) -> new_session => {
                let new_session = new_session.expect("session events should be live");
                sessions.push(new_session);
                debug!("new change watch session");
            },
            recv(event_rx) -> event => {
                if let Ok(event) = event.expect("filesystem events should be live") {
                    'this_event: for path in event.paths {
                        let changed = path.display().to_string();

                        info!("file change detected at {}", &changed);

                        for session in &sessions {
                            session.do_send(Changed(changed.clone()));
                        }
                        break 'this_event;
                    }
                }
            },
        }
    }
}

struct Changed(String);

impl Message for Changed {
    type Result = ();
}

struct FilesWatcher {
    uri_tx: Sender<Uri>,
}

impl FilesWatcher {
    fn new(root_path: &Path, session_rx: Receiver<Addr<ChangeWatchSession>>) -> Self {
        let (uri_tx, uri_rx) = chan();
        let root = root_path.to_owned();
        std::thread::spawn(|| pump_channels(root, (uri_rx, session_rx)));

        Self { uri_tx }
    }
}

impl<S, B> Transform<S> for FilesWatcher
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Transform = WatchHandle<S>;
    type InitError = ();
    type Future = Compat<Ready<Result<Self::Transform, Self::InitError>>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ok(WatchHandle {
            service,
            uri_tx: self.uri_tx.clone(),
        })
        .compat()
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
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Box<
        dyn OldFuture<
                Item = <<S as Service>::Future as OldFuture>::Item,
                Error = <<S as Service>::Future as OldFuture>::Error,
            > + 'static,
    >;

    fn poll_ready(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        // TODO inspect the request to see whether we want to add it to watch paths
        let request_uri = req.uri().clone();
        let uri_tx = self.uri_tx.clone();
        let result = self.service.call(req);
        let mapped = result.map(move |response| {
            uri_tx
                .send(request_uri)
                .unwrap_or_else(|_| warn!("wasn't able to send a new uri to watch"));
            response
        });
        Box::new(mapped)
    }
}

struct ChangeWatchSession {
    last_heartbeat: Instant,
    session_tx: Sender<Addr<ChangeWatchSession>>,
}

impl ChangeWatchSession {
    fn tick_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }
}

impl Handler<Changed> for ChangeWatchSession {
    type Result = ();
    fn handle(
        &mut self,
        Changed(changed): Changed,
        cx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        info!("notifying client of changed file");
        cx.text(changed);
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
        self.session_tx.send(cx.address()).unwrap();
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
