# dyn-cache

The [dyn-cache](https://docs.rs/dyn-cache) crate provides incremental caching for Rust function
invocations.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.10.0] - unreleased

### Added

- Crate extracted from `topo::cache` module.
- `{LocalCache,SendCache}::cache` wraps `cache_with` for types that impl `Clone`.
- `Gc` trait is public, inner cache types implement it.
- `Hashed` has an additional type parameter for the type of hasher used to create it.

### Changed

- Rename `Cache`/`SharedCache` to `SendCache`/`SharedSendCache`.
