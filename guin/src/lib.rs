use {
    moxie::*,
    pathfinder_canvas::{CanvasFontContext, CanvasRenderingContext2D},
    pathfinder_content::color::ColorF,
    pathfinder_geometry::vector::Vector2I,
    pathfinder_gl::{GLDevice, GLVersion},
    pathfinder_gpu::resources::FilesystemResourceLoader,
    pathfinder_renderer::{
        concurrent::{rayon::RayonExecutor, scene_proxy::SceneProxy},
        gpu::{
            options::{DestFramebuffer, RendererOptions},
            renderer::Renderer,
        },
        options::BuildOptions,
    },
    tracing::debug,
};

pub trait App: Component + Clone + 'static {
    const TITLE: &'static str;
}

#[derive(Clone, Debug)]
struct PathfinderRenderer<Root: Component> {
    root: Root,
}

impl<Root> Component for PathfinderRenderer<Root>
where
    Root: Component,
{
    fn contents(self) {
        debug!("getting window size from environment");
        let window_size: &moxie_glutin::WindowSize = &*topo::Env::expect();
        let window_size = Vector2I::new(window_size.0.width as i32, window_size.0.height as i32);
        let mut renderer = Renderer::new(
            GLDevice::new(GLVersion::GL3, 0),
            &FilesystemResourceLoader::locate(),
            DestFramebuffer::full_window(window_size),
            RendererOptions {
                background_color: Some(ColorF::white()),
            },
        );

        let mut canvas = CanvasRenderingContext2D::new(
            CanvasFontContext::from_system_source(),
            window_size.to_f32(),
        );

        show!(self.root);

        let scene = SceneProxy::from_scene(canvas.into_scene(), RayonExecutor);
        scene.build_and_render(&mut renderer, BuildOptions::default());
    }
}

pub fn run_app<Root: App>(root: Root) {
    moxie_glutin::show_in_window(PathfinderRenderer { root }, Root::TITLE);
}
