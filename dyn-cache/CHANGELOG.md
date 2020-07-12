# dyn-cache

The [dyn-cache](https://docs.rs/dyn-cache) crate provides incremental caching for Rust function
invocations.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.10.0] - unreleased

### Added

- Crate extracted from `topo::cache` module.
- `{LocalCache,SendCache}::cache` wraps `cache_with` for types that impl `Clone`.
- `{LocalCache,SendCache}::hold` wraps `cache_with` for queries that don't need returns.
- `Gc` trait is public, inner cache types implement it.
- `Hashed` and `Query` have an additional type parameter for the type of hasher used to create them.
  Both default to the hasher type currently in use in their respective maps.

### Changed

- Rename `Cache`/`SharedCache` to `SendCache`/`SharedSendCache`.
