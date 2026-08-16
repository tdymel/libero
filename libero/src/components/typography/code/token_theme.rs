use crate::{CssLayer, hooks::use_css};

/// Fixed classes for `highlight::RAW_SCOPE_CLASSES` - one shared stylesheet
/// registered once (deduped by the same registry every other `use_css` call
/// goes through), not per-span inline styles. Colors are GitHub-inspired
/// approximations, not pixel-verified against GitHub's own palette.
const TOKEN_STYLESHEET: &str = "\
.lsx-tok-keyword{color:#cf222e;}\
.lsx-tok-string{color:#0a3069;}\
.lsx-tok-comment{color:#6e7781;font-style:italic;}\
.lsx-tok-number{color:#0550ae;}\
.lsx-tok-constant{color:#0550ae;}\
.lsx-tok-function{color:#8250df;}\
.lsx-tok-type{color:#953800;}\
.lsx-tok-tag{color:#116329;}\
.lsx-tok-attribute{color:#0969da;}\
.lsx-tok-heading{color:#cf222e;font-weight:600;}\
.lsx-tok-bold{font-weight:600;}\
.lsx-tok-italic{font-style:italic;}\
";

pub(crate) fn use_token_theme() {
    use_css(TOKEN_STYLESHEET, CssLayer::Framework);
}
