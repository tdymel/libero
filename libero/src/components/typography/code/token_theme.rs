use std::sync::LazyLock;

use crate::{
    CssLayer,
    css::Stylesheet,
    hooks::use_css,
    theme::{
        CODE_TOK_ATTRIBUTE, CODE_TOK_COMMENT, CODE_TOK_CONSTANT, CODE_TOK_FUNCTION,
        CODE_TOK_HEADING, CODE_TOK_KEYWORD, CODE_TOK_NUMBER, CODE_TOK_STRING, CODE_TOK_TAG,
        CODE_TOK_TYPE,
    },
};

/// Fixed classes for `highlight::RAW_SCOPE_CLASSES` - one shared stylesheet
/// registered once (deduped by the same registry every other `use_css` call
/// goes through), not per-span inline styles. Colors come from the theme
/// (`CodeDefaults`), GitHub-inspired approximations by default.
static TOKEN_STYLESHEET: LazyLock<String> = LazyLock::new(|| {
    format!(
        ".lsx-tok-keyword{{color:{};}}\
         .lsx-tok-string{{color:{};}}\
         .lsx-tok-comment{{color:{};font-style:italic;}}\
         .lsx-tok-number{{color:{};}}\
         .lsx-tok-constant{{color:{};}}\
         .lsx-tok-function{{color:{};}}\
         .lsx-tok-type{{color:{};}}\
         .lsx-tok-tag{{color:{};}}\
         .lsx-tok-attribute{{color:{};}}\
         .lsx-tok-heading{{color:{};font-weight:600;}}\
         .lsx-tok-bold{{font-weight:600;}}\
         .lsx-tok-italic{{font-style:italic;}}",
        CODE_TOK_KEYWORD.value(),
        CODE_TOK_STRING.value(),
        CODE_TOK_COMMENT.value(),
        CODE_TOK_NUMBER.value(),
        CODE_TOK_CONSTANT.value(),
        CODE_TOK_FUNCTION.value(),
        CODE_TOK_TYPE.value(),
        CODE_TOK_TAG.value(),
        CODE_TOK_ATTRIBUTE.value(),
        CODE_TOK_HEADING.value(),
    )
});

pub(crate) fn use_token_theme() {
    use_css(
        Some(Stylesheet::from(TOKEN_STYLESHEET.as_str())),
        CssLayer::Framework,
    );
}
