# dyn-cache

The [dyn-cache](https://docs.rs/dyn-cache) crate provides incremental caching for Rust function
invocations.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.10.0] - unreleased

### Added

- Crate extracted from `topo::cache` module.
- `{LocalCache,SendCache}::cache` wraps `cache_with` for types that impl `Clone`.
- `{LocalCache,SendCache}::hold` wraps `cache_with` for queries that don't need returns.
- `CacheMiss` struct is used to ensure storage happens where the failed lookup happened.

### Changed

- Rename `Cache`/`SharedCache` to `SendCache`/`SharedSendCache`.
