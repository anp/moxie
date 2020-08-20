use actix::prelude::*;
use actix_web_actors::ws;
use crossbeam::channel::Sender;
use std::time::{Duration, Instant};
use tracing::*;

const TIMEOUT: Duration = Duration::from_secs(10);
const FREQUENCY: Duration = Duration::from_secs(1);

pub struct Changed(pub String);

impl Message for Changed {
    type Result = ();
}

pub struct ChangeWatchSession {
    last_heartbeat: Instant,
    last_change: Option<Changed>,
    session_tx: Sender<Addr<ChangeWatchSession>>,
}

impl ChangeWatchSession {
    pub fn new(session_tx: Sender<Addr<ChangeWatchSession>>) -> Self {
        ChangeWatchSession { last_heartbeat: Instant::now(), last_change: None, session_tx }
    }

    fn tick_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }
}

impl Handler<Changed> for ChangeWatchSession {
    type Result = ();

    fn handle(&mut self, changed: Changed, _cx: &mut <Self as Actor>::Context) -> Self::Result {
        self.last_change = Some(changed);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChangeWatchSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, cx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.tick_heartbeat();
                cx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.tick_heartbeat();
            }
            Ok(ws::Message::Close(_)) => {
                cx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(text)) => {
                self.tick_heartbeat();
                debug!("ignoring text ws message {:?}", text);
            }
            Ok(ws::Message::Binary(_bin)) => {
                self.tick_heartbeat();
                debug!("ignoring binary ws message");
            }
            Ok(ws::Message::Continuation(_)) => self.tick_heartbeat(),
            Err(e) => warn!({ %e }, "websocket protocol error"),
        }
    }
}

impl Actor for ChangeWatchSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, cx: &mut Self::Context) {
        self.session_tx.send(cx.address()).unwrap();
        cx.run_interval(FREQUENCY, |session, cx| {
            if Instant::now().duration_since(session.last_heartbeat) > TIMEOUT {
                info!("ws change event client timed out, disconnecting");
                cx.stop();
                return;
            }

            if let Some(Changed(change)) = session.last_change.take() {
                info!(%change, "notifying client");
                cx.text(change);
            } else {
                cx.ping(b"");
            }
        });
    }
}
