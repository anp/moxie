use {
    futures::task::{waker, ArcWake},
    glutin::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopProxy},
        window::WindowBuilder,
        ContextBuilder,
    },
    moxie::*,
    parking_lot::Mutex,
    std::sync::Arc,
    tracing::*,
};

pub fn show_in_window(root: impl Component + Clone + 'static, title: &str) {
    let el = EventLoop::new();
    let el_waker = waker(Arc::new(GuinWaker(Mutex::new(el.create_proxy()))));

    let wb = WindowBuilder::new().with_title(title);
    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    windowed_context.swap_buffers().unwrap();

    let mut runtime = moxie::Runtime::new(move || show!(root.clone()));
    runtime.set_state_change_waker(el_waker);

    info!("entering glutin event loop");
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut tick_runtime = || {
            let window_size = WindowSize(windowed_context.window().inner_size());
            debug!({ ?window_size }, "ticking runtime");
            topo::call! {
                {
                    let window_size: &WindowSize = &*topo::Env::expect();
                    debug!({ ?window_size }, "before calling root");
                    runtime.run_once();
                },
                env! {
                    WindowSize => window_size,
                }
            };
            windowed_context.swap_buffers().unwrap();
        };

        let outcome = match event {
            Event::LoopDestroyed => Outcome::None,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(logical_size) => Outcome::Resize(*logical_size),
                WindowEvent::RedrawRequested => Outcome::TickRuntime,
                WindowEvent::CloseRequested => Outcome::Exit,
                _ => Outcome::None,
            },
            Event::UserEvent(()) => Outcome::TickRuntime,
            _ => Outcome::None,
        };

        match outcome {
            Outcome::Resize(new_size) => {
                let dpi_factor = windowed_context.window().hidpi_factor();
                windowed_context.resize(new_size.to_physical(dpi_factor));
                tick_runtime();
            }
            Outcome::TickRuntime => tick_runtime(),
            Outcome::Exit => *control_flow = ControlFlow::Exit,
            Outcome::None => (),
        }
    });
}

#[derive(Debug)]
pub struct WindowSize(pub glutin::dpi::LogicalSize);

#[derive(Debug, PartialEq)]
enum Outcome {
    Resize(glutin::dpi::LogicalSize),
    TickRuntime,
    Exit,
    None,
}

struct GuinWaker(Mutex<EventLoopProxy<()>>);

impl ArcWake for GuinWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let _dont_care = arc_self.0.lock().send_event(());
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
