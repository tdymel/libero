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
