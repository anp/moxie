# Contributing to moxie

## Bottom line: not yet

This is a placeholder for if/when the project is ready to onboard more contributors.

## Continuous Integration

CI is run via [GitHub Actions](https://github.com/anp/moxie/actions), and 
[configured in-tree](.github/workflows/main.yml). 

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

#### moxie-dom

The main workflow for the dom library:

```shell
$ cargo dom-flow
```

To view examples, in a separate terminal:

```shell
$ cargo serve
```

This will start a local HTTP server providing access to the project directory. It also watches the
filesystem for changes to files it has served, delivering notifications when any of them
change. The examples include `scripts/reloadOnChanges.js` which opens a websocket and reloads the
page when changes to the examples are detected.

#### Releases

Use [`cargo-release`](https://github.com/sunng87/cargo-release) 
([reference](https://github.com/sunng87/cargo-release/blob/master/docs/reference.md#bump-level))
to bump the relevant crate versions. The configuration at the workspace root will only create
commits. Release commits on `master` will be published by CI (TODO).

##### topo

`topo` and `topo-macro` must be released in sync.

#### New crates

Things to update:

* `Cargo.toml`
* `.cargo/config`
* `.github/workflows/main.yml`

(Dependabot discovers the workspace members from the root manifest.)
