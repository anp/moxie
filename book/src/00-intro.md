## About

`moxie` (<small>/ˈmäksē/</small>) is a lightweight platform-agnostic UI runtime written in Rust, powering a strongly-typed declarative programming style with minimal interaction latency.

`moxie` is principally inspired by [React][react] and more specifically the recent [Hooks API][hooks]. It aims to smoothly bridge the gap between stateless tools like [dear imgui][dear] and "traditional" UI paradigms with manually managed graphs of stateful, mutable objects. There are many interesting parallels in our design to those of recently announced UI frameworks [Jetpack Compose][compose] and [SwiftUI][swiftui], although a more in-depth comparison hasn't yet been made. Also, in the course of looking for prior art (_ahem_ googling "memoized imgui"), I found a very interesting [thread on LtU](http://lambda-the-ultimate.org/node/4561) discussing various commenters' efforts and curiosities -- it's a fun read.

[react]: https://reactjs.org
[hooks]: https://reactjs.org/docs/hooks-intro.html
[dear]: https://github.com/ocornut/imgui
[swiftui]: https://developer.apple.com/xcode/swiftui/
[compose]: https://developer.android.com/jetpack/compose

## Hands On

Want to try things out? Check out 
[the development requirements](CONTRIBUTING.md#development-environment), and run 
these commands in two terminals:

```
$ cargo dom-flow
```

This will build all of the web examples and watch for local changes.

```
$ cargo server
```

This will start an HTTP server for the static files in the repo, and opens the repo directory in a
browser.

Take a look at [`.cargo/config`](.cargo/config) for other subcommand aliases used in the project.

---

random:

# Declarative style

TODO "imperative but idempotent"

"describe the UI *right now*"

partition the render space using function calls

while managing persistent stateful elements

with minimal incremental updates

in order to achieve minimal latency and consistent responsiveness
