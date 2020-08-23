# TodoMVC Example

Commands all assume the working directory is the repository root.

## Serving

Build the example and start the project's local HTTP server:

```
$ cargo build-dom-todo
$ cargo ofl serve
```

## Using

To use the example locally, follow the directions for [serving](#serving) the project and
navigate to `http://[::1]:8000/dom/examples/todo/index.html` in your browser.

## Tests

Unit & integration tests can be run with `cargo test-dom-todo`.

### End-to-end

End-to-end tests are run with [Cypress](https://cypress.io) which requires
[Node.js](https://nodejs.org) to run.

The tests require a running HTTP server and a current build. The `test-dom-todo-e2e` cargo command
starts an HTTP server for the test and only requires a build to have been run first:

```
$ cargo build-dom-todo
$ cargo test-dom-todo-e2e
```

If you've already followed the [serving](#serving) instructions the e2e tests should be run
directly:

```
$ cd dom/examples/todo/e2e
$ npx cypress run
```
