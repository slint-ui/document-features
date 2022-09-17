#[test]
fn ensure_it_compiles() {
    document_features::document_features!();
    document_features::document_features!(feature_label = "**`{feature}`**");
    document_features::document_features!(feature_label = r"**`{feature}`**");
    document_features::document_features!(feature_label = r#"**`{feature}`**"#);
    document_features::document_features!(
        feature_label = "<span class=\"stab portability\"><code>{feature}</code></span>"
    );
    document_features::document_features!(
        feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#
    );
    document_features::document_features!(
        feature_label = r##"<span class="stab portability"><code>{feature}</code></span>"##
    );
}

#[test]
fn self_doc() {
    let actual = document_features::document_features!();
    let expected =
        "* **`self-test`** —  Internal feature used only for the tests, don't enable\n\n";
    assert_eq!(actual, expected);
}

#[test]
fn self_doc_with_custom_label() {
    let actual = document_features::document_features!(
        feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#
    );
    let expected =
        "* <span class=\"stab portability\"><code>self-test</code></span> —  Internal feature used only for the tests, don't enable\n\n";
    assert_eq!(actual, expected);
}
