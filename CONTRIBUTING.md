# Contributing to moxie

Hello! The project is still very early but we're so excited to see you here!

The core bits have only recently stabilized enough to invite contribution, and we're still working
on a body of starter issues and docs that can enable more participation. If this doesn't scare you
away, then read on.

The project currently uses a [Discord server](https://discord.gg/vTAzk3d) for chat and we
recommend joining if you're interested in contributing at this phase. If you would be interested in
contributing but prefer other communications media, please let us know! It's certainly not
required to contribute, but GitHub issues are a bit constraining for the level of ambiguity in the
project today.

## Code of Conduct

See the [project's Code of Conduct](./CODE_OF_CONDUCT.md) for more details.

## Continuous Integration

CI is run via [GitHub Actions](https://github.com/anp/moxie/actions), and
[configured in-tree](.github/workflows/main.yml).

### Landing PRs

When a PR is ready to merge, apply the `autorebase` label. This will inform [the autorebase GitHub App](https://github.com/tibdex/autorebase) to merge the PR if all required checks pass, and to attempt to rebase the PR onto master when the latter changes.

GitHub now offers an option to require that a branch is up-to-date before it is merged in a PR, which is enabled for the repository to aid in implementing [The Not Rocket Science Rule of Software Engineering](https://graydon.livejournal.com/186550.html):

> automatically maintain a repository of code that always passes all the tests

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
$ cargo server
```

This will start a local HTTP server providing access to the project directory. It also watches the
filesystem for changes to files it has served, delivering notifications when any of them
change. The examples include `project-project/reloadOnChanges.js` which opens a websocket and
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
* `index.html`

(Dependabot discovers the workspace members from the root manifest.)
