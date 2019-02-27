#![feature(await_macro, futures_api, async_await, integer_atomics)]

fn simple_root(db: &impl moxie_wrench::Components, scope: moxie::ScopeId) {
    let compose = db.scope(scope);
    db.surface(moxie::scope!(compose.id), 1920, 1080);
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .default_format_timestamp(true)
        .default_format_level(true)
        .default_format_module_path(true)
        .filter(Some("webrender"), log::LevelFilter::Warn)
        .filter(Some("salsa"), log::LevelFilter::Warn)
        .init();
    log::debug!("logger initialized");

    let runtime = moxie_wrench::Toolbox::default();
    let mut executor = futures::executor::ThreadPool::new().unwrap();
    let spawner = executor.clone();
    let fut = moxie::run(runtime, spawner, simple_root);
    executor.run(fut);
}
