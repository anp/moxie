#![feature(await_macro, futures_api, async_await, integer_atomics)]

use {
    moxie::*,
    moxie_wrench::{color::Color, position::Position, rect::Rect, size::Size, surface::Surface},
    noisy_float::prelude::*,
    tokio_trace::*,
};

#[props]
struct SimpleApp;

impl Component for SimpleApp {
    fn compose(scp: Scope, SimpleApp: Self) {
        let initial_size = Size::new(1920.0, 1080.0);

        let mouse_position = state!(
            scp <- Position {
                x: r32(0.0),
                y: r32(0.0)
            }
        );

        let background_color = fun_color_from_mouse_position(initial_size, *mouse_position);

        let mouse_pos_hdl = mouse_position.handle();

        mox! { scp <- Surface {
            initial_size,
            mouse_position: mouse_pos_hdl,
            background_color,
            child: Rect {
                color: Color::new(0.0, 0.0, 0.3, 1.0),
                x: mouse_position.x,
                y: mouse_position.y,
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
