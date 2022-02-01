# Document your crate's feature flags

[![Crates.io](https://img.shields.io/crates/v/document-features)](https://crates.io/crates/document-features)
[![Documentation](https://docs.rs/document-features/badge.svg)](https://docs.rs/document-features/)

This crate provide a macro that extracts "documentation" comments from Cargo.toml

In order to use this crate, simply add `#![doc = document_features::document_features!()]`
within your crate documentation.
The `document_features!()` reads the Cargo.toml file and generate a markdown string
suitable to be used within the documentation.

Use `## ` and `#! ` comments in your Cargo.toml to document features.

```toml
[dependencies]
document-features = "0.1"
## ...

[features]
## The foo feature is enabling the `foo` functions
foo = []
## The bar feature enable the [`bar`] module
bar = []

#! ### Experimental features
#! The following features are experimental

## Activate the fusion reactor
fusion = []
```

And these document will be rendered nicely in your documentation.

Checkout the [documentation](https://docs.rs/document-features/) for more details

## Contributions

Contributions are welcome. We accept pull requests and bug reports.

## License

MIT
