# Contributing to moxie

## Bottom line: not yet

This is a placeholder for if/when the project is ready to onboard more contributors.

## Development environment

### Requirements

* [rustup](https://rustup.rs)
  * `rustup component add clippy rustfmt`
* [cargo-watch](https://crates.io/crates/cargo-watch)
* [cargo-script](https://crates.io/crates/cargo-script)

### Workflows

#### Core libraries

From the project root, this command will run the default development loop:

```shell
$ cargo core-flow
```

See [its definition](./.cargo/config) for details.

#### dom examples

The moxie-dom examples are compiled as part of `core-flow`, and you can serve them locally with `cargo serve`. This will start a local HTTP server providing access to the project directory.

#### Releases

##### topo

`topo` and `topo-macro` must be released in sync.

#### New crates

Things to update:

* `Cargo.toml`
* `.cargo/config`
* `.dependabot/config.yml`
* `.github/workflows/main.yml`
