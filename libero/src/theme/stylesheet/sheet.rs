use super::{
    declarations::theme_declarations,
    palette::{foreground_var, theme_ends},
};
use crate::{
    CssLayer,
    css::{CssDeclaration, CssScope, Stylesheet},
    theme::{
        ColorCss, ColorShade, INDICATOR_KEYFRAMES, LOADER_KEYFRAMES, MARQUEE_KEYFRAMES,
        NOTIFICATION_KEYFRAMES, NamedColorCss, PAPER_BACKGROUND, PROGRESS_BAR_KEYFRAMES,
        RIPPLE_KEYFRAMES, SCROLL_AREA_KEYFRAMES, SKELETON_KEYFRAMES, SORTABLE_KEYFRAMES, Size,
        TEXT_FONT_FAMILY, TEXT_FONT_SIZE, TEXT_FONT_WEIGHT, TEXT_LETTER_SPACING, TEXT_LINE_HEIGHT,
        TOOLTIP_KEYFRAMES, TRANSITION_KEYFRAMES, Theme, ThemeSet,
    },
};

impl From<&Theme> for Stylesheet {
    /// One theme's sheet: its vars at `:root`, then reset, `body` and keyframes.
    /// `color-scheme` follows this theme's surface.
    fn from(theme: &Theme) -> Self {
        let mut css = root_block(":root", theme);
        css.push_str(&base_layer_and_keyframes(
            theme,
            theme.surface.color_scheme(),
        ));
        Stylesheet::from(css)
    }
}

impl From<&ThemeSet> for Stylesheet {
    /// The pair up front, so switching is one root attribute: no re-render, right
    /// on first paint, `prefers-color-scheme` without JS.
    ///
    /// ```css
    /// :root { … }                            /* the light theme */
    /// @media (prefers-color-scheme: dark) {
    ///   :root:not([data-lsx-theme]) { … }    /* the dark theme, system case */
    /// }
    /// :root[data-lsx-theme="light"] { … }
    /// :root[data-lsx-theme="dark"]  { … }
    /// ```
    ///
    /// Without a dark theme, one theme's sheet. [`ThemeSet::named`] themes are not in
    /// here: selecting one rebuilds the sheet.
    fn from(set: &ThemeSet) -> Self {
        let light = set.light_theme();
        let Some(dark) = set.dark_theme() else {
            return Stylesheet::from(light);
        };

        let mut css = root_block(":root", light);
        // The system case; `:not(..)` lets an explicit choice win, as `@media` adds no specificity.
        css.push_str(&format!(
            "@media {DARK_SCHEME_QUERY}{{{}}}",
            root_block(&format!(":root:not([{THEME_ATTRIBUTE}])"), dark)
        ));
        css.push_str(&root_block(&theme_selector(ThemeSet::LIGHT), light));
        css.push_str(&root_block(&theme_selector(ThemeSet::DARK), dark));

        // From the light theme: its colours are vars, and `font_smoothing` is not a scheme choice.
        css.push_str(&base_layer_and_keyframes(light, "light dark"));
        Stylesheet::from(css)
    }
}

/// The attribute a document root carries to pin one theme of the pair.
pub(crate) const THEME_ATTRIBUTE: &str = "data-lsx-theme";

/// The dark half's media query, which the web backend also reads the platform scheme from.
pub(crate) const DARK_SCHEME_QUERY: &str = "(prefers-color-scheme: dark)";

fn theme_selector(name: &str) -> String {
    format!(":root[{THEME_ATTRIBUTE}=\"{name}\"]")
}

fn root_block(selector: &str, theme: &Theme) -> String {
    Stylesheet::new(vec![CssScope::new(selector, theme_declarations(theme))])
        .as_str()
        .to_string()
}

/// Everything but the vars, emitted once per sheet. Reset and `body` go in `lsx-base`, or
/// they'd beat an app's layered base styles (Tailwind v4); vars and keyframes stay unlayered.
fn base_layer_and_keyframes(theme: &Theme, color_scheme: &str) -> String {
    let mut base_scopes = global_reset_scopes(theme, color_scheme);
    base_scopes.push(body_scope(theme));

    let mut css = format!(
        "@layer {}{{{}}}",
        CssLayer::Base.css_name(),
        Stylesheet::new(base_scopes).as_str()
    );

    css.push_str(RIPPLE_KEYFRAMES);
    css.push_str(PROGRESS_BAR_KEYFRAMES);
    css.push_str(LOADER_KEYFRAMES);
    css.push_str(INDICATOR_KEYFRAMES);
    css.push_str(SKELETON_KEYFRAMES);
    css.push_str(MARQUEE_KEYFRAMES);
    css.push_str(NOTIFICATION_KEYFRAMES);
    css.push_str(TOOLTIP_KEYFRAMES);
    css.push_str(TRANSITION_KEYFRAMES);
    css.push_str(SCROLL_AREA_KEYFRAMES);
    css.push_str(SORTABLE_KEYFRAMES);
    css
}

fn global_reset_scopes(theme: &Theme, color_scheme: &str) -> Vec<CssScope> {
    let mut html_declarations = vec![
        CssDeclaration::new("box-sizing", "border-box"),
        // Canvas, scrollbars and native controls follow the theme; a pair passes `light dark`.
        CssDeclaration::new("color-scheme", color_scheme),
    ];
    if theme.font_smoothing {
        html_declarations.push(CssDeclaration::new("-webkit-font-smoothing", "antialiased"));
        html_declarations.push(CssDeclaration::new("-moz-osx-font-smoothing", "grayscale"));
    }

    vec![
        CssScope::new("html", html_declarations),
        CssScope::new(
            "*, *::before, *::after",
            vec![CssDeclaration::new("box-sizing", "inherit")],
        ),
    ]
}

/// A physical `text-align` per `dir`, in the base layer, for a renderer that
/// aligns the initial `start` left under `rtl` (Blitz).
pub(crate) fn physical_text_align() -> String {
    let scopes = [("rtl", "right"), ("ltr", "left")]
        .into_iter()
        .map(|(dir, side)| {
            CssScope::new(
                format!(":where([dir={dir}])"),
                vec![CssDeclaration::new("text-align", side)],
            )
        })
        .collect();
    format!(
        "@layer {}{{{}}}",
        CssLayer::Base.css_name(),
        Stylesheet::new(scopes).as_str()
    )
}

/// A field's paper and border for raw form controls, in the base layer, for a
/// renderer that paints them white whatever `color-scheme` says (Blitz).
pub(crate) fn themed_form_controls() -> String {
    // Toggles and file/button inputs keep the UA look; text stays inherited.
    let controls = ":where(input:not([type=checkbox], [type=radio], [type=range], \
        [type=color], [type=file], [type=image], [type=submit], [type=reset], \
        [type=button]), textarea, select)";
    let scope = CssScope::new(
        controls,
        vec![
            CssDeclaration::new("background-color", PAPER_BACKGROUND.value()),
            CssDeclaration::new("border-color", ColorCss::MUTED.value(ColorShade::S6)),
        ],
    );
    format!(
        "@layer {}{{{}}}",
        CssLayer::Base.css_name(),
        Stylesheet::new(vec![scope]).as_str()
    )
}

fn body_scope(theme: &Theme) -> CssScope {
    // Ink and surface have no contrast var, so computed as `push_color_declarations` does.
    let text_color_var = foreground_var(theme.surface.contrast(), theme_ends(theme));

    CssScope::new(
        "body",
        vec![
            CssDeclaration::new("margin", "0"),
            CssDeclaration::new("background-color", NamedColorCss::SURFACE.value()),
            CssDeclaration::new("color", text_color_var),
            CssDeclaration::new("font-family", TEXT_FONT_FAMILY.value()),
            CssDeclaration::new("font-size", TEXT_FONT_SIZE.value(Size::Md)),
            CssDeclaration::new("font-weight", TEXT_FONT_WEIGHT.value(Size::Md)),
            CssDeclaration::new("line-height", TEXT_LINE_HEIGHT.value(Size::Md)),
            CssDeclaration::new("letter-spacing", TEXT_LETTER_SPACING.value(Size::Md)),
        ],
    )
}
