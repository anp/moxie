use {
    crate::{color::Color, events::WindowEvents, position::Position, size::Size, Components},
    futures::future::AbortHandle,
    gleam::gl,
    glutin::{GlContext, GlWindow},
    log::*,
    moxie::{channel::Sender, *},
    parking_lot::Mutex,
    std::{sync::Arc, task::Waker},
    webrender::api::*,
    webrender::ShaderPrecacheFlags,
    winit::WindowId,
};

// FIXME: fns that take children work with salsa
pub fn surface(
    compose: &impl Components,
    scope: Scope,
    initial_size: Size,
    mouse_events: Sender<CursorMoved>,
    color: Color,
) {
    surface_impl(compose, scope, initial_size, color, mouse_events);
}

async fn handle_events(
    this_window: WindowId,
    mut events: WindowEvents,
    waker: Waker,
    top_level_exit: AbortHandle,
    mut send_mouse_positions: Sender<CursorMoved>,
) {
    'top: while let Some(event) = await!(events.next()) {
        let event = match event.inner {
            winit::Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == this_window => event,
            // we only care about events for this particular window
            _ => continue 'top,
        };
        trace!("handling event {:?}", event);

        use winit::WindowEvent::*;
        match event {
            CloseRequested | Destroyed => {
                info!("close requested or window destroyed. exiting.");
                top_level_exit.abort();
                futures::pending!(); // so nothing else in this task fires accidentally
            }
            Resized(new_size) => {
                debug!("resized: {:?}", new_size);
            }
            Moved(_new_position) => {}
            DroppedFile(_path) => {}
            HoveredFile(_path) => {}
            HoveredFileCancelled => {}
            ReceivedCharacter(_received_char) => {}
            Focused(_in_focus) => {}
            KeyboardInput {
                device_id: _device_id,
                input: _input,
            } => {}
            CursorMoved {
                device_id: _device_id,
                position,
                modifiers: _modifiers,
            } => {
                let _ = await!(send_mouse_positions.send(self::CursorMoved {
                    position: (*position).into(),
                }));
            }
            CursorEntered {
                device_id: _device_id,
            } => {}
            CursorLeft {
                device_id: _device_id,
            } => {}
            MouseWheel {
                device_id: _device_id,
                delta: _delta,
                phase: _phase,
                modifiers: _modifiers,
            } => {}

            MouseInput {
                device_id: _device_id,
                state: _state,
                button: _button,
                modifiers: _modifiers,
            } => {}

            TouchpadPressure {
                device_id: _device_id,
                pressure: _pressure,
                stage: _stage,
            } => {}

            AxisMotion {
                device_id: _device_id,
                axis: _axis,
                value: _value,
            } => {}

            Refresh => {
                // technically unnecessary? it would be nice to only wake
                // on state changes
                waker.wake();
            }

            Touch(_touch) => {}
            HiDpiFactorChanged(new_factor) => {
                info!("DPI factor changed, is now {}", new_factor);
            }
        }

        waker.wake();
    }
}

pub fn surface_impl(
    compose: &impl Components,
    scope: Scope,
    initial_size: Size,
    background_color: Color,
    send_mouse_positions: Sender<CursorMoved>,
) {
    let (window, notifier) = &*state!(
        scope <- {
            let events = WindowEvents::new();

            info!("initializing window");
            let window = GlWindow::new(
                winit::WindowBuilder::new()
                    .with_title("moxie is alive?")
                    .with_multitouch()
                    .with_dimensions(initial_size.into()),
                glutin::ContextBuilder::new().with_gl(glutin::GlRequest::GlThenGles {
                    opengl_version: (3, 2),
                    opengles_version: (3, 0),
                }),
                events.raw_loop(),
            )
            .unwrap();

            let window_id = window.id();
            info!("making window {:?} the current window", window_id);
            unsafe {
                window.make_current().ok();
            }

            // this notifier needs to be created before events is captured by the move block below
            let notifier = events.notifier();

            task_fut!(
                scope <- handle_events(
                    window_id,
                    events,
                    compose.waker(),
                    compose.top_level_exit(),
                    send_mouse_positions,
                )
            );

            (window, notifier)
        }
    );

    let gl = state!(
        scope <- match window.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => unimplemented!(),
        }
    );

    let device_pixel_ratio = window.get_hidpi_factor() as f32;
    let framebuffer_size = {
        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(f64::from(device_pixel_ratio));
        DeviceIntSize::new(size.width as i32, size.height as i32)
    };

    // TODO split returned state tuples?
    let (renderer, render_sender) = &mut *state!(
        scope <- {
            debug!("creating webrender renderer");
            info!("OpenGL version {}", gl.get_string(gl::VERSION));
            info!("Device pixel ratio: {}", device_pixel_ratio);

            let (renderer, sender) = webrender::Renderer::new(
                gl.clone(),
                (*notifier).clone(),
                webrender::RendererOptions {
                    precache_flags: ShaderPrecacheFlags::EMPTY,
                    device_pixel_ratio,
                    clear_color: Some(ColorF::new(0.0, 0.4, 0.3, 1.0)),
                    ..webrender::RendererOptions::default()
                },
                None,
            )
            .unwrap();

            // webrender is not happy if we fail to deinit the renderer by ownership before its Drop impl runs
            let renderer = crate::drop_guard::DropGuard::new(renderer, |r| r.deinit());

            (Arc::new(Mutex::new(renderer)), Arc::new(Mutex::new(sender)))
        }
    );

    let api = state!(scope <- render_sender.lock().create_api());
    let document_id = state!(scope <- api.add_document(framebuffer_size, 0));

    let epoch = Epoch(0);
    let pipeline_id = PipelineId(0, 0);
    let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);

    let builder = DisplayListBuilder::new(pipeline_id, layout_size);

    trace!("new render xact, generating frame");
    let mut txn = Transaction::new();
    txn.set_root_pipeline(pipeline_id);
    txn.generate_frame();

    // FIXME render child functions here

    trace!("setting display list, generating frame, and sending transaction");
    txn.set_display_list(
        epoch,
        Some(background_color.into()),
        layout_size,
        builder.finalize(),
        true,
    );
    txn.generate_frame();
    api.send_transaction(*document_id, txn);
    let mut renderer = renderer.lock();
    renderer.update();

    trace!("rendering");
    renderer.render(framebuffer_size).unwrap();
    let _ = renderer.flush_pipeline_info();

    trace!("swapping buffers");
    window.swap_buffers().unwrap();
}

#[derive(Debug, Clone)]
pub struct CursorMoved {
    pub position: Position,
}
