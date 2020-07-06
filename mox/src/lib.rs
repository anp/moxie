/// Accepts an XML-like expression and expands it to builder method calls.
///
/// # Outputs
///
/// The `mox!` macro's contents are expanded to a nested builder pattern.
///
/// ## Tags
///
/// Each tag expands to a function call with the same name as the tag. The
/// function call, all attributes, and children are wrapped in [`topo::call`] to
/// create a nested scope in the callgraph.
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
/// Text nodes are wrapped in calls to `text(...)`.
///
/// If an expression is a formatter, the arguments are wrapped
/// in the `format!(...)` macro before being treated as a text node.
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
/// They take the form `<NAME ATTR=VAL ...> CHILDREN </NAME>`. Each optional
/// portion can be omitted.
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
/// ## Expressions
///
/// Raw Rust expressions can be inserted as a child node. They are delimited
/// with `{` and `}`.
///
/// ## Format expressions
///
/// Expressions can optionally be opened with `{%` to denote a "formatter" item.
/// The enclosed tokens are passed
///
/// # Example
///
/// ```
/// use mox::mox;
///
/// #[derive(Debug, PartialEq)]
/// struct Tag {
///     name: String,
///     children: Vec<Tag>,
/// }
///
/// fn built() -> TagBuilder {
///     TagBuilder::default()
/// }
///
/// #[derive(Default)]
/// struct TagBuilder {
///     name: Option<String>,
///     children: Vec<Tag>,
/// }
///
/// impl TagBuilder {
///     fn name(mut self, name: impl Into<String>) -> Self {
///         self.name = Some(name.into());
///         self
///     }
///
///     fn child(mut self, child: Tag) -> Self {
///         self.children.push(child);
///         self
///     }
///
///     fn build(self) -> Tag {
///         Tag { name: self.name.unwrap(), children: self.children }
///     }
/// }
///
/// assert_eq!(
///     mox! {
///         <built name="alice">
///             <built name="bob"/>
///         </built>
///     },
///     Tag {
///         name: String::from("alice"),
///         children: vec![Tag { name: String::from("bob"), children: vec![] }],
///     },
/// );
/// ```
///
/// [JSX]: https://facebook.github.io/jsx/
#[proc_macro_hack::proc_macro_hack(support_nested)]
pub use mox_impl::mox;

#[doc(hidden)]
pub use topo;
