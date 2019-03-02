#![feature(await_macro, futures_api, async_await, integer_atomics)]

use {
    moxie::*,
    moxie_wrench::{
        color::Color, position::Position, size::Size, surface::CursorMoved, Components,
    },
};

// #[moxie::component]
fn simple_root(compose: &impl Components, scope: Scope) {
    let initial_size = Size::new(1920.0, 1080.0);

    let color = state! { scope <- Color::new(0.0, 0.0, 0.3, 1.0) };
    let color_hdl: Handle<Color> = color.handle();

    let (send_mouse_events, mut mouse_positions): (Sender<CursorMoved>, _) = channel!(scope);

    task! {
        scope <- async move {
            while let Some(cursor_moved) = await!(mouse_positions.next()) {
                color_hdl.set(|_prev_color| {
                    fun_color_from_mouse_position(initial_size, cursor_moved.position)
                });
            }
        }
    };

    mox! { scope <- compose.surface(initial_size, send_mouse_events, *color) }
}

fn fun_color_from_mouse_position(window_size: Size, pos: Position) -> Color {
    let x_factor = (pos.x / window_size.width).raw() as f32;
    let y_factor = (pos.y / window_size.height).raw() as f32;

    Color::new(x_factor, x_factor, y_factor, y_factor)
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
