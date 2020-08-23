# prettiest

[prettiest](https://docs.rs/prettiest) provides pretty-printing `Debug` and `Display` impls
for Javascript values in the [wasm-bindgen](https://docs.rs/wasm-bindgen) crate.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.2.0] - 2020-08-22

### Added

- `Pretty` trait offers a `.pretty()` method to anything `AsRef<JsValue>`.
- `Prettified` implements `Display`.
- `Prettified::skip_property` allows deleting properties that aren't useful to print, like
  `timeStamp`.

### Fixed

- Cycles in objects are broken correctly.
- Null and undefined values are handled correctly.
- Values not explicitly handled are represented by `Pretty::Unknown`.
- Objects print properties from their prototype chain.

### Changed

- `Pretty` enum renamed to `Prettified` to allow trait to be named `Pretty`.
- Objects print non-function properties before function properties.
- Objects print their properites in prototype-order.
- HTML elements, window and document all have abbreviated output.

## [0.1.0] - 2020-08-20

Initial release. Only sort of works -- not recommended for use.
