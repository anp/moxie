#![feature(await_macro, futures_api, async_await, integer_atomics)]

use {
    moxie::*,
    moxie_wrench::{color::Color, surface::CursorMoved},
    webrender::api::ColorF,
    winit::dpi::{LogicalPosition, LogicalSize},
};

fn simple_root(db: &impl moxie_wrench::Components, scope: moxie::ScopeId) {
    let compose = db.scope(scope);

    let initial_size = LogicalSize::new(1920.0, 1080.0);

    let color = compose.state(callsite!(scope), || Color(ColorF::new(0.3, 0.0, 0.0, 1.0)));
    let color_hdl: Handle<Color> = color.handle();

    let (send_mouse_events, mut mouse_positions) = moxie::channel::<CursorMoved>(callsite!(scope));

    compose.task(
        moxie::callsite!(scope),
        async move {
            let color = color_hdl;
            while let Some(cursor_moved) = await!(mouse_positions.next()) {
                color.set(|_prev_color| {
                    fun_color_from_mouse_position(initial_size, cursor_moved.position)
                });
            }
        },
    );

    db.surface(
        moxie::scope!(compose.id),
        1920,
        1080,
        send_mouse_events,
        *color,
    );
}

fn fun_color_from_mouse_position(window_size: LogicalSize, pos: LogicalPosition) -> Color {
    let x_factor = (pos.x / window_size.width) as f32;
    let y_factor = (pos.y / window_size.height) as f32;

    Color(webrender::api::ColorF {
        r: x_factor,
        g: x_factor,
        b: y_factor,
        a: y_factor,
    })
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
