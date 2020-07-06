# mox

[mox] implements the `mox!` macro: "a Mockery Of X(ML)". A [JSX]-like Rust DSL for calling builders.

[mox]: https://docs.rs/moxie
[JSX]: https://reactjs.org/docs/introducing-jsx.html

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.10.0] - unreleased

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
