use crate::{error::Error, revision::EvaluatorExt, state::WorkspaceState};
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use juniper_actix::graphql_handler;
use std::{path::Path, thread::JoinHandle};
use tracing::{debug, error, info, instrument};

pub mod builtins;
pub mod error;
pub mod gql;
pub mod graph;
pub mod revision;
pub mod state;
pub mod vfs;

pub(crate) type Result<T> = color_eyre::eyre::Result<T, Error>;

pub struct Workspace {
    state: WorkspaceState,
}

impl Workspace {
    /// The asset path used to resolve the root of a honk workspace.
    const ASSET_PATH: &'static str = "WORKSPACE.honk";

    pub fn new(root: impl AsRef<Path>) -> Self {
        let state = WorkspaceState::new(root);
        Self { state }
    }

    pub fn maintain(self) -> crate::Result<()> {
        // TODO change current directory to workspace root?
        info!("maintaining workspace");
        spawn_server(self.state.clone());
        loop {
            if let Err(error) = self.converge() {
                error!(%error, "couldn't converge current workspace revision");
            }
            self.state.wait_for_changes();
        }
    }

    #[instrument(level = "info", skip(self), fields(root = %self.state.root().display()))]
    fn converge(&self) -> crate::Result<()> {
        debug!("constructing workspace env");
        self.state.start_new_revision();
        let _workspace_env = self.state.load(Self::ASSET_PATH)?;

        let _build = self.state.resolve()?;
        info!("discovered targets");

        tracing::warn!("uh run some builds i guess?");

        info!("finished");
        Ok(())
    }
}

pub fn spawn_server(state: crate::WorkspaceState) -> JoinHandle<std::io::Result<()>> {
    info!("spawning server");
    std::thread::spawn(move || {
        actix_web::rt::System::new("honk_workspace_server").block_on(async move {
            HttpServer::new(move || {
                App::new()
                    .wrap(middleware::Compress::default())
                    .wrap(middleware::Logger::default())
                    .data(state.clone())
                    .service(http_root)
                    .service(
                        web::resource("/graphgl")
                            .route(web::post().to(gql::graphql_route))
                            .route(web::get().to(gql::graphql_route)),
                    )
                    .service(
                        web::resource("/playground").route(web::get().to(gql::playground_route)),
                    )
                    .service(web::resource("/graphiql").route(web::get().to(gql::graphiql_route)))
            })
            // FIXME make this configurable
            .bind("[::1]:8080")?
            .run()
            .await
        })
    })
}

#[get("/")]
async fn http_root(data: web::Data<WorkspaceState>) -> impl Responder {
    use petgraph::dot::{Config, Dot};
    HttpResponse::Ok().body(format!(
        "Hello world! from {}\n\n{}",
        data.root().display(),
        Dot::with_config(&data.resolve().unwrap(), &[Config::EdgeNoLabel]),
    ))
}
