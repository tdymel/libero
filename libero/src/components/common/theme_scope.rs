use dioxus::prelude::*;

use crate::{
    LiberoContext,
    context::{CssLayer, ThemeScopeContext},
    css::Stylesheet,
    hooks::use_css,
    theme::{THEME_SCOPE_ATTRIBUTE, theme_scope_css},
    utils::warn,
};

/// Shows its children in another theme of the provider's [`ThemeSet`](crate::theme::ThemeSet),
/// such as a dark preview on a light page. The box paints that theme's surface and ink, and every
/// colour var, `color-scheme` and overlay opened inside follow it. An unknown name warns in a
/// debug build and leaves the children in the page's theme.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{ThemeScope, components::Button};
/// # fn app() -> Element {
/// rsx! {
///     ThemeScope { theme: "dark",
///         Button { "Dark, whatever the page shows" }
///     }
/// }
/// # }
/// ```
///
/// Values read in Rust (`use_theme()`) and `Sx::dark`/`Sx::light` keep following the page.
///
/// Docs: <https://libero-ui.dev/about/theming>
#[component]
pub fn ThemeScope(
    /// A theme name of the provider's set: `"light"`, `"dark"` or a named one; reactive.
    theme: ReadSignal<String>,
    children: Element,
) -> Element {
    let themes = use_context::<LiberoContext>().themes;
    let css = use_memo(move || {
        let css = theme_scope_css(&themes.read(), &theme.read());
        if css.is_none() {
            warn(&format!(
                "ThemeScope: unknown theme `{}`; the set has no such theme, so the children keep the page's",
                theme.read()
            ));
        }
        css
    });
    let scope = use_memo(move || css.read().is_some().then(|| theme.cloned()));
    use_context_provider(|| ThemeScopeContext(scope));
    use_css(
        css().map(|css| Stylesheet::from(&*css)),
        CssLayer::Framework,
    );

    let attribute = vec![Attribute::new(
        THEME_SCOPE_ATTRIBUTE,
        scope().map_or(dioxus::core::AttributeValue::None, |name| {
            dioxus::core::AttributeValue::Text(name)
        }),
        None,
        false,
    )];
    rsx! {
        div { ..attribute, {children} }
    }
}
