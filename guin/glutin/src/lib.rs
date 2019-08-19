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

    let mut runtime = moxie::Runtime::new(move || show!(root.clone()));
    runtime.set_state_change_waker(el_waker);

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut should_tick = false;
        let mut should_swap = false;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = windowed_context.window().hidpi_factor();
                    windowed_context.resize(logical_size.to_physical(dpi_factor));
                    should_tick = true;
                }
                WindowEvent::RedrawRequested => {
                    trace!("redraw requested");
                    should_swap = true;
                }
                WindowEvent::CloseRequested => {
                    info!("exiting");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => (),
            },
            Event::UserEvent(()) => {
                debug!("woken by moxie runtime");
                should_tick = true;
            }
            _ => (),
        }

        if should_tick {
            runtime.run_once();
            should_swap = true;
        }

        if should_swap {
            windowed_context.swap_buffers().unwrap();
        }
    });
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
