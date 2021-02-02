# illicit

The [illicit](https://docs.rs/illicit) crate provides type-indexed thread-local environments.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [1.1.2] - 2021-02-01

### Changed

- Internal refactors to reduce code size.

## [1.1.1] - 2020-08-20

### Added

- Guard returned from `get()` impls `Debug`.

## [1.1.0] - 2020-07-12

### Added

- `AsContext` trait offers a shorthand for adding a single type to the local environment.

## [1.0.0] - 2020-07-03

Commiting to the current API for future `1.*` releases.

## [0.10.0] - 2020-07-03

### Added

- `impl From<Snapshot> for Layer` allows reusing collected snapshots.

### Removed

- `#![feature(track_caller)]` (requires >= nightly-2020-07-02 until 1.46 is beta/stable)
- Support for owned/cloned arguments in `from_env`.

### Changed

- `EnvSnapshot` renamed to `Snapshot`.
- `Env::get`, `Env::expect`, `Env::snapshot` moved to `get`, `expect`, free functions and
  `Snapshot::get` associated function.
- `Env` renamed to `Layer`.
- `Layer::with` renamed to `Layer::offer`.
- `from_env` requires at least one argument.
- `from_env` adds "Environment Expectations" doc comment to expanded function.
- Additions to the environment track their location with `std::panic::Location`.

## [0.9.2] - 2019-12-25

### Changed

- Microbenchmark times were improved.

## [0.9.1] - 2019-11-22

### Fixed

- Relaxed intra-crate doc lint to allow docs.rs publish to succeed.

## [0.9.0] - 2019-11-19

Initial release which consists mostly of code extracted from `topo`.

### Added

- `Env`, `EnvSnapshot`, `from_env`.

### Changed

- `Env` implements `Debug`.
- `expect` prints the current environment as a stack of layers on failure.

### Removed

- `Scope`
- `root! { ... }`
