# TodoMVC Example

Commands all assume the working directory is the repository root.

## Serving

First install [Trunk](https://trunkrs.dev/).

```
$ trunk serve
```

## Using

To use the example locally, follow the directions for [serving](#serving) the project and
navigate to `http://[::1]:8000/dom/examples/todo/index.html` in your browser.

## Tests

Unit & integration tests can be run with `cargo test-dom-todo`.

### End-to-end

End-to-end tests are run with [Cypress](https://cypress.io) which requires
[Node.js](https://nodejs.org) to run.

If you've already followed the [serving](#serving) instructions the e2e tests can be run from the
Cypress UI directly. Start the test runner with the `cypress` VSCode task or run the following:

```
$ cd dom/examples/todo/e2e; npx cypress run
```

#### One-off

The tests require a running HTTP server and a current build. The `test-dom-todo-e2e` cargo command
runs a build, and starts an HTTP server for the test before running it:

```
$ cargo test-dom-todo-e2e
```
