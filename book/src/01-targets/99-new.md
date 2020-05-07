# Adding new targets

Expressing an interactive system "in terms of moxie's tools" requires making some decisions.

## A core choice in moxie: Builders

Existing UI systems tend to come with complex objects that require nuanced initialization, often
with many parameters, some of which are optional and some which are not. Rust has one main tool
for describing those initializations: the [builder pattern]. It is possible to describe complex
"mixed-optionality" initialization *without* the builder pattern in Rust, but it's so prevalent
in Rust that it's [officially recommended][builder pattern].

The [`mox!`][mox] macro ("**M**ockery **O**f **X**ML") is essentially an XML syntax for Rust
builders. See [its documentation][mox] in the moxie crate for information about exactly how it
expands.

## Finding Event Loops

TODO

## Memoization

TODO

## Persistence

TODO

## Parent/child relationships

TODO

[mox]: https://docs.rs/moxie/latest/moxie/macro.mox.html
[builder pattern]: https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder