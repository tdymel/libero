use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    sx::ThemeAwareValue,
    theme::{GlassTint, Gradient, Theme, ThemeSet, glass_gradient_declarations, glass_tint},
};

/// The active theme. Reactive, so a Rust-side read never disagrees with the CSS vars.
/// Panics outside a `LiberoProvider`.
///
/// Docs: <https://libero-ui.dev/about/theming>
pub fn use_theme() -> &'static Theme {
    *use_context::<LiberoContext>().theme.read()
}

/// The active [`ThemeSet`], and how to swap it: a theme picker's base.
/// Reactive. A swap rebuilds the stylesheet; the colour-scheme setting survives.
/// Panics outside a `LiberoProvider`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{theme::ThemeSet, use_theme_set};
/// # fn app() -> Element {
/// let themes = use_theme_set();
///
/// rsx! {
///     for set in ThemeSet::CATALOGUE {
///         button {
///             onclick: {
///                 let themes = themes.clone();
///                 move |_| themes.set((*set).clone())
///             },
///             "{set.name()}"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-theme-set>
pub fn use_theme_set() -> ThemeSetHandle {
    ThemeSetHandle {
        context: use_context::<LiberoContext>(),
    }
}

/// The inline gradient vars for `gradient` from the component's `color`, `None`
/// unless `active` and either is set (the theme's `:root` vars cover the rest).
/// Measured on both themes of the set, since a scheme switch re-renders nothing.
/// `text` puts palette stops in the text role, for gradient text.
pub(crate) fn use_gradient_style(
    gradient: Option<&Gradient>,
    color: Option<&ThemeAwareValue>,
    active: bool,
    text: bool,
) -> Option<String> {
    let context = use_context::<LiberoContext>();
    if !active || (gradient.is_none() && color.is_none()) {
        return None;
    }
    let gradient = gradient.cloned().unwrap_or_default();
    Some(
        gradient
            .declarations(color, &scheme_themes(&context), text)
            .iter()
            .map(ToString::to_string)
            .collect(),
    )
}

/// The gradient vars of [`use_gradient_style`] plus, for a glass surface, the label
/// and share the glass raises them to; `None` under the same conditions.
pub(crate) fn use_glass_gradient_style(
    gradient: Option<&Gradient>,
    color: Option<&ThemeAwareValue>,
    active: bool,
    glass: bool,
) -> Option<String> {
    let style = use_gradient_style(gradient, color, active, false);
    // A hook: called before the early return, or toggling `glass` panics (todo 2134).
    let context = use_context::<LiberoContext>();
    if !glass {
        return style;
    }
    let gradient = gradient.cloned().unwrap_or_default();
    let glass = glass_gradient_declarations(&gradient, color, &scheme_themes(&context));
    Some(style? + &glass.iter().map(ToString::to_string).collect::<String>())
}

/// The label and share of a glass tint of `fill` (see [`glass_tint`]), measured
/// as [`use_gradient_style`] is. `None` without a `fill`.
pub(crate) fn use_glass_tint(fill: Option<&ThemeAwareValue>) -> Option<GlassTint> {
    let context = use_context::<LiberoContext>();
    glass_tint(fill?, &scheme_themes(&context))
}

/// The active theme first, then every theme a scheme switch can show.
fn scheme_themes(context: &LiberoContext) -> Vec<&'static Theme> {
    let set = context.themes.read();
    let mut themes = vec![*context.theme.read(), set.light_theme()];
    themes.extend(set.dark_theme());
    themes
}

/// What [`use_theme_set`] hands back.
#[derive(Clone)]
pub struct ThemeSetHandle {
    context: LiberoContext,
}

impl ThemeSetHandle {
    /// The active set. Cloned, because a set owns its list of extra themes.
    pub fn get(&self) -> ThemeSet {
        self.context.themes.read().clone()
    }

    /// What the active set calls itself.
    pub fn name(&self) -> &'static str {
        self.context.themes.read().name()
    }

    /// Swaps the set and rebuilds the stylesheet.
    pub fn set(&self, themes: ThemeSet) {
        self.context.set_theme_set(themes);
    }
}
