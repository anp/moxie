//! Tools for declaratively constructing and incrementally updating HTML DOM
//! trees on the web. Based on the [`moxie`] UI runtime.

#![deny(clippy::all, missing_docs)]

/// Internal macros for stamping out types to match stringly-typed web APIs.
#[macro_use]
mod macros;

pub(crate) mod cached_node;
pub mod elements;
pub mod embed;
pub mod interfaces;
pub mod text;

/// A module for glob-importing the most commonly used moxie-dom items.
pub mod prelude {
    #[cfg(feature = "webdom")]
    pub use augdom::{document, sys};
    pub use augdom::{event, Dom as _};
    pub use moxie::{cache, cache_state, cache_with, once, once_with, state, Key};

    pub use crate::{
        elements::html,
        interfaces::{
            content_categories::{
                EmbeddedContent as _, FlowContent as _, FormAssociatedContent as _,
                HeadingContent as _, InteractiveContent as _, LabelableContent as _,
                ListedContent as _, MetadataContent as _, PhrasingContent as _,
                ResettableContent as _, SectioningContent as _, SubmittableContent as _,
            },
            element::Element as _,
            event_target::EventTarget as _,
            global_events::{GlobalEvent as _, GlobalEventHandler as _},
            html_element::HtmlElement as _,
            node::{Node as _, Parent as _},
        },
        text::text,
    };
}

/// Provides the underlying DOM implementation for moxie-dom.
pub use augdom as raw;

/// The "boot sequence" for a moxie-dom instance creates a
/// [crate::embed::WebRuntime] with the provided arguments, runs its root
/// function once and begins scheduling its subsequent execution with
/// `requestAnimationFrame` on state changes.
///
/// If you need to schedule your root function more or less frequently than when
/// state variables are updated, see the [embed](crate::embed) module for
/// granular control over scheduling.
///
/// In terms of the embed module's APIs, this function constructs a new
/// [`WebRuntime`](crate::embed::WebRuntime) and begins scheduling it with an
/// [`AnimationFrameScheduler`](raf::AnimationFrameScheduler) which requests an
/// animation frame only when there are updates to state variables.
///
/// Requires the `webdom` feature.
#[cfg(any(feature = "webdom", doc))]
pub fn boot<Root>(new_parent: impl Into<augdom::Node>, root: impl FnMut() -> Root + 'static)
where
    Root: interfaces::node::Child + 'static,
{
    let mut rt = embed::WebRuntime::new(new_parent.into(), root);
    rt.run_once();
    rt.animation_frame_scheduler().run_on_wake();
}

/// Runs the provided closure once and produces a prettified HTML string from
/// the contents.
///
/// If you need more control over the output of the HTML, see the implementation
/// of this function.
///
/// Requires the `rsdom` feature.
#[cfg(any(feature = "rsdom", doc))]
pub fn render_html<Root>(root: impl FnMut() -> Root + 'static) -> String
where
    Root: interfaces::node::Child + 'static,
{
    use augdom::Dom;

    let (mut tester, root) = embed::WebRuntime::in_rsdom_div(root);
    tester.run_once();
    let outer = augdom::Node::Virtual(root).pretty_outer_html(2);
    // because we use the indented version, we know that only at the top and bottom
    // is what we want
    outer.lines().filter(|l| *l != "<div>" && *l != "</div>").map(|l| l.split_at(2).1).fold(
        String::new(),
        |mut fragment, line| {
            if !fragment.is_empty() {
                fragment.push('\n');
            }
            fragment.push_str(line);
            fragment
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        boot,
        elements::html::{b, div, p},
        prelude::*,
        raw::testing::Query,
    };
    use pretty_assertions::assert_eq;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub async fn hello_browser() {
        let root = augdom::document().create_element("div").unwrap();
        boot(root.clone(), || {
            mox::mox! {
                <div>
                    <p>"hello browser"</p>
                    <div>
                        <p><b>"looooool"</b></p>
                    </div>
                </div>
            }
        });

        root.find().by_text("looooool").until().many().await;
        assert_eq!(
            root.pretty_outer_html(2),
            r#"<div>
  <div>
    <p>hello browser</p>
    <div>
      <p>
        <b>looooool</b>
      </p>
    </div>
  </div>
</div>"#
        );
    }
}
