use {
    crate::{prelude::*, winit_future::WindowEvents},
    gleam::gl,
    glutin::GlContext,
    webrender::api::*,
    webrender::ShaderPrecacheFlags,
};

const PRECACHE_SHADER_FLAGS: ShaderPrecacheFlags = ShaderPrecacheFlags::EMPTY;
const TITLE: &'static str = "Moxie Wrench Sample App";
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

// FIXME: fns that take children work with salsa
pub fn surface(compose: &impl ComposeDb, key: ScopeId) {
    // get the state port for the whole scope
    let port = compose.state(key);
    let events = port.get(callsite!(key), || WindowEvents::new());

    let window: Guard<glutin::GlWindow> = port.get(callsite!(key), || {
        let context_builder =
            glutin::ContextBuilder::new().with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0),
            });
        let window_builder = winit::WindowBuilder::new()
            .with_title(TITLE)
            .with_multitouch()
            .with_dimensions(winit::dpi::LogicalSize::new(WIDTH as f64, HEIGHT as f64));

        let window =
            glutin::GlWindow::new(window_builder, context_builder, events.raw_loop()).unwrap();

        unsafe {
            window.make_current().ok();
        }

        window
    });

    let gl = port.get(callsite!(key), || match window.get_api() {
        glutin::Api::OpenGl => unsafe {
            gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
        },
        glutin::Api::OpenGlEs => unsafe {
            gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
        },
        glutin::Api::WebGl => unimplemented!(),
    });

    // TODO split returned state tuples?
    let renderer = port.get(callsite!(key), || {
        info!("OpenGL version {}", gl.get_string(gl::VERSION));
        let device_pixel_ratio = window.get_hidpi_factor() as f32;
        info!("Device pixel ratio: {}", device_pixel_ratio);

        info!("Loading shaders...");
        let opts = webrender::RendererOptions {
            resource_override_path: None,
            precache_flags: PRECACHE_SHADER_FLAGS,
            device_pixel_ratio,
            clear_color: Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
            //scatter_gpu_cache_updates: false,
            debug_flags: webrender::DebugFlags::ECHO_DRIVER_MESSAGES,
            ..webrender::RendererOptions::default()
        };

        let framebuffer_size = {
            let size = window
                .get_inner_size()
                .unwrap()
                .to_physical(device_pixel_ratio as f64);
            DeviceIntSize::new(size.width as i32, size.height as i32)
        };

        webrender::Renderer::new(gl.clone(), events.notifier(), opts, None).unwrap()
    });

    let api = port.get(callsite!(key), || renderer.1.create_api());

    // let document_id = api.add_document(framebuffer_size, 0);

    // let epoch = Epoch(0);
    // let pipeline_id = PipelineId(0, 0);
    // let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
    // let api = Some(api);

    // match global_event {
    //     winit::Event::WindowEvent {
    //         event: winit::WindowEvent::CloseRequested,
    //         ..
    //     } => return ControlFlow::Break,
    //     Event::WindowEvent {
    //         event:
    //             winit::WindowEvent::KeyboardInput {
    //                 input:
    //                     winit::KeyboardInput {
    //                         state: winit::ElementState::Pressed,
    //                         virtual_keycode: Some(key),
    //                         ..
    //                     },
    //                 ..
    //             },
    //         ..
    //     } => match key {
    //         winit::VirtualKeyCode::Escape => return ControlFlow::Break,
    //         k => info!("keycode received: {:?}", k),
    //     },
    //     Event::WindowEvent { .. } => {}
    //     _ => return ControlFlow::Continue,
    // };

    // // TODO only do this on state changes!
    // let builder = DisplayListBuilder::new(self.pipeline_id, self.layout_size);

    // let mut txn = Transaction::new();
    // txn.set_root_pipeline(self.pipeline_id);
    // txn.generate_frame();

    // self.app.render();

    // txn.set_display_list(
    //     self.epoch,
    //     Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
    //     self.layout_size,
    //     builder.finalize(),
    //     true,
    // );
    // txn.generate_frame();
    // self.api
    //     .as_mut()
    //     .unwrap()
    //     .send_transaction(self.document_id, txn);

    // self.renderer.update();
    // self.renderer.render(self.framebuffer_size).unwrap();
    // let _ = self.renderer.flush_pipeline_info();
    // self.window.swap_buffers().ok();

    // ControlFlow::Continue
}
