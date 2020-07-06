/// Accepts an XML-like expression and expands it to builder method calls.
///
/// # Outputs
///
/// The `mox!` macro's contents are expanded to a nested builder pattern.
///
/// ## Tags
///
/// Each tag expands to a function call with the same name as the tag, with the
/// tag's arguments passed through as function arguments. The function call and
/// all attributes and children are wrapped in `#[topo::nested]` to create the
/// correct topology.
///
/// Each attribute expands to a method called on the value returned from the tag
/// opening or the previous attribute. The attribute name is used as the method
/// name, with the attribute value passed as the argument.
///
/// A tag with children has each child passed as the argument to a call to
/// `.child(...)`, one per child in order of declaration. The calls to `child`
/// come after attributes.
///
/// After all attributes and children, `.build()` is called on the final value.
///
/// ## Fragments
///
/// Fragments are not currently supported.
///
/// ## Content/Text
///
/// Content items are wrapped in calls to `text(...)`.
///
/// If a content item is a formatter the contained expression is first wrapped
/// in the `format!(...)` macro.
///
/// # Inputs
///
/// Each macro invocation must resolve to a single item. Items can be tags,
/// fragments, or content.
///
/// [snax](https://docs.rs/snax) is used to tokenize the input as [JSX]\(ish\).
///
/// ## Tags
///
/// Tags always have a name and can have zero or more arguments, attributes, and
/// children.
///
/// They take the form `<NAME _=(ARGS ...) ATTR=VAL ...> CHILDREN </NAME>`.
/// Each optional portion can be omitted.
///
/// ### Arguments
///
/// A tag's arguments are wrapped with `_=(` and `)`, delimited by `,` (comma),
/// and must precede any attributes. Each argument is a Rust expression.
///
/// If there are no arguments the `_=()` wrapper must be omitted.
///
/// ### Attributes
///
/// Each attribute takes the form `NAME=VAL` where `NAME` is an identifier and
/// `VALUE` is an expression.
///
/// If the attribute's name is `async`, `for`, `loop`, or `type` an underscore
/// is appended to avoid colliding with the Rust keyword.
///
/// ### Children
///
/// Tags have zero or more nested items (tags, fragments, content) as children.
///
/// If there are no children the tag can be "self-closing": `<NAME ... />`.
///
/// ## Fragments
///
/// Fragments are opened with `<>` and closed with `</>`. Their only purpose is
/// to provide a parent for children. They do not accept arguments or
/// attributes.
///
/// ## Content
///
/// Content items represent text. They are delimited with `{` and `}`. They can
/// optionally be opened with `{%` to denote a "formatter" item.
///
/// [JSX]: https://facebook.github.io/jsx/
#[proc_macro_hack::proc_macro_hack(support_nested)]
pub use mox_impl::mox;
