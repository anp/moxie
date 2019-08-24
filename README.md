# moxie

![Crates.io](https://img.shields.io/crates/l/moxie.svg)

## Hands On

Want to try things out? Run these commands in two terminals:

```
$ cargo dom-flow
```

This will build all of the web examples and watch for local changes.

```
$ cargo server
```

This will start an HTTP server for the static files in the repo, and opens the repo directory in a
browser.

## About

`moxie` (<small>/ˈmäksē/</small>) is a lightweight platform-agnostic UI runtime written in Rust, powering a strongly-typed declarative programming style with minimal interaction latency.

`moxie` is principally inspired by [React][react] and more specifically the recent [Hooks API][hooks]. It aims to smoothly bridge the gap between stateless tools like [dear imgui][dear] and "traditional" UI paradigms with manually managed graphs of stateful, mutable objects. There are many interesting parallels in our design to those of recently announced UI frameworks [Jetpack Compose][compose] and [SwiftUI][swiftui], although a more in-depth comparison hasn't yet been made. Also, in the course of looking for prior art (_ahem_ googling "memoized imgui"), I found a very interesting [thread on LtU](http://lambda-the-ultimate.org/node/4561) discussing various commenters' efforts and curiosities -- it's a fun read.

[react]: https://reactjs.org
[hooks]: https://reactjs.org/docs/hooks-intro.html
[dear]: https://github.com/ocornut/imgui
[swiftui]: https://developer.apple.com/xcode/swiftui/
[compose]: https://developer.android.com/jetpack/compose

## Contributing

The project is not yet ready for external contributions, although I hope to change this message before RustConf 2019 (we'll see!). In the meantime, I'm currently trying out a [Discord server](https://discord.gg/vTAzk3d) for chat and will continue working towards the project's maturity enough to onboard!

See [CONDUCT.md](CONDUCT.md) for the project's Code of Conduct.

## License

Licensed under either of

  * [Apache License, Version 2.0](LICENSE-APACHE)
  * [MIT license](LICENSE-MIT)

at your option.
