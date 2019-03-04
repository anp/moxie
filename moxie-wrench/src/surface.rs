use {
    crate::{color::Color, events::WindowEvents, size::Size},
    gleam::gl,
    glutin::{GlContext, GlWindow},
    log::*,
    moxie::{Sender, *},
    parking_lot::Mutex,
    std::sync::Arc,
    webrender::api::*,
    webrender::ShaderPrecacheFlags,
};

mod events;

pub use events::CursorMoved;

// FIXME: fns that take children work with salsa
#[moxie::component]
pub fn Surface(
    background_color: Color,
    initial_size: Size,
    send_mouse_positions: Sender<CursorMoved>,
) {
    let (window, notifier) = &*state!({
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

        task_fut! {
            scope <- events::dispatch(
                window_id,
                events,
                compose.waker(),
                compose.top_level_exit(),
                send_mouse_positions,
            )
        };

        (window, notifier)
    });

    let gl = state!(match window.get_api() {
        glutin::Api::OpenGl => unsafe {
            gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
        },
        glutin::Api::OpenGlEs => unsafe {
            gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
        },
        glutin::Api::WebGl => unimplemented!(),
    });

    let device_pixel_ratio = window.get_hidpi_factor() as f32;
    let framebuffer_size = {
        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(f64::from(device_pixel_ratio));
        DeviceIntSize::new(size.width as i32, size.height as i32)
    };

    // TODO split returned state tuples?
    let (renderer, render_sender) = &mut *state!({
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

        // webrender is not happy if we fail to deinit the renderer by ownership
        // before its Drop impl runs
        let renderer = crate::drop_guard::DropGuard::new(renderer, |r| r.deinit());

        (Arc::new(Mutex::new(renderer)), Arc::new(Mutex::new(sender)))
    });

    let api = state!(render_sender.lock().create_api());
    let document_id = state!(api.add_document(framebuffer_size, 0));

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
