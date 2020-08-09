# dyn-cache

The [dyn-cache](https://docs.rs/dyn-cache) crate provides incremental caching for Rust function
invocations.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.12.0] - 2020-08-09

### Changed

- `CacheMiss` handles initialization of borrowed inputs for storage, this removes arguments from
  some lower-level functions.

## [0.11.0] - 2020-08-08

### Fixed

- Nested queries to `SharedLocalCache`/``SharedSendCache` have their intermediate dependencies
  retained as long as a transitive dependent is used in a revision.

## [0.10.0] - 2020-07-19

### Added

- Crate extracted from `topo::cache` module.
- `{LocalCache,SendCache}::cache` wraps `cache_with` for types that impl `Clone`.
- `{LocalCache,SendCache}::hold` wraps `cache_with` for queries that don't need returns.
- `CacheMiss` struct is used to ensure storage happens where the failed lookup happened.

### Changed

- Rename `Cache`/`SharedCache` to `SendCache`/`SharedSendCache`.
