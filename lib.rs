// Copyright © SixtyFPS GmbH <info@sixtyfps.io>
// SPDX-License-Identifier: MIT OR Apache-2.0

/*!
Document your crate's feature flags.

This crates provides a macro that extracts "documentation" comments from Cargo.toml

To use this crate, add `#![doc = document_features::document_features!()]` in your crate documentation.
The `document_features!()` macro reads your `Cargo.toml` file, extracts feature comments and generates
a markdown string for your documentation.

Basic example:

```rust
//! Normal crate documentation goes here.
//!
//! ## Feature flags
#![doc = document_features::document_features!()]

// rest of the crate goes here.
```

## Documentation format:

The documentation of your crate features goes into `Cargo.toml`, where they are defined.

The `document_features!()` macro analyzes only the `[features]` section.
Similar to Rust's documentation comments `///` and `//!`, the macro understands
comments that start with `## ` and `#! `. Note the required trailing space.
Lines starting with `###` will not be understood as doc comment.

`## ` comments are meant to be *above* the feature they document.
There can be several `## ` comments, but they must always be followed by a
feature name, and no other `#! ` comments in between.

`#! ` comments are not associated with a particular feature, and will be printed
in where they occur. Use them to group features, for example.

## Examples:

*/
// Note: because rustdoc escapes the first `#` of a line starting with `#`,
// these docs comments have one more `#` ,
#![doc = self_test!(/**
[package]
name = "..."
## ...

[features]
default = ["foo"]
##! This comments goes on top

### The foo feature enables the `foo` functions
foo = []

### The bar feature enables the bar module
bar = []

##! ### Experimental features
##! The following features are experimental

### Enable the fusion reactor
fusion = []

[dependencies]
##! ### Optional dependencies

### Enable this when building the docs
document-features = { version = "0.1", optional = true }

### This awesome dependency is specified in its own table
[dependencies.awesome]
version = "1.3.5"
optional = true
*/
=>
    /**
This comments goes on top
* **`foo`** *(enabled by default)* —  The foo feature enables the `foo` functions

* **`bar`** —  The bar feature enables the bar module

#### Experimental features
The following features are experimental
* **`fusion`** —  Enable the fusion reactor

#### Optional dependencies
* **`document-features`** —  Enable this when building the docs

* **`awesome`** —  This awesome dependency is specified in its own table

*/
)]
/*!

## Compatibility

The minimum Rust version required to use this crate is Rust 1.54 because of the
feature to have macro in doc comments. You can make this crate optional and use
`#[cfg_attr()]` statements to enable it only when building the documentation:
You need to have two levels of `cfg_attr` because Rust < 1.54 doesn't parse the attribute
otherwise.

```rust,ignore
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
```

In your Cargo.toml, enable this feature while generating the documentation on docs.rs:

```toml
[dependencies]
document-features = { version = "0.1", optional = true }

[package.metadata.docs.rs]
features = ["document-features"]
## Alternative: enable all features so they are all documented
## all-features = true
```
 */

extern crate proc_macro;
use proc_macro::TokenStream;

use std::collections::HashSet;
use std::fmt::Write;
use std::path::Path;
use std::str::FromStr;

fn error(e: &str) -> TokenStream {
    TokenStream::from_str(&format!("::core::compile_error!{{\"{}\"}}", e.escape_default())).unwrap()
}

/// Produce a literal string containing documentation extracted from Cargo.toml
///
/// See the [crate] documentation for details
#[proc_macro]
pub fn document_features(_: TokenStream) -> TokenStream {
    document_features_impl().unwrap_or_else(std::convert::identity)
}

fn document_features_impl() -> Result<TokenStream, TokenStream> {
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut cargo_toml = std::fs::read_to_string(Path::new(&path).join("Cargo.toml"))
        .map_err(|e| error(&format!("Can't open Cargo.toml: {:?}", e)))?;

    if !cargo_toml.contains("\n##") && !cargo_toml.contains("\n#!") {
        // On crates.io, Cargo.toml is usually "normalized" and stripped of all comments.
        // The original Cargo.toml has been renamed Cargo.toml.orig
        if let Ok(orig) = std::fs::read_to_string(Path::new(&path).join("Cargo.toml.orig")) {
            if orig.contains("##") || orig.contains("#!") {
                cargo_toml = orig;
            }
        }
    }

    let result = process_toml(&cargo_toml).map_err(|e| error(&e))?;
    Ok(std::iter::once(proc_macro::TokenTree::from(proc_macro::Literal::string(&result))).collect())
}

fn process_toml(cargo_toml: &str) -> Result<String, String> {
    // Get all lines between the "[features]" and the next block
    let mut lines = cargo_toml
        .lines()
        .map(str::trim)
        // and skip empty lines and comments that are not docs comments
        .filter(|l| {
            !l.is_empty() && (!l.starts_with("#") || l.starts_with("##") || l.starts_with("#!"))
        });
    let mut top_comment = String::new();
    let mut current_comment = String::new();
    let mut features = vec![];
    let mut default_features = HashSet::new();
    let mut current_table = "";
    while let Some(line) = lines.next() {
        if let Some(x) = line.strip_prefix("#!") {
            if !x.is_empty() && !x.starts_with(" ") {
                continue; // it's not a doc comment
            }
            if !current_comment.is_empty() {
                return Err("Cannot mix ## and #! comments between features.".into());
            }
            writeln!(top_comment, "{}", x).unwrap();
        } else if let Some(x) = line.strip_prefix("##") {
            if !x.is_empty() && !x.starts_with(" ") {
                continue; // it's not a doc comment
            }
            writeln!(current_comment, "{}", x).unwrap();
        } else if let Some(table) = line.strip_prefix("[") {
            current_table = table
                .split_once("]")
                .map(|(t, _)| t.trim())
                .ok_or_else(|| format!("Parse error while parsing line: {}", line))?;
            if !current_comment.is_empty() {
                let dep = current_table
                    .rsplit_once(".")
                    .and_then(|(table, dep)| table.trim().ends_with("dependencies").then(|| dep))
                    .ok_or_else(|| format!("Not a feature: `{}`", line))?;
                features.push((
                    dep.trim(),
                    std::mem::take(&mut top_comment),
                    std::mem::take(&mut current_comment),
                ));
            }
        } else if let Some((dep, rest)) = line.split_once("=") {
            if current_table == "features" && dep.trim() == "default" {
                let defaults = rest
                    .trim()
                    .strip_prefix("[")
                    .and_then(|r| r.strip_suffix("]"))
                    .ok_or_else(|| format!("Parse error while parsing dependency {}", dep))?
                    .split(",")
                    .map(|d| d.trim().trim_matches(|c| c == '"' || c == '\'').trim())
                    .filter(|d| !d.is_empty());
                default_features.extend(defaults);
            }
            if !current_comment.is_empty() {
                if current_table.ends_with("dependencies") {
                    if !rest
                        .split_once("optional")
                        .and_then(|(_, r)| r.trim().strip_prefix("="))
                        .map_or(false, |r| r.trim().starts_with("true"))
                    {
                        return Err(format!(
                            "Dependency {} is not an optional dependency",
                            dep.trim()
                        ));
                    }
                } else if current_table != "features" {
                    return Err(format!(
                        "Comment cannot be associated with a feature: {}",
                        current_comment
                    ));
                }
                features.push((
                    dep.trim(),
                    std::mem::take(&mut top_comment),
                    std::mem::take(&mut current_comment),
                ));
            }
        }
    }
    if !current_comment.is_empty() {
        return Err("Found comment not associated with a feature".into());
    }
    if features.is_empty() {
        return Err("Could not find documented features in Cargo.toml".into());
    }
    let mut result = String::new();
    for (f, top, comment) in features {
        let default = if default_features.contains(f) { " *(enabled by default)*" } else { "" };
        if !comment.trim().is_empty() {
            writeln!(result, "{}* **`{}`**{} — {}", top, f, default, comment).unwrap();
        } else {
            writeln!(result, "{}* **`{}`**{}\n", top, f, default).unwrap();
        }
    }
    result += &top_comment;
    Ok(result)
}

#[cfg(feature = "self-test")]
#[proc_macro]
#[doc(hidden)]
/// Helper macro for the tests. Do not use
pub fn self_test_helper(input: TokenStream) -> TokenStream {
    process_toml((&input).to_string().trim_matches(|c| c == '"' || c == '#')).map_or_else(
        |e| error(&e),
        |r| std::iter::once(proc_macro::TokenTree::from(proc_macro::Literal::string(&r))).collect(),
    )
}

#[cfg(feature = "self-test")]
macro_rules! self_test {
    (#[doc = $toml:literal] => #[doc = $md:literal]) => {
        concat!(
            "\n`````rust\n\
            fn normalize_md(md : &str) -> String {
               md.lines().skip_while(|l| l.is_empty()).map(|l| l.trim())
                .collect::<Vec<_>>().join(\"\\n\")
            }
            assert_eq!(normalize_md(document_features::self_test_helper!(",
            stringify!($toml),
            ")), normalize_md(",
            stringify!($md),
            "));\n`````\n\n"
        )
    };
}

#[cfg(not(feature = "self-test"))]
macro_rules! self_test {
    (#[doc = $toml:literal] => #[doc = $md:literal]) => {
        concat!(
            "This contents in Cargo.toml:\n`````toml",
            $toml,
            "\n`````\n Generates the following:\n\
            <table><tr><th>Preview</th></tr><tr><td>\n\n",
            $md,
            "\n</td></tr></table>\n\n&nbsp;\n",
        )
    };
}

#[cfg(test)]
mod tests {
    use super::process_toml;

    #[track_caller]
    fn test_error(toml: &str, expected: &str) {
        let err = process_toml(toml).unwrap_err();
        assert!(err.contains(expected), "{:?} does not contain {:?}", err, expected)
    }

    #[test]
    fn parse_error1() {
        test_error(
            r#"
[features]
[dependencies]
foo = 4;
"#,
            "Could not find documented features",
        );
    }

    #[test]
    fn parse_error2() {
        test_error(
            r#"
[packages]
[dependencies]
"#,
            "Could not find documented features",
        );
    }

    #[test]
    fn parse_error3() {
        test_error(
            r#"
[features]
ff = []
[abcd
efgh
[dependencies]
"#,
            "Parse error while parsing line: [abcd",
        );
    }

    #[test]
    fn parse_error4() {
        test_error(
            r#"
[features]
## dd
## ff
#! ee
## ff
"#,
            "Cannot mix",
        );
    }

    #[test]
    fn parse_error5() {
        test_error(
            r#"
[features]
## dd
"#,
            "not associated with a feature",
        );
    }

    #[test]
    fn parse_error6() {
        test_error(
            r#"
[features]
# ff
foo = []
default = [
#ffff
# ff
"#,
            "Parse error while parsing dependency default",
        );
    }

    #[test]
    fn not_a_feature1() {
        test_error(
            r#"
## hallo
[features]
"#,
            "Not a feature: `[features]`",
        );
    }

    #[test]
    fn not_a_feature2() {
        test_error(
            r#"
[package]
## hallo
foo = []
"#,
            "Comment cannot be associated with a feature:  hallo",
        );
    }

    #[test]
    fn non_optional_dep1() {
        test_error(
            r#"
[dev-dependencies]
## Not optional
foo = { version = "1.2", optional = false }
"#,
            "Dependency foo is not an optional dependency",
        );
    }

    #[test]
    fn non_optional_dep2() {
        test_error(
            r#"
[dev-dependencies]
## Not optional
foo = { version = "1.2" }
"#,
            "Dependency foo is not an optional dependency",
        );
    }

    #[test]
    fn basic() {
        assert_eq!(
            process_toml(
                r#"
[abcd]
[features]#xyz
#! abc
#
###
#! def
#!
## 123
## 456
feat1 = ["plop"]
#! ghi
no_doc = []
##
feat2 = ["momo"]
#! klm
default = ["feat1", "something_else"]
#! end
        "#
            )
            .unwrap(),
            " abc\n def\n\n* **`feat1`** *(enabled by default)* —  123\n 456\n\n ghi\n* **`feat2`**\n\n klm\n end\n"
        );
    }

    #[test]
    fn dependencies() {
        assert_eq!(
            process_toml(
                r#"
#! top
[dev-dependencies] #yo
## dep1
dep1 = { version="1.2", optional=true}
#! yo
dep2 = "1.3"
## dep3
[target.'cfg(unix)'.build-dependencies.dep3]
version = "42"
optional = true
        "#
            )
            .unwrap(),
            " top\n* **`dep1`** —  dep1\n\n yo\n* **`dep3`** —  dep3\n\n"
        );
    }
}
