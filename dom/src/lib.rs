//! Tools for declaratively constructing and incrementally updating HTML DOM trees on the web. Based
//! on the [`moxie`] UI runtime.

#![deny(clippy::all, missing_docs)]

use {
    augdom::{
        event::{Event, EventHandle},
        Dom, Node,
    },
    moxie,
    std::{
        cell::Cell,
        fmt::{Debug, Formatter, Result as FmtResult},
    },
};

pub mod elements;
pub mod embed;

/// A module for glob-importing the most commonly used moxie-dom items.
pub mod prelude {
    #[cfg(feature = "webdom")]
    pub use augdom::{document, sys};
    pub use augdom::{event, Dom};
    pub use moxie::mox;
    pub use moxie::prelude::*;

    pub use crate::text;
}

use prelude::*;

/// The "boot sequence" for a moxie-dom instance creates a [crate::embed::WebRuntime] with the
/// provided arguments and begins scheduling its execution with `requestAnimationFrame` on state
/// changes.
///
/// If you need to schedule your root function more or less frequently than when state variables are
/// updated, see the [embed](crate::embed) module for granular control over scheduling.
///
/// In terms of the embed module's APIs, this function constructs a new
/// [`WebRuntime`](crate::embed::WebRuntime) and begins scheduling it with an
/// [`AnimationFrameScheduler`](crate::embed::AnimationFrameScheduler) which requests an animation
/// frame only when there are updates to state variables.
#[cfg(feature = "webdom")]
pub fn boot(new_parent: impl Into<Node>, root: impl FnMut() + 'static) {
    embed::WebRuntime::new(new_parent.into(), root)
        .animation_frame_scheduler()
        .run_on_wake();
}

/// Runs the provided closure once and produces a prettified HTML string from the contents.
///
/// If you need more control over the output of the HTML, see the implementation of this function.
#[cfg(feature = "rsdom")]
pub fn render_html(root: impl FnMut() + 'static) -> String {
    let (mut tester, root) = embed::WebRuntime::in_rsdom_div(root);
    tester.run_once();
    let outer = augdom::Node::Virtual(root).pretty_outer_html(2);
    // because we use the indented version, we know that only at the top and bottom is what we want
    outer
        .lines()
        .filter(|l| *l != "<div>" && *l != "</div>")
        .map(|l| l.split_at(2).1)
        .fold(String::new(), |mut fragment, line| {
            if !fragment.is_empty() {
                fragment.push('\n');
            }
            fragment.push_str(line);
            fragment
        })
}

/// Create and mount a [DOM text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
/// This is normally called by the [`moxie::mox!`] macro.
#[topo::nested]
#[illicit::from_env(parent: &MemoElement)]
pub fn text(s: impl ToString) {
    // TODO consider a ToOwned-based memoization API that's lower level?
    // memo_ref<Ref, Arg, Output>(reference: Ref, init: impl FnOnce(Arg) -> Output)
    // where Ref: ToOwned<Owned=Arg> + PartialEq, etcetcetc
    let text_node = memo(s.to_string(), |s| parent.node.create_text_node(s));
    parent.ensure_child_attached(&text_node);
}

/// Create and mount an [HTML element](https://developer.mozilla.org/en-US/docs/Web/API/Element).
/// Called from the individual element macros, which in turn are normally called by the output of
/// the [`moxie::mox!`] macro.
///
/// The created DOM node is memoized at the bound callsite, allowing for subsequent re-executions to
/// be very cheap.
///
/// Mutation of the created element is performed during the `with_elem` closure via the provided
/// [`moxie_dom::MemoElement`] wrapper.
#[topo::nested]
#[illicit::from_env(parent: &MemoElement)]
pub fn element<ChildRet>(
    ty: &'static str,
    with_elem: impl FnOnce(&MemoElement) -> ChildRet,
) -> ChildRet {
    let elem = memo(ty, |ty| parent.node.create_element(ty));
    parent.ensure_child_attached(&elem);
    let elem = MemoElement::new(elem);
    with_elem(&elem)
}

/// A topologically-nested "incremental smart pointer" for an HTML element.
///
/// Created during execution of the (element) macro and the element-specific wrappers. Offers a
/// "stringly-typed" API for mutating the contained DOM nodes, adhering fairly closely to the
/// upstream web specs.
pub struct MemoElement {
    curr: Cell<Option<Node>>,
    node: Node,
}

impl MemoElement {
    fn new(node: Node) -> Self {
        Self {
            curr: Cell::new(None),
            node,
        }
    }

    /// Retrieves access to the raw HTML element underlying the (MemoElement).
    ///
    /// Because this offers an escape hatch around the memoized mutations, it should be used with
    /// caution. Also because of this, it has a silly name intended to loudly announce that
    /// care must be taken.
    ///
    /// Code called by the root function of your application will be run quite frequently and
    /// so the tools for memoization are important for keeping your application responsive. If you
    /// have legitimate needs for this API, please consider filing an issue with your use case so
    /// the maintainers of this crate can consider "official" ways to support it.
    pub fn raw_node_that_has_sharp_edges_please_be_careful(&self) -> Node {
        self.node.clone()
    }

    // FIXME this should be topo-nested
    // TODO and it should be able to express its slot as an annotation
    /// Declare an attribute of the element, mutating the actual element's attribute when the passed
    /// value changes.
    ///
    /// A guard value is stored as a resulting "effect" of the mutation, and removes the attribute
    /// when `drop`ped, to ensure that the attribute is removed when this declaration is no longer
    /// referenced in the most recent (`moxie::Revision`).
    pub fn attr(&self, name: &'static str, value: impl ToString) -> &Self {
        topo::call_in_slot(name, || {
            memo_with(
                value.to_string(),
                |v| {
                    self.node.set_attribute(name, v);
                    scopeguard::guard(self.node.clone(), move |elem| elem.remove_attribute(name))
                },
                |_| {},
            )
        });
        self
    }

    // FIXME this should be topo-nested
    /// Declare an event handler on the element.
    ///
    /// A guard value is stored as a resulting "effect" of the mutation, and removes the attribute
    /// when `drop`ped, to ensure that the attribute is removed when this declaration is no longer
    /// referenced in the most recent (`moxie::Revision`).
    ///
    /// Currently this is performed on every Revision, as changes to event handlers don't typically
    /// affect the debugging experience and have not yet shown up in performance profiles.
    pub fn on<Ev>(&self, callback: impl FnMut(Ev) + 'static) -> &Self
    where
        Ev: 'static + Event,
    {
        topo::call_in_slot(Ev::NAME, || {
            memo_with(
                moxie::embed::Revision::current(),
                |_| EventHandle::new(&self.node, callback),
                |_| {},
            );
        });
        self
    }

    fn ensure_child_attached(&self, new_child: &Node) {
        let prev_sibling = self.curr.replace(Some(new_child.clone()));

        let existing = if prev_sibling.is_none() {
            self.node.first_child()
        } else {
            prev_sibling.and_then(|p| p.next_sibling())
        };

        if let Some(ref existing) = existing {
            if existing != new_child {
                self.node.replace_child(new_child, existing);
            }
        } else {
            self.node.append_child(new_child);
        }
    }

    /// Declare the inner contents of the element, usually declaring children within the inner
    /// scope. After any children have been run and their nodes attached, this clears any trailing
    /// child nodes to ensure the element's children are correct per the latest declaration.
    // FIXME this should be topo-nested
    pub fn inner<Ret>(&self, children: impl FnOnce() -> Ret) -> Ret {
        let elem = self.node.clone();
        let mut last_desired_child = None;
        let mut ret = None;
        illicit::child_env!(MemoElement => MemoElement::new(self.node.clone())).enter(|| {
            topo::call(|| {
                ret = Some(children());

                // before this melement is dropped when the environment goes out of scope,
                // we need to get the last recorded child from this revision
                last_desired_child = Some(illicit::Env::expect::<MemoElement>().curr.replace(None));
            })
        });
        let last_desired_child = last_desired_child.unwrap();
        let ret = ret.unwrap();

        // if there weren't any children declared this revision, we need to make sure we clean up
        // any from the last revision
        let mut next_to_remove = if let Some(c) = last_desired_child {
            c.next_sibling()
        } else {
            elem.first_child()
        };

        while let Some(to_remove) = next_to_remove {
            next_to_remove = to_remove.next_sibling();
            elem.remove_child(&to_remove).unwrap();
        }

        ret
    }
}

impl Debug for MemoElement {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("MemoElement")
            .field("node", &self.node)
            .finish()
    }
}
