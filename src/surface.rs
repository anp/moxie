use {
    crate::{events::WindowEvents, prelude::*},
    gleam::gl,
    glutin::{GlContext, GlWindow},
    webrender::api::*,
    webrender::ShaderPrecacheFlags,
};

// FIXME: fns that take children work with salsa
pub fn surface(compose: &impl Surface, key: ScopeId) {
    // get the state port for the whole scope
    let compose = compose.scope(key);

    let window_and_notifier = compose.state(callsite!(key), || {
        let mut events = WindowEvents::new();

        info!("initializing window");
        let window = GlWindow::new(
            winit::WindowBuilder::new()
                .with_title("if you're reading this, i win")
                .with_multitouch()
                .with_dimensions(winit::dpi::LogicalSize::new(1920.0, 1080.0)),
            glutin::ContextBuilder::new().with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0),
            }),
            events.raw_loop(),
        )
        .unwrap();

        info!("making window the current window");
        unsafe {
            window.make_current().ok();
        }

        let notifier = events.notifier();

        compose.task(
            callsite!(key),
            async move {
                while let Some(ev) = await!(events.next()) {
                    trace!("event received: {:?}", ev);
                }
            },
        );

        (window, notifier)
    });
    let (window, notifier) = &*window_and_notifier;

    let gl = compose.state(callsite!(key), || match window.get_api() {
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
            .to_physical(device_pixel_ratio as f64);
        DeviceIntSize::new(size.width as i32, size.height as i32)
    };

    // TODO split returned state tuples?
    let mut renderer = compose.state(callsite!(key), || {
        info!("OpenGL version {}", gl.get_string(gl::VERSION));
        info!("Device pixel ratio: {}", device_pixel_ratio);

        info!("Loading shaders...");
        let opts = webrender::RendererOptions {
            precache_flags: ShaderPrecacheFlags::EMPTY,
            device_pixel_ratio,
            clear_color: Some(ColorF::new(0.0, 0.4, 0.3, 1.0)),
            ..webrender::RendererOptions::default()
        };

        webrender::Renderer::new(gl.clone(), (*notifier).clone(), opts, None).unwrap()
    });

    let api = compose.state(callsite!(key), || renderer.1.create_api());

    let document_id = compose.state(callsite!(key), || api.add_document(framebuffer_size, 0));

    let epoch = Epoch(0);
    let pipeline_id = PipelineId(0, 0);
    let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);

    // FIXME only do the following on state changes!
    let builder = DisplayListBuilder::new(pipeline_id, layout_size);

    let mut txn = Transaction::new();
    txn.set_root_pipeline(pipeline_id);
    txn.generate_frame();

    // FIXME render child functions here

    txn.set_display_list(
        epoch,
        Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
        layout_size,
        builder.finalize(),
        true,
    );
    txn.generate_frame();
    api.send_transaction(*document_id, txn);

    renderer.0.update();
    renderer.0.render(framebuffer_size).unwrap();
    let _ = renderer.0.flush_pipeline_info();
    window.swap_buffers().unwrap();
}
