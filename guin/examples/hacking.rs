use {
    tracing::*,
    tracing_fmt::{filter::env::EnvFilter, FmtSubscriber},
};

fn main() {
    const RUST_LOG: &str = "debug";

    tracing::subscriber::with_default(
        FmtSubscriber::builder()
            .with_filter(EnvFilter::new(RUST_LOG))
            .finish(),
        || {
            debug!("logging init'd");
            guin::run_app(HackingApp);
        },
    )
}

#[derive(Clone, Debug)]
struct HackingApp;

impl guin::App for HackingApp {
    const TITLE: &'static str = "guin-hacking";
}

impl moxie::Component for HackingApp {
    #[tracing::instrument]
    fn contents(self) {}
}
