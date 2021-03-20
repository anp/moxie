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
        Stateful,
    };
}

use std::fmt::Debug;

/// A stateful element within the application.
// TODO make a way for the invoker to pass extra args instead of default?
pub trait Stateful: Debug + Sized + 'static {
    /// The value returned from `update` on each revision.
    type Output: interfaces::node::Child;

    /// The type used to generate updates to the app, typically a wrapper around
    /// [`moxie::Key`]s.
    type Updater: From<moxie::Key<Self>>;

    /// Compute a new version of the output.
    fn tick(&self, updater: Self::Updater) -> Self::Output;
}

/// A "root" stateful element which can be booted directly without any
/// arguments.
pub trait Boot: Stateful + Default {
    /// Start the app running with the provided `root`.
    fn boot(root: impl Into<prelude::RawNode>) {
        boot(root, || {
            let (app, updater) = prelude::state(Self::default);
            app.tick(updater.into())
        });
    }
}

/// Produce an interactive entrypoint for the specified app type. Creates a
/// `#[wasm_bindgen]` export with the name of the app type prefixed with `boot`.
/// For example, `app_boot!(Example)` would export a JavaScript function named
/// `bootExample`.
#[macro_export]
macro_rules! app_boot {
    ($app:ty) => {
        moxie_dom::__paste! {
            impl moxie_dom::Boot for $app {}

            #[moxie_dom::__wasm_bindgen(js_name = [<boot $app>])]
            #[doc(hidden)]
            pub fn [<__js_boot_ $app:snake>] (root: moxie_dom::raw::sys::Node) {
                <$app as moxie_dom::Boot>::boot(root);
            }
        }
    };
}

#[cfg(feature = "webdom")]
#[doc(hidden)]
pub use wasm_bindgen::prelude::wasm_bindgen as __wasm_bindgen;

#[doc(hidden)]
pub use paste::paste as __paste;

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
