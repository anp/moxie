# moxie

moxie supports incremental "declarative" Rust code for interactive systems.
It comes with a lightweight event loop runtime that supports granular
reuse of arbitrary work, state change notifications, and async loaders.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.7.1] - 2021-05-05

### Added

- `Key::mutate` allows naive clone-update-compare access to a state variable.
- `#[moxie::updater(...)]` attribute macro supports creating a `Key` wrapper with shorthand for
  mutating methods.
- `wasm-bindgen` cargo feature which enables correct usage of parking_lot on wasm32 targets.

### Fixed

- Some new clippy lints.

### Changed

- No longer requires a nightly cargo to build.

## [0.7.0] - 2020-09-27

### Added

- Support borrowed arguments to cache functions to avoid cloning on every revision.
- Common trait implementations for `Key`.
- Updated crate & module docs.
- Testing utilities in `testing` module.

### Removed

- Futures loading is no longer feature flagged.
- moxie's cache (previously MemoStorage) is moved to dyn-cache, a new dependency.
- Built-in executor which was only used for testing.
- No longer depends on nightly Rust for `#[track_caller]` -- it's stabilized.
- `mox!` macro is now published separately.

### Changed

- "Memoization" renamed to "caching" in all APIs.
- `Runtime::run_once` allows passing an argument to the root function.
- `Runtime` no longer owns the root function.
- `embed` module renamed to `runtime`.
- State functions return a tuple `(Commit, Key)` instead of just a `Key`.

## [0.2.3] - 2019-12-27

### Fixed

- Incorrect version numbers which prevented 0.2.2 from working from crates.io.

## [0.2.2] - 2019-12-27

### Added

- Depends on nightly Rust for `#[track_caller]` feature.

### Changed

- Update to topo version that produces functions instead of macros from `#[topo::nested]`. No more
  macros! Makes use of `#[track_caller]`.

## [0.2.1] - 2019-11-22

### Added

- Async executor integration w/ futures loading (`load`, `load_once`, ...). Under feature flag.
- `#![forbid(unsafe_code)]`
- `Runtime::run_once` returns the root closure's return value.
- `memo_with`, `once_with` functions that allows non-`Clone` types in storage
- `Key` tracks its callsite.
- `mox!` re-exported.

### Removed

- Attempts at memoizing all components.
- Unnecessary revision bookkeeping for state variables.

### Fixed

- Passing illicit env values to a root function.

## [0.1.1-alpha.0] - 2019-08-17

Initial release in support of moxie-dom.

## [0.1.0] - 2018-11-10

Initial name reservation.
