//! `index.html` inlines the colour-scheme restore script by hand; this pins the copy.

const INDEX_HTML: &str = include_str!("../index.html");

#[test]
fn the_inline_restore_script_is_the_libero_constant() {
    let scripts: Vec<&str> = INDEX_HTML
        .split("<script>")
        .skip(1)
        .filter_map(|rest| rest.split_once("</script>").map(|(body, _)| body))
        .collect();
    assert_eq!(scripts, [libero::theme::COLOR_SCHEME_RESTORE_SCRIPT]);
}

/// The `content` of the `<meta>` tag whose `name` or `property` is `key`.
fn meta(key: &str) -> &'static str {
    let tag = INDEX_HTML
        .split("<meta ")
        .find(|tag| tag.contains(&format!("=\"{key}\"")))
        .unwrap_or_else(|| panic!("no <meta> for {key}"));
    let content = tag.split_once("content=\"").unwrap().1;
    content.split_once('"').unwrap().0
}

#[test]
fn the_site_wide_head_matches_the_site_constants() {
    let description = meta("description");
    assert_eq!(meta("og:description"), description);
    assert_eq!(meta("twitter:description"), description);
    let image = format!("{}/og-image.png", crate::site::SITE);
    assert_eq!(meta("og:image"), image);
    assert_eq!(meta("twitter:image"), image);
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    assert!(public.join("og-image.png").exists());

    let (_, rest) = INDEX_HTML
        .split_once("<script type=\"application/ld+json\">")
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(rest.split_once("</script>").unwrap().0).unwrap();
    let graph = json["@graph"].as_array().unwrap();
    for node in graph {
        assert_eq!(node["url"], crate::site::SITE);
    }
    let code = graph
        .iter()
        .find(|node| node["@type"] == "SoftwareSourceCode")
        .unwrap();
    assert_eq!(code["codeRepository"], crate::site::GITHUB);
}
