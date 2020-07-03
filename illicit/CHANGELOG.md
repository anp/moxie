# illicit

The [illicit](https://docs.rs/illicit) crate provides type-indexed thread-local environments.

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.10.0-pre] - unreleased

### Added

- `impl From<Snapshot> for Layer` allows reusing collected snapshots.
 
### Removed

- `#![feature(track_caller)]`

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
