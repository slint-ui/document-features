# Document your crate's feature flags

[![Crates.io](https://img.shields.io/crates/v/document-features)](https://crates.io/crates/document-features)
[![Documentation](https://docs.rs/document-features/badge.svg)](https://docs.rs/document-features/)

This crate provides a macro that extracts documentation comments from Cargo.toml

To use this crate, add `#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]` in your crate documentation.
The `document_features!()` macro reads your `Cargo.toml` file, extracts feature comments and generates
a markdown string for your documentation.

Use `## ` and `#! ` comments in your Cargo.toml to document features, for example:

```toml
[package.metadata.docs.rs]
all-features = true # ensures that `document-features` is enabled when building docs

[dependencies]
document-features = { version = "0.2", optional = true }
## ...

[features]
## The foo feature enables the `foo` functions
foo = []
## The bar feature enables the [`bar`] module
bar = []

#! ### Experimental features
#! The following features are experimental

## Activate the fusion reactor
fusion = []
```

These comments keep the feature definition and documentation next to each other, and they are then
rendered into your crate documentation.

Check out the [documentation](https://docs.rs/document-features/) for more details.

## Contributions

Contributions are welcome. We accept pull requests and bug reports.

## License

MIT OR Apache-2.0
