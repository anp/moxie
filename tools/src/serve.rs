use {
    failure::{Error, ResultExt},
    futures::compat::{Compat, Future01CompatExt},
    gumdrop::Options,
    hyper::{
        service::{make_service_fn, service_fn},
        Body, Response,
    },
    std::{net::IpAddr, path::PathBuf, sync::Arc},
    tracing::*,
};
// let mut opts = BuildOptions::default();
// wasm_pack::command::build::*,
// let build = Build::try_from_opts(opts)?;

#[derive(Debug, Options)]
pub struct ServeOpts {
    // interestingly, gumdrop requires this to print help text
    #[options(help = "print help message")]
    help: bool,
    #[options(free, help = "path to the project to watch")]
    path: PathBuf,
    #[options(help = "address to bind server to", default = "::1")]
    addr: IpAddr,
    #[options(help = "port to bind server to", default = "8000")]
    port: u16,
}

impl ServeOpts {
    pub async fn run(self) -> Result<(), Error> {
        let watch_path = self.path.canonicalize()?;
        let project = Arc::new(Project::new(watch_path)?);
        info!("starting {:?}", &project);

        let server = hyper::server::Server::bind(&(self.addr, self.port).into());

        let maker = make_service_fn(|_| {
            let project = project.clone();
            Compat::new(Box::pin(async {
                Ok::<_, hyper::Error>(service_fn(move |request| {
                    Compat::new(Box::pin(Project::handle(project.clone(), request)))
                }))
            }))
        });
        let with_service = server.serve(maker);
        let compatible = with_service.compat();

        compatible.await.context("serving project")?;
        Ok(())
    }
}

#[derive(Debug)]
struct Project {
    path: PathBuf,
}

impl Project {
    fn new(path: PathBuf) -> Result<Self, Error> {
        Ok(Self { path })
    }

    async fn handle(
        this: Arc<Self>,
        request: hyper::Request<hyper::Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        unimplemented!()
    }
}
