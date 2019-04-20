#[derive(Debug)]
pub(crate) struct Weaver {
    root_dom_node: web_sys::Node,
}

impl moxie::Witness for Weaver {}
