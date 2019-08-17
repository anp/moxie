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
change. The examples include `tools/project-server/reloadOnChanges.js` which opens a websocket and 
reloads the page when changes to the examples are detected.

#### Releases

During development all non-tool crate versions should be suffixed with `-pre` indicating a
pre-release of some kind. To release a version of a crate, publish a commit to `origin/master/HEAD`
without the pre-release suffix. The project's continuous integration ensures that any "release"
versions (without `-pre`) have been published to crates.io.

After a release, all version numbers should be incremented and have `-pre` re-appended. PRs are
expected to bump the version number of the crate they're modifying behind the `-pre` suffix.

#### New crates

Things to update:

* `Cargo.toml`
* `.cargo/config`
* `.github/workflows/main.yml`

(Dependabot discovers the workspace members from the root manifest.)
