# mox

[mox] implements the `mox!` macro: "a Mockery Of X(ML)". A [JSX]-like Rust DSL for calling builders.

[mox]: https://docs.rs/moxie
[JSX]: https://reactjs.org/docs/introducing-jsx.html

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.11.0] - 2021-01-10

### Added

- "Attribute init shorthand" allows pulling an attribute from an identically-named binding n the
  local scope:

  ```rust
  let onclick = |_| { ... };
  mox!(<button onclick>"click me?"</button>)
  ```

- Module-nested tag names: `mox!(<krate::module::tag>"foo"</krate::module::tag>)`.
- Attributes support single-expression values without braces: `<button disabled=true/>`.
- XML comments: `mox!(<div> <!-- COMMENT HERE --> </div>)`.

### Changed

- `mox!` invocations are now lexed by the [syn-rsx](https://docs.rs/syn-rsx) crate.
- Non-tag children have `.into_child()` appended to them.

## [0.10.0] - 2020-07-06
### Removed

- Support for `_=(...)` style function invocation in tags.

### Added

- Implementation is now in `mox-impl` crate to allow publication separately from `moxie`.

### Changed

- Handles collisions with Rust keywords when used as attributes.
- Expands directly to builder methods rather than emulating them with macro calls.
- Expanded builder syntax is compatible with owned builders & typed parent/child bindings.
- Each tag is wrapped in its own `topo::call(...)`.

## [0.2.0] - 2019-11-19

This version was a complete rewrite in tandem with `moxie` v0.2.0.

## [0.1.0] - 2018-11-09

Initial release.
