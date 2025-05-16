# moxie server-side rendering example

A proof-of-concept implementation of rendering HTML in gotham using moxie-dom without any browser
dependencies.

`cargo run` starts a server that listens on `127.0.0.1:7878`, serving HTML based on the URL after 
`/paths/*`.

`cargo test` uses gotham's (very nice) test server tool to verify the behavior matches what we
expect.
