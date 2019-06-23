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
$ cargo watch-core
```

See [its definition](./.cargo/config) for details.

### moxie-dom dev env

* [node >= LTS](https://nodejs.org)
* [wasm-pack](https://rustwasm.github.io/wasm-pack/)
