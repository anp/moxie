#![feature(await_macro, futures_api, async_await, integer_atomics)]

use {
    futures::stream::StreamExt,
    moxie::*,
    moxie_wrench::{
        color::Color,
        position::Position,
        rect::Rect,
        size::Size,
        surface::{CursorMoved, Surface},
    },
    noisy_float::prelude::*,
    tokio_trace::*,
};

#[props]
struct SimpleApp;

impl Component for SimpleApp {
    fn compose(scp: Scope, SimpleApp: Self) {
        let initial_size = Size::new(1920.0, 1080.0);

        let color = state! { scp <- Color::new(0.0, 0.0, 0.3, 1.0) };
        let color_hdl: Handle<Color> = color.handle();

        let (send_mouse_positions, mut mouse_positions): (Sender<CursorMoved>, _) = channel!(scp);

        task! { scp <-
            while let Some(cursor_moved) = await!(mouse_positions.next()) {
                color_hdl.set(|_prev_color| {
                    fun_color_from_mouse_position(initial_size, cursor_moved.position)
                });
            }
        };

        mox! { scp <- Surface {
            initial_size,
            send_mouse_positions,
            background_color: *color,
            child: Rect {
                // TODO vary these based on inputs
                color: Color::new(0.0, 0.0, 0.3, 1.0),
                x: r32(600.0),
                y: r32(450.0),
                width: r32(200.0),
                height: r32(100.0),
            },
        }};
    }
}

fn fun_color_from_mouse_position(window_size: Size, pos: Position) -> Color {
    let x_factor = (pos.x / window_size.width).raw() as f32;
    let y_factor = (pos.y / window_size.height).raw() as f32;

    Color::new(x_factor, x_factor, y_factor, y_factor)
}

fn main() {
    std::env::set_var(
        "RUST_LOG",
        "debug,webrender=info,moxie::compose=trace,moxie_wrench::surface=trace",
    );
    let subscriber = tokio_trace_fmt::FmtSubscriber::builder().full().finish();

    tokio_trace::subscriber::with_default(subscriber, || {
        info!("spawning executor");
        let mut executor = futures::executor::ThreadPool::new().unwrap();
        executor.run(Runtime::go(executor.clone(), SimpleApp));
    });
}
