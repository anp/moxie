pub trait App: moxie::Component + Clone + 'static {
    const TITLE: &'static str;
}

#[derive(Clone, Debug)]
struct PathfinderRenderer;

impl moxie::Component for PathfinderRenderer {
    fn contents(self) {
        unimplemented!()
    }
}

pub fn run_app<Root: App>(root: Root) {
    moxie_glutin::show_in_window(root, Root::TITLE);
}
