use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    theme::{Gradient, Theme, ThemeSet},
};

/// The active theme. Reactive, so a Rust-side read never disagrees with the CSS vars.
///
/// Docs: <https://libero-ui.dev/about/theming>
pub fn use_theme() -> &'static Theme {
    *use_context::<LiberoContext>().theme.read()
}

/// The active [`ThemeSet`], and how to swap it: a theme picker's base.
/// Reactive. A swap rebuilds the stylesheet; the colour-scheme setting survives.
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

/// The inline gradient vars for `gradient`, `None` unless `active`. Measured
/// on both themes of the set, since a scheme switch re-renders nothing.
/// `text` puts palette stops in the text role, for gradient text.
pub(crate) fn use_gradient_style(
    gradient: Option<&Gradient>,
    active: bool,
    text: bool,
) -> Option<String> {
    let context = use_context::<LiberoContext>();
    let gradient = gradient.filter(|_| active)?;
    let active = *context.theme.read();
    let set = context.themes.read();
    let mut themes = vec![active, set.light_theme()];
    themes.extend(set.dark_theme());
    Some(
        gradient
            .declarations(&themes, text)
            .iter()
            .map(ToString::to_string)
            .collect(),
    )
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
