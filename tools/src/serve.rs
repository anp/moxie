use {
    crate::workspace::Workspace,
    failure::{Error, ResultExt},
    futures::compat::{Compat, Future01CompatExt},
    gumdrop::Options,
    hyper::service::{make_service_fn, service_fn},
    std::{net::IpAddr, sync::Arc},
    tracing::*,
};

#[derive(Debug, Options)]
pub struct ServeOpts {
    // interestingly, gumdrop requires this to print help text
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "address to bind server to", default = "::1")]
    addr: IpAddr,
    #[options(help = "port to bind server to", default = "8000")]
    port: u16,
}

impl ServeOpts {
    pub async fn run(self, workspace: Arc<Workspace>) -> Result<(), Error> {
        info!("serving {}", &workspace);

        let server = hyper::server::Server::bind(&(self.addr, self.port).into());

        let maker = make_service_fn(|_| {
            let workspace = workspace.clone();
            Compat::new(Box::pin(async {
                Ok::<_, hyper::Error>(service_fn(move |request| {
                    Compat::new(Box::pin(Workspace::handle(workspace.clone(), request)))
                }))
            }))
        });
        let with_service = server.serve(maker);
        let compatible = with_service.compat();

        compatible.await.context("serving project")?;
        Ok(())
    }
}
