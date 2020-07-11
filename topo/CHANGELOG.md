# topo

The [topo](https://docs.rs/topo) crate provides incremental caching and identifiers for
repeated function invocations. Together with a change notification mechanism it can be used
to implement a form of [incremental computing](https://en.wikipedia.org/wiki/Incremental_computing).

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.13.0] - unreleased

### Removed

- `cache` module extracted to the `dyn-cache` crate.

## [0.12.0] - 2020-07-06

### Changed

- Return type of `cache::{Cache, LocalCache}::get_if_arg_eq_prev_input` also returns hash of the
  query type, and `cache::{Cache, LocalCache}::store` requires it.

## [0.11.0] - 2020-07-06

### Added

- `cache::Hashed` holds a query key and its hash for later storage.

### Changed

- `cache::{Cache, LocalCache}::get_if_arg_eq_prev_input` now returns `Err(Hashed)` rather than
  `None` when a lookup fails. The `store` function on both types now requires that `Hashed` to be
  passed when inserting new input & output.
- Cache contents are GC'd in a single loop, previously there were two iterations over each
  namespace.

### Fixed

- This release removes the last "known redundant" work in the cache API.

## [0.10.0] - 2020-07-05

### Fixed

- `Id` generation is no longer vulnerable to hashing collisions.

### Added

- #[nested] allows specifying a `slot`.
- `cache::{Cache, GlobalCache}` types for storing interned and memoized values.
- `cache::{SharedCache, SharedGlobalCache}` types for safe multiple-owner access to caches, 
  implementing `cache_with` with careful locking to allow nested calls in the future.
- `root` free function for allowing one to re-root a call topology (i.e. if running inside of a
  broader one).

### Removed

- `Callsite` and `Point` are no longer `pub`.
- `#![feature(track_caller)]` is no longer needed, although until 1.46 hits beta/stable an MSRV of
  nightly-2020-07-02 applies.

### Changed

- `call_in_slot` accepts borrowed slots.
- `Id` renamed to `CallId`.
- `illicit` dependency updated to 1.0.
- `impl Trait` has been removed from public APIs where it may cause accidental `Send`/`!Send`
  contracts.

## [0.9.4] - 2019-12-26

### Changed

- Updated `illicit` dependency to `0.9.0`.

## [0.9.3] - 2019-12-25

### Changed

- `#[track_caller]` is used to generate `Id`s, replacing macros. Requires nightly for now.
- Use `DefaultHasher` instead of `FnvHasher`.

### Added

- `call`, `call_in_slot` functions.

### Removed

- `call!` and `unstable_make_topo_macro!` macros.

## [0.9.2] - 2019-11-23

### Changed

- Using `fnv` crate for hashing `Id`s.

## [0.9.1] - 2019-11-21

### Removed

- `#![warn(intra_doc_resolution_failure)]` was causing docs.rs issues due to root_html_url.

## [0.9.0] - 2019-11-19

### Added

- `#![forbid(unsafe_code)]`
- `call!` accepts a "slot" other than the number of times a callsite has been seen. The callsite
  count is still the default.
- Invoking `call!` when no `Point` has already been entered will now create a new root and enter it
  before executing the block.

### Changed

- Rename `#[bound]` to `#[nested]`.
- Rename `current_callsite_count` to `Callsite::current_count`.

### Removed

- `env!`, `Env`, `#[from_env]` moved to `illicit` crate.
- `root!` removed in favor of creating a new root whenever `call!` is invoked outside of a `Point`.

## [0.8.2] - 2019-08-20

### Fixed

- `root!` no longer hides the outer environment from the called block.

## [0.8.1] - 2019-08-17

### Changed

- `Id`'s `Debug` impl uses hex.

### Fixed

- Incorrect line endings.

## [0.8.0] - 2019-06-23

### Added

- `#[topo::bound]` attaches a function to the topology.
- `root!` and `call!` macros attach arbitrary blocks to a new or the current topology respectively,
  entering new `Point`s for each call, each of which has a (mostly) unique `Id`.
- `env!` macro allows declaring type-indexed implicit variables, produces `Env` instances.

## [0.1.0] - 2019-05-26

Published to reserve name on crates.io.
