//! Quoting from [DOM Testing Library]'s motivations, as this crate's test
//! utilities are similar in design:
//!
//! > The more your tests resemble the way your software is used, the more
//! confidence they can give you.
//!
//! > As part of this goal, the utilities this library provides facilitate
//! querying the DOM in the same way the user would. Finding form elements by
//! their label text (just like a user would), finding links and buttons from
//! their text (like a user would), and more.
//!
//! These tools lend themselves to this basic test design:
//!
//! 1. setup test DOM
//! 1. execute user-oriented queries to find nodes of interest (see
//!    [`Query::find`] and [`Finder::by_label_text`])
//! 1. fire events as a user would (see [`crate::Dom::dispatch`])
//! 1. wait for async queries to complete (see [`Found::until`] and
//!    [`Until`])
//! 1. assert on results
//!
//! TODO write examples that work in doctests
//!
//! [DOM]: https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model/Introduction
//! [DOM Testing Library]: https://testing-library.com/docs/dom-testing-library/intro

use super::Dom;
use futures::prelude::*;
use std::fmt::Debug;

/// A type which can be queried as a DOM container, returning results from its
/// subtree.
pub trait Query: Sized {
    /// Begin a subtree query. The returned value supports methods like
    /// [`Finder::by_label_text`] which create queries against this container's
    /// children.
    fn find(&self) -> Finder<Self>;
}

impl<N> Query for N
where
    N: Dom,
{
    fn find(&self) -> Finder<Self> {
        Finder { target: self }
    }
}

/// The outcome of a failed query.
#[derive(Debug)]
pub enum QueryError<'a, N> {
    /// Couldn't find any matching nodes.
    Missing {
        /// the original query
        lookup: &'a dyn Debug,
    },
    /// Found more nodes than the 1 requested.
    TooMany {
        /// the first node we found
        matched: N,
        /// unexpected nodes
        extra: Vec<N>,
        /// the original query
        lookup: &'a dyn Debug,
    },
}

/// Executes a search strategy over a DOM container's subtree via depth-first
/// pre-order traversal.
#[derive(Debug)]
pub struct Finder<'n, N> {
    target: &'n N,
}

macro_rules! strat_method {
    (
        $(#[$outer:meta])+
        $met:ident $strat:ident
    ) => {
        $(#[$outer])*
        pub fn $met<'find, 'pat>(&'find self, pattern: &'pat str) -> Found<'find, 'pat, 'node, N> {
            Found { strat: Strategy::$strat, pattern, finder: self }
        }
    };
}

impl<'node, N> Finder<'node, N> {
    strat_method! {
        /// Find by `label`'s or `aria-label`'s normalized text content.
        ///
        /// The default choice for selecting form elements as it most closely
        /// mirrors how users interact with forms.
        by_label_text       LabelText
    }

    strat_method! {
        /// Find by `input`'s `placeholder` value.
        ///
        /// Used for form fields, choose if [`Finder::by_label_text`] is not available.
        by_placeholder_text PlaceholderText
    }

    strat_method! {
        /// Find by aria `role`.
        ///
        /// The default choice for interactive elements like buttons.
        by_role             Role
    }

    strat_method! {
        /// Find by element's normalized text content.
        by_text             Text
    }

    strat_method! {
        /// Find by form element's current `value`.
        by_display_value    DisplayValue
    }

    strat_method! {
        /// Find by `img`'s `alt` attribute.
        by_alt_text         AltText
    }

    strat_method! {
        /// Find by `title` attribute's or svg `title` tag's normalized text content.
        by_title            Title
    }

    strat_method! {
        /// Find by `data-testid` attribute. Not visible to humans, only
        /// use as a last resort.
        by_test_id          TestId
    }
}

/// The final description of a subtree query. The methods on this struct
/// execute the underlying search and return the results in various forms.
#[derive(Debug)]
pub struct Found<'find, 'pat, 'node, N> {
    strat: Strategy,
    pattern: &'pat str,
    finder: &'find Finder<'node, N>,
}

impl<'find, 'pat, 'node, N> Found<'find, 'pat, 'node, N>
where
    N: Dom + Debug,
{
    /// Wrap the query in a [`MutationObserver`] with async methods that resolve
    /// once the wrapped query could succeed or a 1 second timeout has expired.
    ///
    /// [`MutationObserver`]: https://developer.mozilla.org/en-US/docs/Web/API/MutationObserver
    pub fn until(&self) -> Until<'_, 'find, 'pat, 'node, N> {
        Until::new(self)
    }

    /// Execute the query and return the only matching node in the queried
    /// subtree.
    ///
    /// # Panics
    ///
    /// If more than one matching node is found.
    pub fn one(&self) -> Result<N, QueryError<'_, N>> {
        let mut matches = self.many().into_iter();
        let matched = matches.next();
        let extra = matches.collect::<Vec<_>>();

        if let Some(matched) = matched {
            if extra.is_empty() {
                Ok(matched)
            } else {
                Err(QueryError::TooMany { matched, extra, lookup: self })
            }
        } else {
            Err(QueryError::Missing { lookup: self })
        }
    }

    /// Execute the query and return a `Vec` of matching nodes in the queried
    /// subtree.
    pub fn many(&self) -> Vec<N> {
        // first accumulate the subtree
        let mut candidates = Vec::new();
        collect_children_dfs_preorder(self.finder.target, &mut candidates);

        // then keep only those which match
        candidates.retain(|n| self.matches(n));

        candidates
    }

    fn normalize(s: impl AsRef<str>) -> String {
        s.as_ref().split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn matches(&self, node: &N) -> bool {
        use Strategy::*;
        match self.strat {
            Text => Some(node.get_inner_text()),
            // TODO(#120) add tests and make sure this is correct
            LabelText => node
                .get_attribute("id")
                .map(|id| {
                    let selector = format!("label[for={}]", id);
                    self.finder.target.query_selector(&selector).map(|l| l.get_inner_text())
                })
                .flatten(),
            AltText => node.get_attribute("alt"),
            Title => node.get_attribute("title"),
            DisplayValue => node.get_attribute("value"),
            PlaceholderText => node.get_attribute("placeholder"),
            Role => node.get_attribute("role"),
            TestId => node.get_attribute("data-testid"),
        }
        // normalize the string, removing redundant whitespace
        .map(Self::normalize)
        .map(|text| text == self.pattern)
        .unwrap_or(false)
    }
}

/// Performs a depth-first pre-order traversal of the containing node,
/// adding all of its transitive children to `queue`.
fn collect_children_dfs_preorder<N: Dom>(node: &N, queue: &mut Vec<N>) {
    let mut next_child = node.first_child();
    while let Some(child) = next_child {
        collect_children_dfs_preorder(&child, queue);
        next_child = child.next_sibling();
        queue.push(child);
    }
}

/// Which portion of a queried node to examine.
#[derive(Clone, Copy, Debug)]
enum Strategy {
    LabelText,
    PlaceholderText,
    Text,
    DisplayValue,
    AltText,
    Title,
    Role,
    TestId,
}

/// A query which resolves asynchronously
#[derive(Debug)]
pub struct Until<'query, 'find, 'pat, 'node, N> {
    query: &'query Found<'find, 'pat, 'node, N>,
}

impl<'query, 'find, 'pat, 'node, N> Until<'query, 'find, 'pat, 'node, N>
where
    N: Dom + Debug,
{
    fn new(query: &'query Found<'find, 'pat, 'node, N>) -> Self {
        Self { query }
    }

    /// Wait until the query can succeed then return the only matching node
    /// in the queried subtree.
    ///
    /// # Panics
    ///
    /// If more than one matching node is found.
    #[cfg(feature = "webdom")]
    pub async fn one(&self) -> Result<N, QueryError<'_, N>> {
        let mut matches = self.many().await.into_iter();
        let matched = matches.next();
        let extra = matches.collect::<Vec<_>>();

        if let Some(matched) = matched {
            if extra.is_empty() {
                Ok(matched)
            } else {
                Err(QueryError::TooMany { matched, extra, lookup: self })
            }
        } else {
            Err(QueryError::Missing { lookup: self })
        }
    }

    /// Wait until the query can succeed then return a `Vec` of matching nodes
    /// in the queried subtree.
    #[cfg(feature = "webdom")]
    pub async fn many(&self) -> Vec<N> {
        macro_rules! try_query {
            () => {{
                let current_results = self.query.many();
                if !current_results.is_empty() {
                    return current_results;
                }
            }};
        }

        let mut mutations = self.query.finder.target.observe_mutations();
        let timeout = gloo_timers::future::TimeoutFuture::new(1_000);
        futures::pin_mut!(timeout);

        try_query!(); // see if we can eagerly eval
        loop {
            futures::select_biased! {
                _ = timeout.as_mut().fuse() => {
                    try_query!(); // first see if we can succeed after timing out
                    return Vec::new(); // return empty results if we still fail
                },
                _ = mutations.next().fuse() => try_query!(),
            }
        }
    }
}
