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
    pub use crate::raw::sys;
    pub use crate::raw::{document, event, Dom as RawDom, Node as RawNode};
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
            element::ElementBuilder,
            event_target::EventTarget as _,
            global_events::{GlobalEvent as _, GlobalEventHandler as _},
            html_element::HtmlElementBuilder,
            node::{Child as _, NodeBuilder as _, NodeWrapper, Parent as _},
        },
        text::text,
    };
}

/// Provides the underlying DOM implementation for moxie-dom.
pub use augdom as raw;

/// The "boot sequence" for a moxie-dom instance creates a
/// [`crate::embed::DomLoop`] with the provided arguments and begins scheduling
/// its execution with `requestAnimationFrame` on state changes.
///
/// If you need to schedule your root function more or less frequently than when
/// state variables are updated, see the [embed](crate::embed) module for
/// granular control over scheduling.
///
/// In terms of the embed module's APIs, this function constructs a new
/// [`crate::embed::DomLoop`] and begins scheduling it with an
/// [`AnimationFrameScheduler`](raf::AnimationFrameScheduler) which requests an
/// animation frame only when there are updates to state variables.
///
/// Requires the `webdom` feature.
#[cfg(any(feature = "webdom", doc))]
pub fn boot<Root>(new_parent: impl Into<augdom::Node>, root: impl FnMut() -> Root + 'static)
where
    Root: interfaces::node::Child + 'static,
{
    embed::DomLoop::new(new_parent.into(), root).animation_frame_scheduler().run_on_wake();
}

#[cfg(test)]
mod tests {
    use crate::{
        boot,
        elements::html::{b, div, p},
        prelude::*,
    };
    use pretty_assertions::assert_eq;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub async fn hello_browser() {
        let root = augdom::document().create_element("div");
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
