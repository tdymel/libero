use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    theme::{Theme, ThemeSet},
};

/// The active theme.
///
/// Reactive: a component that reads a spacing number, a label or a `HexColor`
/// off it re-renders when the theme changes, so a Rust-side read never
/// disagrees with the CSS vars that changed without it.
pub fn use_theme() -> &'static Theme {
    *use_context::<LiberoContext>().theme.read()
}

/// The active [`ThemeSet`], and how to swap it - what a theme picker is
/// built on.
///
/// Reactive: a picker showing the current set's name re-renders when it
/// changes. Swapping a set rebuilds the stylesheet, because the pair the
/// sheet carries is *this* set's pair; the colour-scheme setting survives
/// the swap, so a reader who pinned dark stays in dark.
///
/// ```ignore
/// let themes = use_theme_set();
///
/// for set in ThemeSet::CATALOGUE {
///     rsx! {
///         Button {
///             onclick: move |_| themes.set((*set).clone()),
///             "{set.name()}"
///         }
///     }
/// }
/// ```
pub fn use_theme_set() -> ThemeSetHandle {
    ThemeSetHandle {
        context: use_context::<LiberoContext>(),
    }
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

    pub fn set(&self, themes: ThemeSet) {
        self.context.set_theme_set(themes);
    }
}
