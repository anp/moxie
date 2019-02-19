#![feature(await_macro, futures_api, async_await, integer_atomics)]

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .default_format_timestamp(true)
        .default_format_level(true)
        .default_format_module_path(true)
        .filter(Some("webrender"), log::LevelFilter::Warn)
        .filter(Some("salsa"), log::LevelFilter::Warn)
        .init();
    log::debug!("logger initialized");

    moxie::Composer::run();
}
