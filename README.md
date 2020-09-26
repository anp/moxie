<img src="assets/logo.png" alt="moxie logo" width="175"/>

# moxie

![crates.io](https://img.shields.io/crates/v/moxie)
![License](https://img.shields.io/crates/l/moxie.svg)
[![codecov](https://codecov.io/gh/anp/moxie/branch/main/graph/badge.svg)](https://codecov.io/gh/anp/moxie)

## More Information

For more information about the moxie project, see the [website](https://moxie.rs).

## Crates in this repository

### moxie-dom and other Web/HTML/JS crates

[`moxie-dom`](./dom) offers APIs for incrementally constructing HTML in the browser and elsewhere.
See [examples](./dom/examples) for demos, live versions of which are available on
[the project's website](https://moxie.rs/#web).

[`augdom`](./dom/augdom) wraps the web's [DOM] API and augments it with non-Web polyfills for e.g.
server-side rendering.

[`prettiest`](./dom/prettiest) is a Rust pretty-printer for JavaScript values.

[`raf`](./dom/raf) is a runloop scheduler built with [`requestAnimationFrame`].

### Platform-agnostic crates

[`moxie`](./src) is an [incremental] runtime offering caching and "reactive" state management.

[`dyn-cache`](./dyn-cache) offers generational caches for arbitrary Rust types, allowing a single
database struct to be used for any number of static or dynamic queries.

[`illicit`](./illicit) offers thread-local type-indexed implicit context singletons.

[`topo`](./topo) creates reproducible identifiers for locations in the runtime callgraph. Used to
generate query scopes for `dyn-cache` storage.

[`mox`](/.mox) implements an XML-like syntax for Rust builders inspired by [JSX].

## Contributing and Code of Conduct

See [CONTRIBUTING.md](CONTRIBUTING.md) for overall contributing info and [CONDUCT.md](CODE_OF_CONDUCT.md)
for the project's Code of Conduct. The project is still early in its lifecycle but we welcome
anyone interested in getting involved.

## License

Licensed under either of

  * [Apache License, Version 2.0](LICENSE-APACHE)
  * [MIT license](LICENSE-MIT)

at your option.

[DOM]: https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model
[`requestAnimationFrame`]: https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame
[incremental]: http://adapton.org/
[JSX]: https://facebook.github.io/jsx/
