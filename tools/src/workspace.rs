use {
    failure::{Error, ResultExt},
    hyper::{Body, Response},
    std::{
        fmt::{Display, Formatter, Result as FmtResult},
        path::PathBuf,
        sync::Arc,
    },
    tracing::*,
};

#[derive(Debug)]
pub struct Workspace {
    user_path: PathBuf,
    canonical: PathBuf,
}

impl Workspace {
    pub fn new(user_path: PathBuf) -> Result<Arc<Self>, Error> {
        let canonical = user_path
            .canonicalize()
            .context("checking workspace path exists and has canonical repr")?;
        Ok(Arc::new(Self {
            user_path,
            canonical,
        }))
    }

    pub async fn handle(
        _this: Arc<Self>,
        request: hyper::Request<hyper::Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        info!("received request: {:?}", &request);
        unimplemented!()
    }
}

impl Display for Workspace {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("workspace at {}", self.user_path.display()))
    }
}

// let mut opts = BuildOptions::default();
// wasm_pack::command::build::*,
// let build = Build::try_from_opts(opts)?;
