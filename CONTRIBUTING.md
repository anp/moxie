# Contributing to moxie

GitHub issues are used for tracking and the project has a [Discord
server](https://discord.gg/vTAzk3d).

## Code of Conduct

See the [project's Code of Conduct](./CODE_OF_CONDUCT.md) for more details.

## Continuous Integration

CI is run via [GitHub Actions](https://github.com/anp/moxie/actions), and
[configured in-tree](.github/workflows/main.yml).

### Landing PRs

GitHub now offers an option to require that a branch is up-to-date before it is merged in a PR,
which is enabled for the repository to aid in implementing [The Not Rocket Science Rule of Software
Engineering](https://graydon.livejournal.com/186550.html):

> automatically maintain a repository of code that always passes all the tests

## Development environment

### Requirements

* [rustup](https://rustup.rs)
* [bazelisk](https://github.com/bazelbuild/bazelisk?tab=readme-ov-file#installation)
  * usually a good idea to set this as `bazel` in your `PATH`
  * see <https://github.com/bazelbuild/bazelisk/issues/571> for potential installation improvements
* [buildifier](https://github.com/bazelbuild/buildtools/blob/main/buildifier/README.md)
  * also <https://github.com/bazelbuild/buildtools/releases>, drop it in `PATH`
* (optional) [bazel-watcher](https://github.com/bazelbuild/bazel-watcher)
  * also <https://github.com/bazelbuild/bazel-watcher/releases>, drop it in `PATH`

### VSCode users

Open the `moxie.code-workspace` file as a workspace to get a preconfigured setup for the bazel
build.

### Workflows

Note: if you haven't installed `bazel-watcher` replace `ibazel` with `bazel` below and rerun the
command when you've made changes you'd like to see reflected in test outputs.

#### Build everything

The default build task in vscode, or:

```shell
ibazel build //...
```

#### Test everything

The default test task in vscode, or:

```shell
ibazel test //...
```

#### Formatting

If your editor isn't configured to format-on-save, you can run

```shell
bazel run @rules_rust//:rustfmt
```

to run `rustfmt` on all the source files the build system knows about.

#### Core libraries

From the project root, this command will run the default development loop:

```shell
ibazel test //:core_library_tests
```

#### moxie-dom

The main workflow for the dom library:

```shell
cargo dom-flow
```

To view examples, in a separate terminal:

```shell
cargo server
```

This will start a local HTTP server providing access to the project directory. It also watches the
filesystem for changes to files it has served, delivering notifications when any of them
change. The server injects the necessary JavaScript into each HTML page to open a websocket
connection to listen for changes, reloading when changes occur.

##### End-to-end tests

The TodoMVC example app has some e2e tests which use [cypress.io] and thus require a recent Node/npm
installation.

```shell
cd dom/examples/todo/e2e
npx cypress open
```

Alternatively, there is a project-local VSCode task configured which will open cypress when the
workspace is opened (assuming one enables auto-tasks for this workspace).

#### Releases

During development all non-tool crate versions should be suffixed with `-pre` indicating a
pre-release of some kind. To release a version of a crate, publish a commit to `origin/main/HEAD`
without the pre-release suffix, making sure to update CHANGELOGs appropriately. The project's
continuous integration ensures that any "release" versions (without `-pre`) have been published to
crates.io.

After a release, all version numbers should be incremented and have `-pre` re-appended. PRs are
expected to bump the version number of the crate they're modifying behind the `-pre` suffix as well
as updating the relevant CHANGELOGs.

Changing the version of a crate in the repository should be done by running `cargo ofl versions`.

#### New crates

Things to update:

* `Cargo.toml`
* `.cargo/config`
* `.github/workflows/main.yml`
* `index.html`

(Dependabot discovers the workspace members from the root manifest.)
